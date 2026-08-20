//! Minimal native-feeling desktop UI shared by Windows/macOS/Linux.
//! The UI is intentionally small: connection, HFP capability and pairing code.

use crate::protocol::HfpSupport;
use super::UiBackend;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub connected: bool,
    pub peer_name: Option<String>,
    pub hfp_support: HfpSupport,
    pub pairing_code: Option<String>,
    pub status: String,
}

#[derive(Clone, Default)]
pub struct BasicUi {
    state: Arc<RwLock<UiState>>,
}

impl BasicUi {
    pub fn new() -> Self { Self::default() }
    pub fn state(&self) -> Arc<RwLock<UiState>> { self.state.clone() }
}

#[async_trait]
impl UiBackend for BasicUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) {
        let mut s = self.state.write().await;
        s.status = format!("Incoming call: {} ({})", caller_name.unwrap_or("Unknown"), caller_number.unwrap_or("No number"));
    }
    async fn notify_call_ended(&self) {
        self.state.write().await.status = "Call ended".into();
    }
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) {
        let mut s = self.state.write().await;
        s.connected = connected;
        s.peer_name = peer_name.map(str::to_owned);
        s.status = if connected { "Connected" } else { "Disconnected" }.into();
    }
    async fn update_hfp_status(&self, status: HfpSupport) {
        self.state.write().await.hfp_support = status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn state_updates_without_platform_gui() {
        let ui = BasicUi::new();
        ui.update_connection_status(true, Some("Phone")).await;
        ui.update_hfp_status(HfpSupport::Supported).await;
        let s = ui.state().read().await.clone();
        assert!(s.connected);
        assert_eq!(s.peer_name.as_deref(), Some("Phone"));
        assert_eq!(s.hfp_support, HfpSupport::Supported);
    }
}
