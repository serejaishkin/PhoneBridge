//! Windows RFCOMM incoming-session bridge.
//!
//! This module intentionally contains only transport glue. TLS, pairing and
//! trust decisions remain in the shared connection/session layer.

#![cfg(windows)]

use crate::connection::tls;
use crate::pairing::identity::Identity;
use crate::pairing::server::serve_tls_stream;
use crate::pairing::trust::TrustStore;
use crate::pairing::ui_commands::PairingCommandHub;
use crate::ui::UiBackend;
use super::windows_bluetooth::WindowsBluetoothTransport;
use super::windows_stream::WindowsSocketStream;
use anyhow::{Context, Result};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const BLUETOOTH_TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);

/// Run the PhoneBridge authenticated session for every incoming RFCOMM socket.
pub async fn serve_windows_bluetooth(
    transport: &WindowsBluetoothTransport,
    trust_store: Arc<Mutex<TrustStore>>,
    identity: Arc<Identity>,
    ui: Arc<dyn UiBackend>,
    command_hub: PairingCommandHub,
) -> Result<()> {
    let listener = transport
        .listen()
        .await
        .context("starting Windows RFCOMM listener")?;
    let runtime = tokio::runtime::Handle::current();

    let certificate_der = identity.cert_der.clone();
    let private_key_der = parse_private_key_der(&identity.key_pem)?;
    let acceptor = tls::server_acceptor(certificate_der, private_key_der)?;

    transport.on_connection_async(&listener, runtime, move |socket| {
        let trust_store = trust_store.clone();
        let identity = identity.clone();
        let ui = ui.clone();
        let command_hub = command_hub.clone();
        let acceptor = acceptor.clone();
        async move {
            let stream = WindowsSocketStream::from_socket(socket)
                .await
                .context("creating Windows Bluetooth byte stream")?;

            // The Bluetooth stream enters exactly the same TLS/session path as TCP.
            // Bluetooth discovery or OS pairing never grants PhoneBridge trust.
            let tls_stream = tls::accept(
                &acceptor,
                stream,
                BLUETOOTH_TLS_HANDSHAKE_TIMEOUT,
            )
            .await
            .context("Bluetooth TLS handshake")?;

            serve_tls_stream(tls_stream, trust_store, identity, ui, command_hub).await
        }
    })?;

    log::info!(
        "PhoneBridge Windows RFCOMM listener started: {}",
        listener.service_id()
    );

    // Keep the advertising/listener object alive for the lifetime of the service.
    std::future::pending::<()>().await;
    let _ = listener.stop();
    Ok(())
}

fn parse_private_key_der(pem: &str) -> Result<Vec<u8>> {
    let mut reader = Cursor::new(pem.as_bytes());
    let key = rustls_pemfile::private_key(&mut reader)
        .context("parsing Windows Bluetooth TLS private key")?
        .context("Windows Bluetooth TLS key is missing")?;
    Ok(key.secret_der().to_vec())
}
