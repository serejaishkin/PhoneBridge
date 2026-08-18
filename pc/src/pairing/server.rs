//! TLS-listener для control-plane соединений (пейринг + Hello/IncomingCall/...).
//!
//! Важно: это НЕ аудио-канал. Аудио (Opus/UDP) — отдельный, гораздо более
//! чувствительный к задержке путь, здесь не рассматривается (см. README.md
//! для полной картины протоколов).

use crate::pairing::identity::Identity;
use crate::pairing::trust::{short_code, TrustStore};
use crate::protocol::Message;
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
        log::info!("pairing server listening on :{}", PAIRING_PORT);

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
                if let Err(e) =
                    handle_connection(stream, acceptor, trust_store, identity).await
                {
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

    let hello_line = lines
        .next_line()
        .await?
        .context("connection closed before Hello")?;
    let hello = Message::from_line(&hello_line)?;

    let (peer_id, peer_name) = match &hello {
        Message::Hello {
            device_id,
            device_name,
            ..
        } => (device_id.clone(), device_name.clone()),
        other => {
            anyhow::bail!("expected Hello as first message, got {:?}", other);
        }
    };

    // NOTE: на этом этапе у нас пока нет TLS client-auth (сертификат клиента не
    // запрашивается), поэтому "fingerprint пира" сверяется на прикладном уровне
    // отдельным сообщением, а не через rustls peer certificates. Это сознательное
    // упрощение MVP — TODO для Kimi: перейти на with_client_cert_verifier() и
    // сверять реальный TLS-сертификат клиента, а не JSON-поле.
    let trusted = {
        let store = trust_store.lock().await;
        // fingerprint клиента в этой заготовке приходит отдельным полем в будущем
        // сообщении PairRequest — здесь для MVP считаем untrusted, пока не добавлено.
        store.is_trusted(&peer_id, "")
    };

    if !trusted {
        log::info!(
            "unpaired device connected: {peer_name} ({peer_id}) — pairing confirmation required. \
             short_code placeholder: {}",
            short_code(&identity.fingerprint_hex())
        );
        // TODO(Kimi/GUI): вместо автоответа показать в UI код подтверждения и
        // дождаться нажатия "Confirm" пользователем, прежде чем trust_store.trust(...).
    }

    let ack = Message::HelloAck {
        device_id: identity.device_id.clone(),
        device_name: hostname(),
        trusted,
    };
    writer.write_all(ack.to_line()?.as_bytes()).await?;

    while let Some(line) = lines.next_line().await? {
        let msg = Message::from_line(&line)?;
        match msg {
            Message::Ping => {
                writer.write_all(Message::Pong.to_line()?.as_bytes()).await?;
            }
            Message::IncomingCall {
                caller_number,
                caller_name,
            } => {
                log::info!(
                    "incoming call from {:?} ({:?})",
                    caller_name,
                    caller_number
                );
                // TODO(Kimi/GUI): прокинуть в UiBackend::notify_incoming_call(...)
            }
            Message::CallEnded => {
                log::info!("call ended");
            }
            other => {
                log::debug!("unhandled message: {:?}", other);
            }
        }
    }

    Ok(())
}

fn load_cert_chain(pem: &str) -> Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(pem.as_bytes());
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("parsing certificate chain")?;
    Ok(certs)
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
