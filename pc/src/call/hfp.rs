//! Platform-neutral HFP capability abstraction.
//!
//! The control plane uses this trait only to discover whether the host can
//! expose an HFP Hands-Free endpoint. Actual Bluetooth audio remains native
//! to the operating system.

use crate::protocol::HfpSupport;

pub trait HfpBackend: Send + Sync {
    fn support(&self) -> HfpSupport;
    fn answer_call(&self) -> Result<(), String>;
    fn decline_call(&self) -> Result<(), String>;
    fn end_call(&self) -> Result<(), String>;
}

#[derive(Debug, Default)]
pub struct UnsupportedHfpBackend;

impl HfpBackend for UnsupportedHfpBackend {
    fn support(&self) -> HfpSupport { HfpSupport::Unsupported }
    fn answer_call(&self) -> Result<(), String> { Err("HFP backend is not available on this platform/build".into()) }
    fn decline_call(&self) -> Result<(), String> { Err("HFP backend is not available on this platform/build".into()) }
    fn end_call(&self) -> Result<(), String> { Err("HFP backend is not available on this platform/build".into()) }
}

#[cfg(target_os = "windows")]
use super::windows::WindowsHfpBackend;

#[cfg(target_os = "linux")]
pub mod platform {
    use super::*;
    pub struct LinuxHfpBackend;
    impl HfpBackend for LinuxHfpBackend {
        fn support(&self) -> HfpSupport { HfpSupport::Unknown }
        fn answer_call(&self) -> Result<(), String> { Err("Linux BlueZ HFP integration pending".into()) }
        fn decline_call(&self) -> Result<(), String> { Err("Linux BlueZ HFP integration pending".into()) }
        fn end_call(&self) -> Result<(), String> { Err("Linux BlueZ HFP integration pending".into()) }
    }
}

#[cfg(target_os = "macos")]
pub mod platform {
    use super::*;
    pub struct MacOsHfpBackend;
    impl HfpBackend for MacOsHfpBackend {
        fn support(&self) -> HfpSupport { HfpSupport::Unknown }
        fn answer_call(&self) -> Result<(), String> { Err("macOS IOBluetooth HFP integration pending".into()) }
        fn decline_call(&self) -> Result<(), String> { Err("macOS IOBluetooth HFP integration pending".into()) }
        fn end_call(&self) -> Result<(), String> { Err("macOS IOBluetooth HFP integration pending".into()) }
    }
}

pub fn create_backend() -> Box<dyn HfpBackend> {
    #[cfg(target_os = "windows")]
    { return Box::new(WindowsHfpBackend::new()); }
    #[cfg(target_os = "linux")]
    { return Box::new(platform::LinuxHfpBackend); }
    #[cfg(target_os = "macos")]
    { return Box::new(platform::MacOsHfpBackend); }
    Box::new(UnsupportedHfpBackend)
}
