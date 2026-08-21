//! TLS transport for the KDE Connect pairing path.
//!
//! The migration keeps a live TLS connection while the desktop UI decides
//! whether an incoming device is trusted. This makes Allow/Reject an actual
//! network operation instead of a GUI-only simulation.

use super::packet::{IdentityPacket, Packet};
use super::tls::{LocalTlsIdentity, TrustStore, TrustedPeer};
use anyhow::{Context, Result};
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread, time::{SystemTime, UNIX_EPOCH}};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

pub const TLS_PAIRING_PORT: u16 = 1716;

#[derive(Debug, Clone)]
pub struct IncomingPairing {
    pub identity: IdentityPacket,
    pub fingerprint: String,
}

#[derive(Debug, Clone)]
pub enum PairingDecision {
    Accept,
    Reject,
}

#[derive(Clone)]
pub struct TlsPairingServer {
    pub identity: LocalTlsIdentity,
    trust_store: Arc<Mutex<TrustStore>>,
    trust_path: std::path::PathBuf,
}

impl TlsPairingServer {
    pub fn new(identity: LocalTlsIdentity, trust_path: impl Into<std::path::PathBuf>) -> Self {
        let trust_path = trust_path.into();
        let trust_store = TrustStore::load(&trust_path).unwrap_or_default();
        Self { identity, trust_store: Arc::new(Mutex::new(trust_store)), trust_path }
    }

    pub fn trusted_store(&self) -> Arc<Mutex<TrustStore>> {
        Arc::clone(&self.trust_store)
    }

    pub fn spawn<F>(&self, bind_addr: &str, on_pairing: F) -> Result<thread::JoinHandle<()>>
    where
        F: Fn(IncomingPairing, PairingResponder) + Send + Sync + 'static,
    {
        let listener = TcpListener::bind(bind_addr)
            .with_context(|| format!("bind TLS pairing listener on {bind_addr}"))?;
        let config = Arc::new(self.server_config()?);
        let identity = self.identity.clone();
        let callback = Arc::new(on_pairing);
        let trust_store = Arc::clone(&self.trust_store);
        let trust_path = self.trust_path.clone();

        Ok(thread::spawn(move || {
            for incoming in listener.incoming() {
                let stream = match incoming {
                    Ok(stream) => stream,
                    Err(error) => { log::warn!("KDE Connect TLS listener error: {error}"); continue; }
                };
                let config = Arc::clone(&config);
                let identity = identity.clone();
                let callback = Arc::clone(&callback);
                let trust_store = Arc::clone(&trust_store);
                let trust_path = trust_path.clone();
                thread::spawn(move || {
                    if let Err(error) = handle_client(stream, config, identity, callback, trust_store, trust_path) {
                        log::warn!("KDE Connect TLS pairing connection ended: {error:#}");
                    }
                });
            }
        }))
    }

    fn server_config(&self) -> Result<ServerConfig> {
        Ok(ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![self.identity.rustls_certificate()], self.identity.rustls_private_key())?)
    }
}

pub struct PairingResponder {
    stream: Arc<Mutex<Option<StreamOwned<ServerConnection, TcpStream>>>>,
    peer: IdentityPacket,
    fingerprint: String,
    trust_store: Arc<Mutex<TrustStore>>,
    trust_path: std::path::PathBuf,
}

impl PairingResponder {
    pub fn decide(&self, decision: PairingDecision) -> Result<()> {
        let accepted = matches!(decision, PairingDecision::Accept);
        let packet_id = 1;
        let packet = Packet::pair_response(packet_id, accepted);
        if accepted {
            let mut store = self.trust_store.lock().unwrap();
            store.trust(TrustedPeer {
                device_id: self.peer.device_id.clone(),
                device_name: self.peer.device_name.clone(),
                fingerprint: self.fingerprint.clone(),
                paired_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
            });
            store.save(&self.trust_path)?;
        }
        if let Some(stream) = self.stream.lock().unwrap().as_mut() {
            write_packet(stream, &packet)?;
        }
        Ok(())
    }
}

fn handle_client(
    stream: TcpStream,
    config: Arc<ServerConfig>,
    local: LocalTlsIdentity,
    callback: Arc<dyn Fn(IncomingPairing, PairingResponder) + Send + Sync>,
    trust_store: Arc<Mutex<TrustStore>>,
    trust_path: std::path::PathBuf,
) -> Result<()> {
    let connection = ServerConnection::new(config)?;
    let tls = StreamOwned::new(connection, stream);
    let stream = Arc::new(Mutex::new(Some(tls)));

    {
        let mut guard = stream.lock().unwrap();
        let tls = guard.as_mut().unwrap();
        let local_packet = IdentityPacket {
            device_id: local.fingerprint.replace(':', "").to_lowercase(),
            device_name: std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_else(|_| "PhoneBridge PC".into()),
            device_type: "desktop".into(),
            incoming_capabilities: vec!["kdeconnect.ping".into(), "kdeconnect.share".into()],
            outgoing_capabilities: vec!["kdeconnect.ping".into(), "kdeconnect.share".into()],
            protocol_version: 8,
            transport_fields: Default::default(),
        };
        write_packet(tls, &Packet::identity(0, &local_packet)?)?;
    }

    let packet = {
        let mut guard = stream.lock().unwrap();
        read_packet(guard.as_mut().unwrap())?.context("peer closed before identity/pair packet")?
    };

    anyhow::ensure!(packet.packet_type == "kdeconnect.identity", "first packet must be kdeconnect.identity");
    let identity: IdentityPacket = serde_json::from_value(packet.body)?;
    let fingerprint = identity
        .transport_fields
        .get("certificateFingerprint")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let already_trusted = trust_store.lock().unwrap().is_trusted(&identity.device_id, &fingerprint);
    let responder = PairingResponder { stream: Arc::clone(&stream), peer: identity.clone(), fingerprint: fingerprint.clone(), trust_store: Arc::clone(&trust_store), trust_path: trust_path.clone() };

    if already_trusted {
        responder.decide(PairingDecision::Accept)?;
        return Ok(());
    }

    let pair_packet = {
        let mut guard = stream.lock().unwrap();
        read_packet(guard.as_mut().unwrap())?.context("peer closed before pair request")?
    };
    anyhow::ensure!(pair_packet.packet_type == "kdeconnect.pair", "expected kdeconnect.pair after identity");
    anyhow::ensure!(pair_packet.as_pair()?.pair, "peer did not request pairing");

    callback(IncomingPairing { identity, fingerprint }, responder);
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
