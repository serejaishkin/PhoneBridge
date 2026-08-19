//! macOS HFP backend boundary.
//!
//! Native implementation target: CoreBluetooth/IOBluetooth plus the system
//! telephony/audio routing APIs. The shared controller never imports those
//! frameworks directly.

use super::hfp::HfpBackend;
use crate::protocol::HfpSupport;

#[derive(Debug, Default)]
pub struct MacOsHfpBackend;

impl MacOsHfpBackend {
    pub fn new() -> Self { Self }

    pub fn probe() -> HfpSupport {
        if cfg!(target_os = "macos") { HfpSupport::Unknown } else { HfpSupport::Unsupported }
    }
}

impl HfpBackend for MacOsHfpBackend {
    fn support(&self) -> HfpSupport { Self::probe() }
    fn answer_call(&self) -> Result<(), String> { Err("macOS IOBluetooth HFP command backend is not wired yet".into()) }
    fn decline_call(&self) -> Result<(), String> { Err("macOS IOBluetooth HFP command backend is not wired yet".into()) }
    fn end_call(&self) -> Result<(), String> { Err("macOS IOBluetooth HFP command backend is not wired yet".into()) }
}
