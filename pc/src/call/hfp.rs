//! Platform-neutral HFP capability abstraction.

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
use super::linux::LinuxHfpBackend;
#[cfg(target_os = "macos")]
use super::macos::MacOsHfpBackend;

pub fn create_backend() -> Box<dyn HfpBackend> {
    #[cfg(target_os = "windows")]
    { return Box::new(WindowsHfpBackend::new()); }
    #[cfg(target_os = "linux")]
    { return Box::new(LinuxHfpBackend::new()); }
    #[cfg(target_os = "macos")]
    { return Box::new(MacOsHfpBackend::new()); }
    Box::new(UnsupportedHfpBackend)
}
