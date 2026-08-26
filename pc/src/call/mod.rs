//! Call state on the PC side.
//!
//! Reminder: call audio never flows through this app — it goes over native
//! Bluetooth Classic HFP (phone = Audio Gateway, PC = Hands-Free unit), see the
//! root README. This module is control-plane only: who is calling, the
//! Answer/Decline buttons, and diagnostics for "why don't I hear anything".

use crate::protocol::HfpSupport;
#[cfg(target_os = "linux")]
pub mod hfp_linux;
#[cfg(target_os = "macos")]
pub mod hfp_macos;
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
/// Platform dispatch — all three desktop platforms have real detection now:
/// - Windows: WinRT RFCOMM service cache (see `hfp_windows.rs`).
/// - Linux: BlueZ D-Bus adapter UUIDs (see `hfp_linux.rs`).
/// - macOS: IOBluetooth default controller (see `hfp_macos.rs`).
#[cfg(windows)]
pub async fn check_hfp_support() -> HfpSupport {
    hfp_windows::detect().await
}

#[cfg(target_os = "linux")]
pub async fn check_hfp_support() -> HfpSupport {
    hfp_linux::detect().await
}

#[cfg(target_os = "macos")]
pub async fn check_hfp_support() -> HfpSupport {
    hfp_macos::detect().await
}
