//! Windows HFP backend boundary.
//!
//! This module intentionally keeps Windows-specific implementation details out
//! of the shared call controller. The first implementation uses the Windows
//! Bluetooth/Telephony capability layer when available and reports an explicit
//! unsupported state otherwise.

use super::hfp::HfpBackend;
use crate::protocol::HfpSupport;

#[derive(Debug, Default)]
pub struct WindowsHfpBackend;

impl WindowsHfpBackend {
    pub fn new() -> Self { Self }

    /// Capability probe is kept separate from call commands so the UI can
    /// distinguish "Bluetooth unavailable" from a failed call command.
    pub fn probe() -> HfpSupport {
        if cfg!(target_os = "windows") {
            // Native Windows implementation is isolated here. Until the
            // Windows Runtime bindings are enabled, don't claim support.
            HfpSupport::Unknown
        } else {
            HfpSupport::Unsupported
        }
    }
}

impl HfpBackend for WindowsHfpBackend {
    fn support(&self) -> HfpSupport { Self::probe() }

    fn answer_call(&self) -> Result<(), String> {
        Err("Windows HFP command backend is not wired to Windows Runtime yet".into())
    }

    fn decline_call(&self) -> Result<(), String> {
        Err("Windows HFP command backend is not wired to Windows Runtime yet".into())
    }

    fn end_call(&self) -> Result<(), String> {
        Err("Windows HFP command backend is not wired to Windows Runtime yet".into())
    }
}
