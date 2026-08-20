//! Minimal native-feeling desktop UI shared by Windows/macOS/Linux.
//! The UI state is deliberately independent from any concrete window toolkit.

use crate::protocol::HfpSupport;
use super::UiBackend;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub connected: bool,
    pub peer_name: Option<String>,
    pub peer_device_id: Option<String>,
    pub peer_fingerprint: Option<String>,
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

    /// Update the pending pairing challenge shown by the desktop frontend.
    pub async fn show_pairing_challenge(&self, device_id: &str, fingerprint: &str, code: &str) {
        let mut s = self.state.write().await;
        s.peer_device_id = Some(device_id.to_owned());
        s.peer_fingerprint = Some(fingerprint.to_owned());
        s.pairing_code = Some(code.to_owned());
        s.status = "Pairing confirmation required".into();
    }

    /// Clear pairing UI after successful trust or a rejected/closed session.
    pub async fn clear_pairing(&self) {
        let mut s = self.state.write().await;
        s.pairing_code = None;
    }
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
        if connected { s.pairing_code = None; }
    }
    async fn update_hfp_status(&self, status: HfpSupport) {
        self.state.write().await.hfp_support = status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn pairing_challenge_is_exposed_to_ui_state() {
        let ui = BasicUi::new();
        ui.show_pairing_challenge("phone", "fp", "123456").await;
        let s = ui.state().read().await.clone();
        assert_eq!(s.pairing_code.as_deref(), Some("123456"));
        assert_eq!(s.peer_device_id.as_deref(), Some("phone"));
    }
}
