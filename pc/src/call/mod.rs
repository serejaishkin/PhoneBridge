//! Call state on the PC side.
//!
//! Reminder: call audio never flows through this app — it goes over native
//! Bluetooth Classic HFP (phone = Audio Gateway, PC = Hands-Free unit), see the
//! root README. This module is control-plane only: who is calling, the
//! Answer/Decline buttons, and diagnostics for "why don't I hear anything".

use crate::protocol::HfpSupport;
#[cfg(target_os = "linux")]
pub mod hfp_linux;
#[cfg(windows)]
pub mod hfp_windows;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallState {
    Idle,
    Ringing {
        caller_number: Option<String>,
        caller_name: Option<String>,
    },
    Active,
}

pub struct SharedState {
    pub call: Mutex<CallState>,
    pub hfp_support: Mutex<HfpSupport>,
}

impl SharedState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            call: Mutex::new(CallState::Idle),
            hfp_support: Mutex::new(HfpSupport::Unknown),
        })
    }
}

/// Check whether this PC's Bluetooth adapter supports the Hands-Free Unit role
/// (HFP client). This determines whether call audio can work at all.
///
/// Platform dispatch:
/// - Windows: real detection via WinRT (see `hfp_windows.rs`).
/// - Linux: real detection via BlueZ D-Bus (see `hfp_linux.rs`).
/// - macOS: not implemented yet — returns Unknown; per AI_HANDOFF_GUI.md 4.1
///   the UI must show this as "needs manual verification", not as an error.
///   TODO: IOBluetooth check (macOS).
#[cfg(windows)]
pub async fn check_hfp_support() -> HfpSupport {
    hfp_windows::detect().await
}

#[cfg(target_os = "linux")]
pub async fn check_hfp_support() -> HfpSupport {
    hfp_linux::detect().await
}

#[cfg(not(any(windows, target_os = "linux")))]
pub async fn check_hfp_support() -> HfpSupport {
    log::warn!(
        "check_hfp_support(): platform detection not implemented on this OS yet, \
         returning Unknown. UI must show this as 'needs manual verification', not as an error."
    );
    HfpSupport::Unknown
}
