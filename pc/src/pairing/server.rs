//! TLS control-plane server and live desktop pairing command bridge.

use crate::connection::session::ControlSession;
use crate::connection::state::ConnectionState;
use crate::pairing::identity::Identity;
use crate::pairing::trust::TrustStore;
use crate::pairing::ui_commands::PairingCommandHub;
use crate::protocol::{Message, PROTOCOL_VERSION};
use crate::ui::{PairingUiCommand, UiBackend};
use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout, Duration, Instant};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

pub const PAIRING_PORT: u16 = 17591;
const HANDSHAKE_READ_TIMEOUT: Duration = Duration::from_secs(15);
const SESSION_READ_TIMEOUT: Duration = Duration::from_secs(45);

pub struct PairingServer {
    acceptor: TlsAcceptor,
    trust_store: Arc<Mutex<TrustStore>>,
    identity: Arc<Identity>,
    ui: Arc<dyn UiBackend>,
    command_hub: PairingCommandHub,
}

impl PairingServer {
    pub fn new(identity: Arc<Identity>, trust_store: Arc<Mutex<TrustStore>>, ui: Arc<dyn UiBackend>) -> Result<Self> {
        let cert_der = load_cert_chain(&identity.cert_pem)?;
        let key_der = load_private_key(&identity.key_pem)?;
        let config = ServerConfig::builder().with_no_client_auth().with_single_cert(cert_der, key_der).context("building TLS server config")?;
        Ok(Self { acceptor: TlsAcceptor::from(Arc::new(config)), trust_store, identity, ui, command_hub: PairingCommandHub::new() })
    }

    pub fn command_hub(&self) -> PairingCommandHub { self.command_hub.clone() }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", PAIRING_PORT)).await.with_context(|| format!("binding pairing TCP port {}", PAIRING_PORT))?;
        log::info!("PhoneBridge control server listening on :{}", PAIRING_PORT);
        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let acceptor = self.acceptor.clone();
            let trust_store = self.trust_store.clone();
            let identity = self.identity.clone();
            let ui = self.ui.clone();
            let command_hub = self.command_hub.clone();
            tokio::spawn(async move {
                let result = async {
                    let tls_stream = acceptor.accept(stream).await.context("TLS handshake")?;
                    serve_tls_stream(tls_stream, trust_store, identity, ui, command_hub).await
                }.await;
                if let Err(e) = result { log::warn!("connection from {peer_addr} ended with error: {e}"); }
            });
        }
    }
}

