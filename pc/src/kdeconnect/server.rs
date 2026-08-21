//! Minimal framed TCP server for the KDE Connect migration.
//!
//! This is deliberately transport-focused: it implements KDE Connect's
//! length-prefixed JSON packet framing and pairing handshake, while keeping
//! TLS certificate authentication as the next transport step. The GUI talks
//! to this server through events instead of knowing anything about sockets.

use super::{IdentityPacket, Packet};
use anyhow::{Context, Result};
use std::sync::mpsc::Sender;
use std::thread;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

pub const DEFAULT_PAIRING_PORT: u16 = 1716;
const MAX_PACKET_SIZE: usize = 1024 * 1024;

#[derive(Debug)]
pub enum PairingServerEvent {
    Listening { address: String },
    Request {
        identity: IdentityPacket,
        decision: oneshot::Sender<bool>,
    },
    Connected { address: String },
    Error(String),
}

pub struct PairingServer;

impl PairingServer {
    /// Spawn the prototype pairing listener on a dedicated Tokio runtime.
    pub fn spawn(port: u16, events: Sender<PairingServerEvent>) -> Result<()> {
        thread::Builder::new()
            .name("phonebridge-kdeconnect-pairing".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let _ = events.send(PairingServerEvent::Error(format!(
                            "failed to create pairing runtime: {error}"
                        )));
                        return;
                    }
                };

                runtime.block_on(async move {
                    if let Err(error) = run_listener(port, events.clone()).await {
                        let _ = events.send(PairingServerEvent::Error(error.to_string()));
                    }
                });
            })
            .context("failed to spawn pairing listener thread")?;

        Ok(())
    }
}

async fn run_listener(port: u16, events: Sender<PairingServerEvent>) -> Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .with_context(|| format!("failed to bind KDE Connect pairing port {port}"))?;

    let address = listener
        .local_addr()
        .context("failed to read pairing listener address")?;
    let _ = events.send(PairingServerEvent::Listening {
        address: address.to_string(),
    });

    loop {
        let (stream, peer) = listener.accept().await.context("pairing accept failed")?;
        let events = events.clone();
        tokio::spawn(async move {
            let _ = events.send(PairingServerEvent::Connected {
                address: peer.to_string(),
            });

            if let Err(error) = handle_connection(stream, events.clone()).await {
                let _ = events.send(PairingServerEvent::Error(format!(
                    "{peer}: {error:#}"
                )));
            }
        });
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    events: Sender<PairingServerEvent>,
) -> Result<()> {
    // Send our identity immediately. The fields can be replaced with the
    // persisted PhoneBridge identity once the certificate/trust layer lands.
    let identity = local_identity();
    write_packet(&mut stream, &Packet::identity(0, &identity)?).await?;

    loop {
        let packet = read_packet(&mut stream).await?;

        match packet.packet_type.as_str() {
            "kdeconnect.identity" => {
                let identity: IdentityPacket = serde_json::from_value(packet.body)
                    .context("invalid kdeconnect.identity body")?;

                let (decision_tx, decision_rx) = oneshot::channel();
                events
                    .send(PairingServerEvent::Request {
                        identity,
                        decision: decision_tx,
                    })
                    .map_err(|_| anyhow::anyhow!("pairing UI event channel closed"))?;

                let accepted = decision_rx
                    .await
                    .context("pairing UI closed before a decision")?;

                // KDE Connect expects pair=true to accept and pair=false to
                // reject a pairing request. A timestamp is required only on
                // a new pairing request, not on this response.
                write_packet(&mut stream, &Packet::pair_response(0, accepted)).await?;

                if accepted {
                    return Ok(());
                }
            }
            "kdeconnect.pair" => {
                let pair = packet.as_pair()?;
                if pair.pair {
                    let (decision_tx, decision_rx) = oneshot::channel();
                    events
                        .send(PairingServerEvent::Request {
                            identity: identity_from_peer_hint(&stream),
                            decision: decision_tx,
                        })
                        .map_err(|_| anyhow::anyhow!("pairing UI event channel closed"))?;

                    let accepted = decision_rx
                        .await
                        .context("pairing UI closed before a decision")?;
                    write_packet(&mut stream, &Packet::pair_response(0, accepted)).await?;
                    if accepted {
                        return Ok(());
                    }
                }
            }
            _ => {
                // Before pairing, ignore all non-core packets. This mirrors
                // KDE Connect's rule that normal packets require pairing.
            }
        }
    }
}

fn local_identity() -> IdentityPacket {
    IdentityPacket {
        device_id: "phonebridge0000000000000000000000".into(),
        device_name: "PhoneBridge PC".into(),
        device_type: "desktop".into(),
        incoming_capabilities: vec!["kdeconnect.ping".into()],
        outgoing_capabilities: vec!["kdeconnect.ping".into()],
        protocol_version: 8,
        transport_fields: Default::default(),
    }
}

fn identity_from_peer_hint(_stream: &TcpStream) -> IdentityPacket {
    // A pair packet can arrive before identity on a test peer. Keep a clear
    // placeholder rather than inventing device metadata. Production pairing
    // will require identity before accepting the connection.
    IdentityPacket {
        device_id: "unknown".into(),
        device_name: "Unknown device".into(),
        device_type: "phone".into(),
        incoming_capabilities: Vec::new(),
        outgoing_capabilities: Vec::new(),
        protocol_version: 8,
        transport_fields: Default::default(),
    }
}

async fn read_packet(stream: &mut TcpStream) -> Result<Packet> {
    let length = stream.read_u32().await.context("failed to read packet length")? as usize;
    if length == 0 || length > MAX_PACKET_SIZE {
        anyhow::bail!("invalid KDE Connect packet length: {length}");
    }

    let mut bytes = vec![0u8; length];
    stream
        .read_exact(&mut bytes)
        .await
        .context("failed to read KDE Connect packet body")?;

    Ok(serde_json::from_slice(&bytes).context("invalid KDE Connect JSON packet")?)
}

async fn write_packet(stream: &mut TcpStream, packet: &Packet) -> Result<()> {
    let bytes = serde_json::to_vec(packet).context("failed to encode KDE Connect packet")?;
    if bytes.len() > MAX_PACKET_SIZE {
        anyhow::bail!("KDE Connect packet exceeds maximum size");
    }

    stream
        .write_u32(bytes.len() as u32)
        .await
        .context("failed to write packet length")?;
    stream
        .write_all(&bytes)
        .await
        .context("failed to write KDE Connect packet")?;
    stream.flush().await.context("failed to flush packet")?;
    Ok(())
}
