//! Linux HFP backend boundary.
//!
//! The implementation target is BlueZ over D-Bus. Keep all D-Bus object
//! paths, profiles and audio routing details in this module so the core call
//! controller remains platform-neutral.

use super::hfp::HfpBackend;
use crate::protocol::HfpSupport;

#[derive(Debug, Default)]
pub struct LinuxHfpBackend;

impl LinuxHfpBackend {
    pub fn new() -> Self { Self }

    pub fn probe() -> HfpSupport {
        if cfg!(target_os = "linux") { HfpSupport::Unknown } else { HfpSupport::Unsupported }
    }
}

impl HfpBackend for LinuxHfpBackend {
    fn support(&self) -> HfpSupport { Self::probe() }
    fn answer_call(&self) -> Result<(), String> { Err("Linux BlueZ HFP command backend is not wired yet".into()) }
    fn decline_call(&self) -> Result<(), String> { Err("Linux BlueZ HFP command backend is not wired yet".into()) }
    fn end_call(&self) -> Result<(), String> { Err("Linux BlueZ HFP command backend is not wired yet".into()) }
}
