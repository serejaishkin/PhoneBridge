//! Platform-neutral UI contract. Platform GUI code must depend on this layer.

use crate::protocol::HfpSupport;
use async_trait::async_trait;

mod basic;
mod native;
pub use basic::{BasicUi, UiState};

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
        Self {
            screen: UiScreen::Dashboard,
            connected: false,
            peer_name: None,
            peer_address: None,
            hfp: HfpSupport::Unknown,
            media_enabled: true,
            microphone_enabled: true,
            pairing_code: None,
            diagnostic_lines: Vec::new(),
        }
    }
}

#[async_trait]
pub trait UiBackend: Send + Sync {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>);
    async fn notify_call_ended(&self);
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>);
    async fn update_hfp_status(&self, status: HfpSupport);
}

pub struct HeadlessUi;

#[async_trait]
impl UiBackend for HeadlessUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) { log::info!("[UI] incoming call: {} ({})", caller_name.unwrap_or("unknown"), caller_number.unwrap_or("no number")); }
    async fn notify_call_ended(&self) { log::info!("[UI] call ended"); }
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) { log::info!("[UI] connection status: {} ({})", if connected { "connected" } else { "disconnected" }, peer_name.unwrap_or("-")); }
    async fn update_hfp_status(&self, status: HfpSupport) { log::info!("[UI] HFP support: {:?}", status); }
}
