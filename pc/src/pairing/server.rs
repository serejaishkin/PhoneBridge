//! TLS control-plane server.
//!
//! The server owns transport acceptance and trust lookup; protocol state is
//! delegated to `connection::session::ControlSession`.

use crate::connection::session::ControlSession;
use crate::connection::state::ConnectionState;
use crate::pairing::identity::Identity;
use crate::pairing::trust::TrustStore;
use crate::protocol::{Message, PROTOCOL_VERSION};
use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_rustls::TlsAcceptor;

pub const PAIRING_PORT: u16 = 17591;

pub struct PairingServer {
    acceptor: TlsAcceptor,
    trust_store: Arc<Mutex<TrustStore>>,
    identity: Arc<Identity>,
}

impl PairingServer {
    pub fn new(identity: Arc<Identity>, trust_store: Arc<Mutex<TrustStore>>) -> Result<Self> {
        let cert_der = load_cert_chain(&identity.cert_pem)?;
        let key_der = load_private_key(&identity.key_pem)?;
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_der, key_der)
            .context("building TLS server config")?;
        Ok(Self {
            acceptor: TlsAcceptor::from(Arc::new(config)),
            trust_store,
            identity,
        })
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", PAIRING_PORT))
            .await
            .with_context(|| format!("binding pairing TCP port {}", PAIRING_PORT))?;
        log::info!("PhoneBridge control server listening on :{}", PAIRING_PORT);

        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("accept() failed: {e}");
                    continue;
                }
            };
            let acceptor = self.acceptor.clone();
            let trust_store = self.trust_store.clone();
            let identity = self.identity.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, acceptor, trust_store, identity).await {
                    log::warn!("connection from {peer_addr} ended with error: {e}");
                }
            });
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    acceptor: TlsAcceptor,
    trust_store: Arc<Mutex<TrustStore>>,
    identity: Arc<Identity>,
) -> Result<()> {
    let tls_stream = acceptor.accept(stream).await.context("TLS handshake")?;
    let (reader, mut writer) = tokio::io::split(tls_stream);
    let mut lines = TokioBufReader::new(reader).lines();

    let hello_line = lines.next_line().await?.context("connection closed before Hello")?;
    let hello = Message::from_line(&hello_line)?;
    let (peer_id, peer_name, peer_platform, peer_protocol, peer_fingerprint) = match hello {
        Message::Hello { device_id, device_name, platform, protocol_version, fingerprint } =>
            (device_id, device_name, platform, protocol_version, fingerprint),
        other => anyhow::bail!("expected Hello as first message, got {:?}", other),
    };

    if peer_protocol != PROTOCOL_VERSION {
        writer.write_all(Message::Error {
            message: format!("protocol version mismatch: peer={}, pc={}", peer_protocol, PROTOCOL_VERSION),
        }.to_line()?.as_bytes()).await?;
        anyhow::bail!("unsupported protocol version from {peer_name}: {peer_protocol}");
    }

    log::info!("PhoneBridge peer connected: {} ({}, {}) fingerprint={}", peer_name, peer_id, peer_platform, peer_fingerprint);

    let trusted = {
        let store = trust_store.lock().await;
        store.is_trusted(&peer_id, &peer_fingerprint)
    };

    let mut session = ControlSession::new();
    let outgoing = session.handle(
        Message::Hello {
            device_id: peer_id.clone(),
            device_name: peer_name.clone(),
            platform: peer_platform,
            protocol_version: peer_protocol,
            fingerprint: peer_fingerprint.clone(),
        },
        trusted,
    )?;

    writer.write_all(Message::HelloAck {
        device_id: identity.device_id.clone(),
        device_name: hostname(),
        protocol_version: PROTOCOL_VERSION,
        trusted,
        fingerprint: identity.fingerprint_hex(),
    }.to_line()?.as_bytes()).await?;

    for message in outgoing {
        writer.write_all(message.to_line()?.as_bytes()).await?;
    }

    while let Some(line) = lines.next_line().await? {
        let message = Message::from_line(&line)?;
        let is_trusted = {
            let store = trust_store.lock().await;
            store.is_trusted(&peer_id, &peer_fingerprint)
        };

        match message {
            Message::PairConfirm { device_id, short_code } => {
                match session.handle(Message::PairConfirm { device_id: device_id.clone(), short_code }, is_trusted) {
                    Ok(outgoing) => {
                        for response in outgoing {
                            let should_trust = matches!(&response, Message::PairResult { trusted: true, .. });
                            writer.write_all(response.to_line()?.as_bytes()).await?;
                            if should_trust {
                                let mut store = trust_store.lock().await;
                                store.trust(&peer_id, &peer_fingerprint)?;
                            }
                        }
                    }
                    Err(e) => {
                        writer.write_all(Message::PairResult {
                            device_id,
                            trusted: false,
                            message: e.to_string(),
                        }.to_line()?.as_bytes()).await?;
                    }
                }
            }
            Message::Ping | Message::Pong => {
                for response in session.handle(message, is_trusted)? {
                    writer.write_all(response.to_line()?.as_bytes()).await?;
                }
            }
            other => {
                for response in session.handle(other, is_trusted)? {
                    writer.write_all(response.to_line()?.as_bytes()).await?;
                }
            }
        }

        if matches!(session.state(), ConnectionState::Connected) {
            log::debug!("PhoneBridge control session authenticated for {peer_id}");
        }
    }

    Ok(())
}

fn load_cert_chain(pem: &str) -> Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(pem.as_bytes());
    rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("parsing certificate chain")
}

fn load_private_key(pem: &str) -> Result<PrivateKeyDer<'static>> {
    let mut reader = BufReader::new(pem.as_bytes());
    rustls_pemfile::private_key(&mut reader)
        .context("parsing private key")?
        .context("no private key found in PEM")
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "phonebridge-pc".to_string())
}
