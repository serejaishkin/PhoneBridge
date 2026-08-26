//! Desktop UI backend and platform-neutral UI interface.

use crate::protocol::{HfpSupport, MediaPlaybackState};
use async_trait::async_trait;

pub mod desktop;

/// A pending pairing decision surfaced to the user. The user compares the two
/// short codes visually (phone screen vs PC screen) and allows or rejects.
#[derive(Debug, Clone)]
pub struct PairingRequest {
    pub device_id: String,
    pub device_name: String,
    /// Short code derived from the peer certificate fingerprint.
    pub peer_code: String,
    /// Short code of this PC certificate.
    pub local_code: String,
}

#[async_trait]
pub trait UiBackend: Send + Sync {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>);
    async fn notify_call_ended(&self);
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>);
    async fn update_hfp_status(&self, status: HfpSupport);
    async fn update_media_state(
        &self,
        package: Option<&str>,
        state: MediaPlaybackState,
        title: Option<&str>,
        artist: Option<&str>,
        album: Option<&str>,
    );
    async fn notify_sms_received(&self, address: &str, body: &str, timestamp: i64);
    async fn notify_sms_sent(&self, address: &str, body: &str);
    async fn notify_sms_error(&self, error: &str);
    /// Ask the user to accept or reject a pairing attempt. Returns true when allowed.
    async fn request_pairing_decision(&self, request: PairingRequest) -> bool;
}

pub struct HeadlessUi;

#[async_trait]
impl UiBackend for HeadlessUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) { log::info!("[UI] incoming call: {} ({})", caller_name.unwrap_or("unknown"), caller_number.unwrap_or("no number")); }
    async fn notify_call_ended(&self) { log::info!("[UI] call ended"); }
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) { log::info!("[UI] connection status: {} ({})", if connected { "connected" } else { "disconnected" }, peer_name.unwrap_or("-")); }
    async fn update_hfp_status(&self, status: HfpSupport) { log::info!("[UI] HFP support: {:?}", status); }
    async fn update_media_state(&self, package: Option<&str>, state: MediaPlaybackState, title: Option<&str>, artist: Option<&str>, album: Option<&str>) { log::info!("[UI] media: package={:?} state={:?} title={:?} artist={:?} album={:?}", package, state, title, artist, album); }
    async fn notify_sms_received(&self, address: &str, body: &str, timestamp: i64) { log::info!("[UI] SMS received: from={address} timestamp={timestamp}: {body}"); }
    async fn notify_sms_sent(&self, address: &str, body: &str) { log::info!("[UI] SMS sent: to={address}: {body}"); }
    async fn notify_sms_error(&self, error: &str) { log::warn!("[UI] SMS error: {error}"); }
    async fn request_pairing_decision(&self, request: PairingRequest) -> bool {
        // No interactive surface in headless mode; log loudly and auto-accept so
        // demos and tests keep working. Never use HeadlessUi on an untrusted network.
        log::warn!("[UI] pairing request from {} ({}), code {}; auto-accepted by headless UI", request.device_name, request.device_id, request.peer_code);
        true
    }
}
