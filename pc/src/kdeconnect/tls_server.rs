//! TLS transport for the KDE Connect pairing path.
//!
//! The first migration step uses server-authenticated TLS so the PC has a
//! stable cryptographic identity. Peer certificate authentication will be
//! enabled once the Android side sends a persistent client certificate.

use super::packet::{IdentityPacket, Packet};
use super::tls::LocalTlsIdentity;
use anyhow::{Context, Result};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, sync::Arc, thread};

pub const TLS_PAIRING_PORT: u16 = 1716;

#[derive(Debug, Clone)]
pub struct TlsPairingServer {
    pub identity: LocalTlsIdentity,
}

impl TlsPairingServer {
    pub fn new(identity: LocalTlsIdentity) -> Self {
        Self { identity }
    }

    pub fn spawn<F>(&self, bind_addr: &str, on_packet: F) -> Result<thread::JoinHandle<()>>
    where
        F: Fn(Packet) + Send + Sync + 'static,
    {
        let listener = TcpListener::bind(bind_addr)
            .with_context(|| format!("bind TLS pairing listener on {bind_addr}"))?;
        let config = self.server_config()?;
        let config = Arc::new(config);
        let identity = self.identity.clone();
        let callback = Arc::new(on_packet);

        Ok(thread::spawn(move || {
            for incoming in listener.incoming() {
                match incoming {
                    Ok(stream) => {
                        let config = Arc::clone(&config);
                        let identity = identity.clone();
                        let callback = Arc::clone(&callback);
                        thread::spawn(move || {
                            if let Err(error) = handle_client(stream, config, identity, callback) {
                                log::warn!("KDE Connect TLS pairing connection ended: {error:#}");
                            }
                        });
                    }
                    Err(error) => log::warn!("KDE Connect TLS listener error: {error}"),
                }
            }
        }))
    }

    fn server_config(&self) -> Result<ServerConfig> {
        let cert = self.identity.rustls_certificate();
        let key = self.identity.rustls_private_key();
        Ok(ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)?)
    }
}

fn handle_client(
    stream: TcpStream,
    config: Arc<ServerConfig>,
    identity: LocalTlsIdentity,
    callback: Arc<dyn Fn(Packet) + Send + Sync>,
) -> Result<()> {
    let connection = ServerConnection::new(config)?;
    let mut tls = StreamOwned::new(connection, stream);

    let local = IdentityPacket {
        device_id: identity.fingerprint.replace(':', "").to_lowercase(),
        device_name: std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "PhoneBridge PC".into()),
        device_type: "desktop".into(),
        incoming_capabilities: vec!["kdeconnect.ping".into(), "kdeconnect.share".into()],
        outgoing_capabilities: vec!["kdeconnect.ping".into(), "kdeconnect.share".into()],
        protocol_version: 8,
        transport_fields: Default::default(),
    };
    write_packet(&mut tls, &Packet::identity(0, &local)?)?;

    if let Some(packet) = read_packet(&mut tls)? {
        callback(packet);
    }
    Ok(())
}

fn write_packet(stream: &mut impl Write, packet: &Packet) -> Result<()> {
    let payload = serde_json::to_vec(packet)?;
    let len = u32::try_from(payload.len()).context("KDE Connect packet too large")?;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

fn read_packet(stream: &mut impl Read) -> Result<Option<Packet>> {
    let mut header = [0u8; 4];
    match stream.read_exact(&mut header) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    let length = u32::from_be_bytes(header) as usize;
    anyhow::ensure!(length > 0 && length <= 4 * 1024 * 1024, "invalid KDE Connect packet length");
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload)?;
    Ok(Some(serde_json::from_slice(&payload)?))
}
