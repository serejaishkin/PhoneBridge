//! Platform-neutral UI contract. Platform GUI code must depend on this layer.

use crate::protocol::HfpSupport;
use async_trait::async_trait;
use tokio::sync::mpsc;

mod basic;
mod dashboard;
mod desktop;
mod native;
pub use basic::{BasicUi, UiState};
pub use dashboard::Dashboard;
pub use desktop::DesktopApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiScreen { Dashboard, Pairing, Settings, Diagnostics }

#[derive(Debug, Clone)]
pub struct DesktopUiState {
    pub screen: UiScreen,
    pub connected: bool,
    pub peer_name: Option<String>,
    pub peer_address: Option<String>,
    pub hfp: HfpSupport,
    pub media_enabled: bool,
    pub microphone_enabled: bool,
    pub pairing_code: Option<String>,
    pub diagnostic_lines: Vec<String>,
}

impl Default for DesktopUiState {
    fn default() -> Self {
        Self { screen: UiScreen::Dashboard, connected: false, peer_name: None, peer_address: None, hfp: HfpSupport::Unknown, media_enabled: true, microphone_enabled: true, pairing_code: None, diagnostic_lines: Vec::new() }
    }
}

/// Commands emitted by the desktop UI and delivered to the live pairing session.
#[derive(Debug, Clone)]
pub enum PairingUiCommand {
    Approve { device_id: String, short_code: String },
    Reject { device_id: String, reason: String },
    Forget { device_id: String },
}

pub type PairingUiCommandSender = mpsc::Sender<PairingUiCommand>;
pub type PairingUiCommandReceiver = mpsc::Receiver<PairingUiCommand>;

pub fn pairing_command_channel() -> (PairingUiCommandSender, PairingUiCommandReceiver) { mpsc::channel(16) }

#[async_trait]
pub trait UiBackend: Send + Sync {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>);
    async fn notify_call_ended(&self);
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>);
    async fn update_hfp_status(&self, status: HfpSupport);
    async fn update_pairing_challenge(&self, device_id: &str, fingerprint: &str, short_code: &str);
    async fn update_pairing_result(&self, trusted: bool, message: &str);
}

pub struct HeadlessUi;

#[async_trait]
impl UiBackend for HeadlessUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) { log::info!("[UI] incoming call: {} ({})", caller_name.unwrap_or("unknown"), caller_number.unwrap_or("no number")); }
    async fn notify_call_ended(&self) { log::info!("[UI] call ended"); }
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) { log::info!("[UI] connection status: {} ({})", if connected { "connected" } else { "disconnected" }, peer_name.unwrap_or("-")); }
    async fn update_hfp_status(&self, status: HfpSupport) { log::info!("[UI] HFP support: {:?}", status); }
    async fn update_pairing_challenge(&self, device_id: &str, _fingerprint: &str, short_code: &str) { log::info!("[UI] pairing request from {device_id}; confirmation code={short_code}"); }
    async fn update_pairing_result(&self, trusted: bool, message: &str) { log::info!("[UI] pairing result trusted={trusted}: {message}"); }
}
