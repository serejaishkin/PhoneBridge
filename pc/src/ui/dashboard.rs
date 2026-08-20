//! Basic dashboard model for the first desktop GUI.
//! A real window/tray frontend can bind directly to this model.

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
        }
    }

    pub async fn snapshot(ui: &BasicUi) -> Self {
        let state = ui.state().read().await.clone();
        Self::from_state(&state)
    }
}
