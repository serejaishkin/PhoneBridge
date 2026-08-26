//! TLS control-plane listener: pairing, calls, media and SMS.
//!
//! Security model (mutual TLS):
//! - The phone MUST present its self-signed certificate during the handshake;
//!   a custom verifier accepts any syntactically valid client certificate so
//!   unknown devices can reach the pairing gate, while rustls still proves
//!   possession of the private key through the CertificateVerify exchange.
//! - After the handshake the SHA-256 fingerprint is taken from the *TLS layer*,
//!   not from the Hello message. The fingerprint claimed inside Hello must
//!   match it, binding the protocol identity to the cryptographically
//!   authenticated channel.
//! - Trusted `(device_id, fingerprint)` pairs skip straight in; anything else
//!   goes through the desktop Allow/Reject dialog and is persisted on Allow.

use crate::pairing::identity::Identity;
use crate::pairing::trust::{short_code, TrustStore};
use crate::protocol::Message;
use crate::sms::{SmsController, SmsStore};
use crate::ui::{PairingRequest, UiBackend};
use anyhow::{anyhow, Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::ServerConfig;
use rustls::{DigitallySignedStruct, DistinguishedName, SignatureScheme};
use rustls::client::danger::HandshakeSignatureValid;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_rustls::TlsAcceptor;

pub const PAIRING_PORT: u16 = 17591;

pub struct PairingServer {
    acceptor: TlsAcceptor,
    trust_store: Arc<Mutex<TrustStore>>,
    identity: Arc<Identity>,
    ui: Arc<dyn UiBackend>,
    sms_controller: SmsController,
    sms_store: Arc<Mutex<SmsStore>>,
}

/// Accepts every client certificate presented during the handshake. This does
/// NOT mean "trust everyone": rustls still verifies proof-of-possession of the
/// private key, and authorization happens afterwards via TrustStore / pairing.
#[derive(Debug)]
struct PermissiveClientCertVerifier;

impl ClientCertVerifier for PermissiveClientCertVerifier {
    fn verify_client_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        Ok(ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }
}

impl PairingServer {
    pub fn new(identity: Arc<Identity>, trust_store: Arc<Mutex<TrustStore>>, ui: Arc<dyn UiBackend>, sms_controller: SmsController, sms_store: Arc<Mutex<SmsStore>>) -> Result<Self> {
        let cert_der = load_cert_chain(&identity.cert_pem)?;
        let key_der = load_private_key(&identity.key_pem)?;
        let verifier = Arc::new(PermissiveClientCertVerifier);
        let config = ServerConfig::builder()
            .with_client_cert_verifier(verifier)
            .with_single_cert(cert_der, key_der)
            .context("building TLS server config")?;
        Ok(Self { acceptor: TlsAcceptor::from(Arc::new(config)), trust_store, identity, ui, sms_controller, sms_store })
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", PAIRING_PORT)).await.with_context(|| format!("binding control TCP port {}", PAIRING_PORT))?;
        log::info!("control server listening on :{}", PAIRING_PORT);
        self.run_with_listener(listener).await
    }

    /// Runs the accept loop on an existing listener; split out so tests can
    /// bind an ephemeral port instead of the fixed production one.
    pub async fn run_with_listener(self, listener: TcpListener) -> Result<()> {
        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let acceptor = self.acceptor.clone();
            let trust_store = self.trust_store.clone();
            let identity = self.identity.clone();
            let ui = self.ui.clone();
            let sms_controller = self.sms_controller.clone();
            let sms_store = self.sms_store.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, acceptor, trust_store, identity, ui, sms_controller, sms_store).await {
                    log::warn!("connection from {peer_addr} ended with error: {e}");
                }
            });
        }
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, acceptor: TlsAcceptor, trust_store: Arc<Mutex<TrustStore>>, identity: Arc<Identity>, ui: Arc<dyn UiBackend>, sms_controller: SmsController, sms_store: Arc<Mutex<SmsStore>>) -> Result<()> {
    let tls_stream = acceptor.accept(stream).await.context("TLS handshake")?;

    // Authoritative peer fingerprint: taken from the negotiated TLS session,
    // not from any message the client could have forged.
    let (_, session) = tls_stream.get_ref();
    let tls_fingerprint = session
        .peer_certificates()
        .and_then(|certs| certs.first())
        .map(|cert| sha256_hex(cert.as_ref()))
        .ok_or_else(|| anyhow!("client completed handshake without a certificate"))?;

    let (reader, mut writer) = tokio::io::split(tls_stream);
    let mut lines = TokioBufReader::new(reader).lines();

    let hello_line = lines.next_line().await?.context("connection closed before Hello")?;
    let hello = Message::from_line(&hello_line)?;
    let (peer_id, peer_name, claimed_fingerprint) = match &hello {
        Message::Hello { device_id, device_name, cert_fingerprint, .. } =>
            (device_id.clone(), device_name.clone(), cert_fingerprint.clone()),
        other => anyhow::bail!("expected Hello as first message, got {other:?}"),
    };

    // The claimed fingerprint must match the certificate actually used in the
    // handshake; otherwise something is impersonating another device id.
    match &claimed_fingerprint {
        Some(claimed) if claimed.eq_ignore_ascii_case(&tls_fingerprint) => {}
        Some(_) => {
            let reason = "Hello fingerprint does not match the TLS client certificate".to_string();
            log::warn!("rejecting {peer_name}: {reason}");
            writer.write_all(Message::Error { message: reason }.to_line()?.as_bytes()).await?;
            return Ok(());
        }
        None => {
            let reason = "client did not provide a certificate fingerprint".to_string();
            log::warn!("rejecting {peer_name}: {reason}");
            writer.write_all(Message::Error { message: reason }.to_line()?.as_bytes()).await?;
            return Ok(());
        }
    }
    let peer_fingerprint = tls_fingerprint;

    let mut trusted = trust_store.lock().await.is_trusted(&peer_id, &peer_fingerprint);

    if !trusted {
        let request = PairingRequest {
            device_id: peer_id.clone(),
            device_name: peer_name.clone(),
            peer_code: short_code(&peer_fingerprint),
            local_code: short_code(&identity.fingerprint_hex()),
        };
        log::info!("pairing requested by {} ({}); code {}", peer_name, peer_id, request.peer_code);
        if ui.request_pairing_decision(request).await {
            trust_store.lock().await.trust(&peer_id, &peer_fingerprint).context("persisting trusted peer")?;
            log::info!("pairing accepted for {peer_name} ({peer_id})");
            trusted = true;
        } else {
            log::info!("pairing rejected for {peer_name} ({peer_id})");
            let ack = Message::HelloAck { device_id: identity.device_id.clone(), device_name: hostname(), trusted: false, cert_fingerprint: identity.fingerprint_hex() };
            writer.write_all(ack.to_line()?.as_bytes()).await?;
            return Ok(());
        }
    }

    let ack = Message::HelloAck { device_id: identity.device_id.clone(), device_name: hostname(), trusted, cert_fingerprint: identity.fingerprint_hex() };
    writer.write_all(ack.to_line()?.as_bytes()).await?;
    ui.update_connection_status(true, Some(&peer_name)).await;

    let (tx, mut rx) = mpsc::channel::<Message>(64);
    sms_controller.attach(tx).await;
    let writer_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await { writer.write_all(message.to_line()?.as_bytes()).await?; }
        Ok::<(), anyhow::Error>(())
    });

    let _ = sms_controller.request_history().await;

    while let Some(line) = lines.next_line().await? {
        match Message::from_line(&line)? {
            Message::Ping => {}
            Message::IncomingCall { caller_number, caller_name } => {
                log::info!("incoming call from {:?} ({:?})", caller_name, caller_number);
                ui.notify_incoming_call(caller_name.as_deref(), caller_number.as_deref()).await;
            }
            Message::CallEnded => { log::info!("call ended"); ui.notify_call_ended().await; }
            Message::SmsReceived { address, body, timestamp } => {
                log::info!("SMS received from {address}: {body}");
                sms_store.lock().await.add_received(address.clone(), body.clone(), timestamp);
                ui.notify_sms_received(&address, &body, timestamp).await;
            }
            Message::SmsItem { id, address, body, timestamp } => {
                sms_store.lock().await.upsert(crate::sms::SmsMessage { id, address, body, timestamp });
            }
            Message::SmsListEnd { count } => { log::info!("Android SMS history synchronized: {count} items"); }
            Message::SmsSent { address, body } => {
                log::info!("SMS sent to {address}: {body}");
                ui.notify_sms_sent(&address, &body).await;
            }
            Message::SmsError { error } => {
                log::warn!("Android SMS error: {error}");
                ui.notify_sms_error(&error).await;
            }
            Message::MediaState { package, state, title, artist, album } => {
                ui.update_media_state(package.as_deref(), state, title.as_deref(), artist.as_deref(), album.as_deref()).await;
            }
            Message::PhoneBluetoothStatus { .. }
            | Message::PcBluetoothStatus { .. }
            | Message::HelloAck { .. }
            | Message::Pong => {}
            Message::Error { message } => { log::warn!("Android error: {message}"); ui.notify_sms_error(&message).await; }
            other => log::debug!("unhandled message: {other:?}"),
        }
    }

    sms_controller.detach().await;
    writer_task.abort();
    ui.update_connection_status(false, Some(&peer_name)).await;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn load_cert_chain(pem: &str) -> Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(pem.as_bytes());
    Ok(rustls_pemfile::certs(&mut reader).collect::<std::result::Result<Vec<_>, _>>().context("parsing certificate chain")?)
}

