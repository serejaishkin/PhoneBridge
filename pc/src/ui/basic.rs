//! Minimal native-feeling desktop UI shared by Windows/macOS/Linux.
//! The UI state is deliberately independent from any concrete window toolkit.

use crate::protocol::HfpSupport;
use super::UiBackend;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

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

#[derive(Debug, Clone)]
pub enum UiCommand {
    ApprovePairing { device_id: String, short_code: String },
    RejectPairing { device_id: String, reason: String },
    ForgetPeer { device_id: String },
}

#[derive(Clone)]
pub struct BasicUi {
    state: Arc<RwLock<UiState>>,
    commands: mpsc::UnboundedSender<UiCommand>,
}

impl BasicUi {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<UiCommand>) {
        let (commands, receiver) = mpsc::unbounded_channel();
        (Self { state: Arc::new(RwLock::new(UiState::default())), commands }, receiver)
    }

    pub fn state(&self) -> Arc<RwLock<UiState>> { self.state.clone() }

    /// Queue a pairing approval for the live ControlSession owner.
    pub fn approve_pairing(&self, device_id: impl Into<String>, short_code: impl Into<String>) {
        let _ = self.commands.send(UiCommand::ApprovePairing { device_id: device_id.into(), short_code: short_code.into() });
    }

    /// Queue a pairing rejection for the live ControlSession owner.
    pub fn reject_pairing(&self, device_id: impl Into<String>, reason: impl Into<String>) {
        let _ = self.commands.send(UiCommand::RejectPairing { device_id: device_id.into(), reason: reason.into() });
    }

    /// Queue trust revocation without coupling the GUI to the TrustStore implementation.
    pub fn forget_peer(&self, device_id: impl Into<String>) {
        let _ = self.commands.send(UiCommand::ForgetPeer { device_id: device_id.into() });
    }

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
        self.state.write().await.pairing_code = None;
    }
}

#[async_trait]
impl UiBackend for BasicUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) {
        self.state.write().await.status = format!("Incoming call: {} ({})", caller_name.unwrap_or("Unknown"), caller_number.unwrap_or("No number"));
    }
    async fn notify_call_ended(&self) { self.state.write().await.status = "Call ended".into(); }
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) {
        let mut s = self.state.write().await;
        s.connected = connected;
        s.peer_name = peer_name.map(str::to_owned);
        s.status = if connected { "Connected" } else { "Disconnected" }.into();
        if connected { s.pairing_code = None; }
    }
    async fn update_hfp_status(&self, status: HfpSupport) { self.state.write().await.hfp_support = status; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn pairing_command_is_queued_for_session_owner() {
        let (ui, mut rx) = BasicUi::new();
        ui.approve_pairing("phone", "123456");
        assert!(matches!(rx.recv().await, Some(UiCommand::ApprovePairing { device_id, short_code }) if device_id == "phone" && short_code == "123456"));
    }
}
