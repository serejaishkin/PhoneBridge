//! Windows RFCOMM incoming-session bridge.
//!
//! This module intentionally contains only transport glue. TLS, pairing and
//! trust decisions remain in the shared connection/session layer.

#![cfg(windows)]

use crate::pairing::identity::Identity;
use crate::pairing::server::serve_tls_stream;
use crate::pairing::trust::TrustStore;
use crate::pairing::ui_commands::PairingCommandHub;
use crate::ui::UiBackend;
use super::windows_bluetooth::WindowsBluetoothTransport;
use super::windows_stream::WindowsSocketStream;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Run the PhoneBridge authenticated session for every incoming RFCOMM socket.
pub async fn serve_windows_bluetooth(
    transport: &WindowsBluetoothTransport,
    trust_store: Arc<Mutex<TrustStore>>,
    identity: Arc<Identity>,
    ui: Arc<dyn UiBackend>,
    command_hub: PairingCommandHub,
) -> Result<()> {
    let listener = transport.listen().await.context("starting Windows RFCOMM listener")?;
    let runtime = tokio::runtime::Handle::current();
    let transport_ref = transport.clone();
    transport.on_connection_async(&listener, runtime, move |socket| {
        let trust_store = trust_store.clone();
        let identity = identity.clone();
        let ui = ui.clone();
        let command_hub = command_hub.clone();
        let transport = transport_ref;
        async move {
            let stream = WindowsSocketStream::from_socket(socket).await.context("creating Windows Bluetooth byte stream")?;
            // The Bluetooth stream enters exactly the same TLS/session path as TCP.
            // Bluetooth discovery or OS pairing never grants PhoneBridge trust.
            let tls_acceptor = crate::connection::tls::server_acceptor(&identity)?;
            let tls_stream = tls_acceptor.accept(stream).await.context("Bluetooth TLS handshake")?;
            serve_tls_stream(tls_stream, trust_store, identity, ui, command_hub).await
        }
    })?;
    log::info!("PhoneBridge Windows RFCOMM listener started: {}", listener.service_id());
    // Keep the advertising/listener object alive for the lifetime of the service.
    // The actual connection work is spawned by on_connection_async().
    std::future::pending::<()>().await;
    let _ = listener.stop();
    let _ = transport;
    Ok(())
}
