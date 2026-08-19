//! Call control-plane state on the PC.

pub mod controller;
pub mod hfp;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;

use crate::protocol::HfpSupport;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallState {
    Idle,
    Ringing { caller_number: Option<String>, caller_name: Option<String> },
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

pub async fn check_hfp_support() -> HfpSupport {
    hfp::create_backend().support()
}
