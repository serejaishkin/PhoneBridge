//! Desktop dashboard view model shared by Windows, macOS and Linux.
//! Rendering stays separate from the connection and pairing layers.

use super::{BasicUi, UiState};
use crate::protocol::HfpSupport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dashboard {
    pub title: String,
    pub connection: String,
    pub peer: String,
    pub hfp: String,
    pub status: String,
    pub pairing_code: Option<String>,
    pub media_enabled: bool,
    pub microphone_enabled: bool,
}

impl Dashboard {
    pub fn from_state(state: &UiState) -> Self {
        Self {
            title: "PhoneBridge".into(),
            connection: if state.connected { "Connected" } else { "Disconnected" }.into(),
            peer: state.peer_name.clone().unwrap_or_else(|| "No phone".into()),
            hfp: match state.hfp_support { HfpSupport::Supported => "Available", HfpSupport::Unsupported => "Unavailable", HfpSupport::Unknown => "Unknown" }.into(),
            status: state.status.clone(),
            pairing_code: state.pairing_code.clone(),
            media_enabled: true,
            microphone_enabled: true,
        }
    }

    pub async fn snapshot(ui: &BasicUi) -> Self {
        let state = ui.state().read().await.clone();
        Self::from_state(&state)
    }
}
