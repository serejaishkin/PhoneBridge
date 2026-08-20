//! Platform abstraction for PC integrations.
//!
//! The protocol and connection layers must never call OS-specific APIs directly.
//! Concrete implementations live in target-specific modules.

use crate::protocol::HfpSupport;

pub mod bluetooth;
pub mod bluetooth_native;
pub use bluetooth::{BluetoothEndpoint, BluetoothSupport, BluetoothTransport, UnsupportedBluetooth};

#[cfg(target_os = "windows")]
pub mod windows_bluetooth;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind { Windows, MacOS, Linux, Unknown }

pub trait PlatformBackend: Send + Sync {
    fn kind(&self) -> PlatformKind;
    fn hfp_support(&self) -> HfpSupport;
    fn answer_call(&self) -> anyhow::Result<()>;
    fn decline_call(&self) -> anyhow::Result<()>;
    fn end_call(&self) -> anyhow::Result<()>;
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsBackend as CurrentBackend;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSBackend as CurrentBackend;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxBackend as CurrentBackend;

pub fn current() -> Box<dyn PlatformBackend> { Box::new(CurrentBackend::new()) }