/// Common authenticated session entry point for every TLS transport.
pub async fn serve_tls_stream<S>(
    tls_stream: TlsStream<S>,
    trust_store: Arc<Mutex<TrustStore>>,
    identity: Arc<Identity>,
    ui: Arc<dyn UiBackend>,
    command_hub: PairingCommandHub,
) -> Result<()>
where
    S: tokio::io::AsyncRead + AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(tls_stream);
    let mut lines = TokioBufReader::new(reader).lines();
    let hello_line = timeout(HANDSHAKE_READ_TIMEOUT, lines.next_line()).await.context("Hello read timeout")??.context("connection closed before Hello")?;
    let hello = Message::from_line(&hello_line)?;
    let (peer_id, peer_name, peer_platform, peer_protocol, peer_fingerprint) = match hello {
        Message::Hello { device_id, device_name, platform, protocol_version, fingerprint } => (device_id, device_name, platform, protocol_version, fingerprint),
        other => anyhow::bail!("expected Hello as first message, got {:?}", other),
    };
    if peer_protocol != PROTOCOL_VERSION {
        writer.write_all(Message::Error { message: format!("protocol version mismatch: peer={}, pc={}", peer_protocol, PROTOCOL_VERSION) }.to_line()?.as_bytes()).await?;
        anyhow::bail!("unsupported protocol version from {peer_name}: {peer_protocol}");
    }

    let trusted = { trust_store.lock().await.is_trusted(&peer_id, &peer_fingerprint) };
    let mut session = ControlSession::new();
    let outgoing = session.handle_with_peer(
        Message::Hello { device_id: peer_id.clone(), device_name: peer_name.clone(), platform: peer_platform, protocol_version: peer_protocol, fingerprint: peer_fingerprint.clone() },
        trusted,
        Some(&peer_fingerprint),
    ).await?;
    writer.write_all(Message::HelloAck { device_id: identity.device_id.clone(), device_name: hostname(), protocol_version: PROTOCOL_VERSION, trusted, fingerprint: identity.fingerprint_hex() }.to_line()?.as_bytes()).await?;
    for message in outgoing {
        if let Message::PairChallenge { device_id, fingerprint, short_code } = &message { ui.update_pairing_challenge(device_id, fingerprint, short_code).await; }
        writer.write_all(message.to_line()?.as_bytes()).await?;
    }
    if trusted { ui.update_connection_status(true, Some(&peer_name)).await; }

    let mut command_rx = command_hub.register(peer_id.clone()).await;
    let mut deadline = Box::pin(sleep(SESSION_READ_TIMEOUT));

    while !session.is_expired() {
        tokio::select! {
            line = lines.next_line() => {
                let line = match line { Ok(Some(line)) => line, Ok(None) => break, Err(e) => return Err(e.into()) };
                deadline.as_mut().reset(Instant::now() + SESSION_READ_TIMEOUT);
                let message = Message::from_line(&line)?;
                if let Message::Disconnect { reason } = &message { log::debug!("peer {peer_id} requested disconnect: {reason}"); break; }
                process_protocol_message(&mut session, message, &peer_id, &peer_fingerprint, &peer_name, &trust_store, &ui, &mut writer).await?;
            }
            command = command_rx.recv() => {
                let Some(command) = command else { break; };
                deadline.as_mut().reset(Instant::now() + SESSION_READ_TIMEOUT);
                match command {
                    PairingUiCommand::Approve { device_id, short_code } => {
                        if device_id != peer_id { continue; }
                        match session.pairing_mut().approve(&device_id, &short_code) {
                            Ok(response) => {
                                if matches!(&response, Message::PairResult { trusted: true, .. }) {
                                    trust_store.lock().await.trust(&peer_id, &peer_fingerprint)?;
                                    ui.update_connection_status(true, Some(&peer_name)).await;
                                }
                                ui.update_pairing_result(true, "PC approved pairing").await;
                                writer.write_all(response.to_line()?.as_bytes()).await?;
                            }
                            Err(e) => ui.update_pairing_result(false, &e.to_string()).await,
                        }
                    }
                    PairingUiCommand::Reject { device_id, reason } => {
                        if device_id != peer_id { continue; }
                        match session.pairing_mut().reject(&device_id, &reason) {
                            Ok(response) => {
                                ui.update_pairing_result(false, &reason).await;
                                writer.write_all(response.to_line()?.as_bytes()).await?;
                            }
                            Err(e) => ui.update_pairing_result(false, &e.to_string()).await,
                        }
                    }
                    PairingUiCommand::Forget { device_id } => {
                        if device_id != peer_id { continue; }
                        trust_store.lock().await.revoke(&peer_id)?;
                        writer.write_all(Message::Disconnect { reason: "PC trust revoked".into() }.to_line()?.as_bytes()).await?;
                        break;
                    }
                }
            }
            _ = &mut deadline => {
                let _ = writer.write_all(Message::Disconnect { reason: "idle timeout".into() }.to_line()?.as_bytes()).await;
                break;
            }
        }
        if matches!(session.state(), ConnectionState::Connected) { log::debug!("PhoneBridge authenticated session: {peer_id}"); }
    }

    command_hub.unregister(&peer_id).await;
    ui.update_connection_status(false, Some(&peer_name)).await;
    let _ = writer.write_all(Message::Disconnect { reason: "session closed".into() }.to_line()?.as_bytes()).await;
    let _ = writer.shutdown().await;
    Ok(())
}

async fn process_protocol_message<W>(
    session: &mut ControlSession,
    message: Message,
    peer_id: &str,
    peer_fingerprint: &str,
    peer_name: &str,
    trust_store: &Arc<Mutex<TrustStore>>,
    ui: &Arc<dyn UiBackend>,
    writer: &mut W,
) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let is_trusted = { trust_store.lock().await.is_trusted(peer_id, peer_fingerprint) };
    let is_pair_confirm = matches!(&message, Message::PairConfirm { .. });
    match session.handle_with_peer(message, is_trusted, Some(peer_fingerprint)).await {
        Ok(outgoing) => {
            for response in outgoing {
                let should_trust = matches!(&response, Message::PairResult { trusted: true, .. });
                if let Message::PairResult { trusted, message, .. } = &response { ui.update_pairing_result(*trusted, message).await; }
                writer.write_all(response.to_line()?.as_bytes()).await?;
                if should_trust {
                    trust_store.lock().await.trust(peer_id, peer_fingerprint)?;
                    ui.update_connection_status(true, Some(peer_name)).await;
                }
            }
        }
        Err(e) => {
            let response = if is_pair_confirm { Message::PairResult { device_id: peer_id.to_owned(), trusted: false, message: e.to_string() } } else { Message::Error { message: e.to_string() } };
            ui.update_pairing_result(false, &e.to_string()).await;
            writer.write_all(response.to_line()?.as_bytes()).await?;
        }
    }
    Ok(())
}

fn load_cert_chain(pem: &str) -> Result<Vec<CertificateDer<'static>>> { let mut reader = BufReader::new(pem.as_bytes()); rustls_pemfile::certs(&mut reader).collect::<std::result::Result<Vec<_>, _>>().context("parsing certificate chain") }
fn load_private_key(pem: &str) -> Result<PrivateKeyDer<'static>> { let mut reader = BufReader::new(pem.as_bytes()); rustls_pemfile::private_key(&mut reader).context("parsing private key")?.context("no private key found in PEM") }
fn hostname() -> String { std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_else(|_| "phonebridge-pc".to_string()) }
