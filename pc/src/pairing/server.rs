//! TLS control-plane listener: pairing, calls, media and SMS.

use crate::pairing::identity::Identity;
use crate::pairing::trust::{short_code, TrustStore};
use crate::protocol::Message;
use crate::sms::{SmsController, SmsStore};
use crate::ui::UiBackend;
use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
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

impl PairingServer {
    pub fn new(
        identity: Arc<Identity>,
        trust_store: Arc<Mutex<TrustStore>>,
        ui: Arc<dyn UiBackend>,
        sms_controller: SmsController,
        sms_store: Arc<Mutex<SmsStore>>,
    ) -> Result<Self> {
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
            ui,
            sms_controller,
            sms_store,
        })
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", PAIRING_PORT))
            .await
            .with_context(|| format!("binding control TCP port {}", PAIRING_PORT))?;
        log::info!("control server listening on :{}", PAIRING_PORT);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let acceptor = self.acceptor.clone();
            let trust_store = self.trust_store.clone();
            let identity = self.identity.clone();
            let ui = self.ui.clone();
            let sms_controller = self.sms_controller.clone();
            let sms_store = self.sms_store.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(
                    stream,
                    acceptor,
                    trust_store,
                    identity,
                    ui,
                    sms_controller,
                    sms_store,
                )
                .await
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
    ui: Arc<dyn UiBackend>,
    sms_controller: SmsController,
    sms_store: Arc<Mutex<SmsStore>>,
) -> Result<()> {
    let tls_stream = acceptor.accept(stream).await.context("TLS handshake")?;
    let (reader, mut writer) = tokio::io::split(tls_stream);
    let mut lines = TokioBufReader::new(reader).lines();

    let hello_line = lines.next_line().await?.context("connection closed before Hello")?;
    let hello = Message::from_line(&hello_line)?;
    let (peer_id, peer_name) = match &hello {
        Message::Hello { device_id, device_name, .. } => (device_id.clone(), device_name.clone()),
        other => anyhow::bail!("expected Hello as first message, got {other:?}"),
    };

    let trusted = trust_store.lock().await.is_trusted(&peer_id, "");
    if !trusted {
        log::info!(
            "unpaired device connected: {peer_name} ({peer_id}); short_code={}",
            short_code(&identity.fingerprint_hex())
        );
    }

    writer.write_all(
        Message::HelloAck {
            device_id: identity.device_id.clone(),
            device_name: hostname(),
            trusted,
        }.to_line()?.as_bytes()
    ).await?;

    ui.update_connection_status(true, Some(&peer_name)).await;

    // The UI can enqueue PC -> Android messages through this controller.
    let (tx, mut rx) = mpsc::channel::<Message>(64);
    sms_controller.attach(tx).await;

    let writer_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            writer.write_all(message.to_line()?.as_bytes()).await?;
        }
        Ok::<(), anyhow::Error>(())
    });

    // Synchronize recent SMS immediately after connecting.
    let _ = sms_controller.request_history().await;

    while let Some(line) = lines.next_line().await? {
        match Message::from_line(&line)? {
            Message::Ping => {}
            Message::IncomingCall { caller_number, caller_name } => {
                log::info!("incoming call from {:?} ({:?})", caller_name, caller_number);
                ui.notify_incoming_call(caller_name.as_deref(), caller_number.as_deref()).await;
            }
            Message::CallEnded => {
                log::info!("call ended");
                ui.notify_call_ended().await;
            }
            Message::SmsReceived { address, body, timestamp } => {
                log::info!("SMS received from {address}: {body}");
                sms_store.lock().await.add_received(address.clone(), body.clone(), timestamp);
                ui.notify_sms_received(&address, &body, timestamp).await;
            }
            Message::SmsItem { id, address, body, timestamp } => {
                sms_store.lock().await.upsert(crate::sms::SmsMessage { id, address, body, timestamp });
            }
            Message::SmsListEnd { count } => {
                log::info!("Android SMS history synchronized: {count} items");
            }
            Message::SmsSent { address, body } => {
                log::info!("SMS sent to {address}: {body}");
            }
            Message::SmsError { error } => {
                log::warn!("Android SMS error: {error}");
            }
            Message::MediaState { .. }
            | Message::PhoneBluetoothStatus { .. }
            | Message::PcBluetoothStatus { .. }
            | Message::HelloAck { .. }
            | Message::Pong => {}
            Message::Error { message } => log::warn!("Android error: {message}"),
            other => log::debug!("unhandled message: {other:?}"),
        }
    }

    sms_controller.detach().await;
    writer_task.abort();
    ui.update_connection_status(false, Some(&peer_name)).await;
    Ok(())
}

fn load_cert_chain(pem: &str) -> Result<Vec<CertificateDer<'static>>> {
    let mut reader = BufReader::new(pem.as_bytes());
    Ok(rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("parsing certificate chain")?)
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