fn load_private_key(pem: &str) -> Result<PrivateKeyDer<'static>> {
    let mut reader = BufReader::new(pem.as_bytes());
    rustls_pemfile::private_key(&mut reader).context("parsing private key")?.context("no private key found in PEM")
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_else(|_| "phonebridge-pc".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::HfpSupport;
    use crate::ui::HeadlessUi;
    use rustls::client::danger::ServerCertVerifier;
    use tokio::io::AsyncBufReadExt;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        // Unique per call: parallel tests share one process.
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("pb-pairing-{name}-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    async fn spawn_server() -> (u16, std::path::PathBuf) {
        let dir = temp_dir("server");
        let identity = Arc::new(Identity::load_or_create(&dir).unwrap());
        let trust = Arc::new(Mutex::new(TrustStore::load(&dir).unwrap()));
        let ui = Arc::new(HeadlessUi);
        let server = PairingServer::new(identity, trust, ui, SmsController::new(), Arc::new(Mutex::new(SmsStore::new()))).unwrap();
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(server.run_with_listener(listener));
        (port, dir)
    }

    /// Client-side verifier that trusts any self-signed server certificate:
    /// acceptable in tests where the server identity was just generated locally.
    #[derive(Debug)]
    struct AcceptAnyServer;

    impl ServerCertVerifier for AcceptAnyServer {
        fn verify_server_cert(
            &self,
            _end_entity: &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name: &rustls::pki_types::ServerName<'_>,
            _ocsp_response: &[u8],
            _now: UnixTime,
        ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
            rustls::crypto::verify_tls12_signature(message, cert, dss, &rustls::crypto::ring::default_provider().signature_verification_algorithms)
        }

        fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
            rustls::crypto::verify_tls13_signature(message, cert, dss, &rustls::crypto::ring::default_provider().signature_verification_algorithms)
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            rustls::crypto::ring::default_provider().signature_verification_algorithms.supported_schemes()
        }
    }

    fn client_connector(client_dir: &std::path::Path) -> tokio_rustls::TlsConnector {
        let identity = Identity::load_or_create(client_dir).unwrap();
        let certs = load_cert_chain(&identity.cert_pem).unwrap();
        let key = load_private_key(&identity.key_pem).unwrap();
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAnyServer))
            .with_client_auth_cert(certs, key)
            .unwrap();
        tokio_rustls::TlsConnector::from(Arc::new(config))
    }

    /// Connector without a client certificate: must be rejected by the server.
    fn certless_connector() -> tokio_rustls::TlsConnector {
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAnyServer))
            .with_no_client_auth();
        tokio_rustls::TlsConnector::from(Arc::new(config))
    }

    async fn say_hello_and_expect_trusted(port: u16, connector: &tokio_rustls::TlsConnector, client_dir: &std::path::Path, expected_trusted: bool) {
        let client_identity = Identity::load_or_create(client_dir).unwrap();
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let name = rustls::pki_types::ServerName::IpAddress(stream.local_addr().unwrap().ip().into());
        let mut tls = connector.connect(name, stream).await.expect("TLS handshake");

        let hello = Message::Hello {
            device_id: client_identity.device_id.clone(),
            device_name: "TestPhone".into(),
            platform: "test".into(),
            protocol_version: 1,
            cert_fingerprint: Some(client_identity.fingerprint_hex()),
        };
        tls.write_all(hello.to_line().unwrap().as_bytes()).await.unwrap();

        let mut reader = Box::pin(TokioBufReader::new(tls));
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("HelloAck line");
        let line = line.trim_end();
        match Message::from_line(&line).unwrap() {
            Message::HelloAck { trusted, .. } => assert_eq!(trusted, expected_trusted),
            other => panic!("expected HelloAck, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn pairing_gate_accepts_then_remembers_device() {
        let (port, server_dir) = spawn_server().await;
        let client_dir = temp_dir("client");
        let connector = client_connector(&client_dir);

        // First connection goes through the (auto-accepting headless) pairing
        // gate and must persist the peer in the server's TrustStore.
        say_hello_and_expect_trusted(port, &connector, &client_dir, true).await;

        let client_identity = Identity::load_or_create(&client_dir).unwrap();
        let reloaded = TrustStore::load(&server_dir).unwrap();
        let stored_file = std::fs::read_to_string(server_dir.join("trusted_peers.json"));
        assert!(
            reloaded.is_trusted(&client_identity.device_id, &client_identity.fingerprint_hex()),
            "pairing decision must be persisted; want=({} {}) stored={stored_file:?}",
            client_identity.device_id,
            client_identity.fingerprint_hex()
        );

        // Second connection is recognized without another pairing round.
        say_hello_and_expect_trusted(port, &connector, &client_dir, true).await;
    }

    #[tokio::test]
    async fn rejects_clients_without_certificate() {
        use tokio::io::AsyncReadExt;
        let (port, _server_dir) = spawn_server().await;
        let connector = certless_connector();
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let name = rustls::pki_types::ServerName::IpAddress(stream.local_addr().unwrap().ip().into());
        // Under TLS 1.3 the client-side handshake can complete before the
        // server's certificate_required alert arrives, so rejection is only
        // observable on the first exchange.
        if let Ok(mut tls) = connector.connect(name, stream).await {
            let hello = Message::Hello { device_id: "certless".into(), device_name: "NoCert".into(), platform: "test".into(), protocol_version: 1, cert_fingerprint: None };
            let _ = tls.write_all(hello.to_line().unwrap().as_bytes()).await;
            let mut buf = [0u8; 64];
            match tls.read(&mut buf).await {
                // Connection aborted (error) or closed without data (Ok(0)):
                // both prove the server refused to talk to a certless peer.
                Err(_) => {}
                Ok(0) => {}
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    assert!(
                        !text.contains("HelloAck"),
                        "certless session must never receive HelloAck, got: {text}"
                    );
                }
            }
        }
        // If even the handshake failed, that satisfies the requirement too.
    }
}
