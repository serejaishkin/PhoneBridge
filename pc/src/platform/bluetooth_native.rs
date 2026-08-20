//! OS-specific Bluetooth backend entry points.
//!
//! The native backends intentionally expose a common byte-stream contract. The
//! TLS/control layers must not know whether the underlying link is RFCOMM or L2CAP.

use super::bluetooth::{BluetoothEndpoint, BluetoothSupport, BluetoothTransport};
use anyhow::{bail, Result};
use async_trait::async_trait;

#[cfg(target_os = "windows")]
pub struct WindowsBluetooth;
#[cfg(target_os = "linux")]
pub struct LinuxBluetooth;
#[cfg(target_os = "macos")]
pub struct MacOsBluetooth;

pub fn current() -> Box<dyn BluetoothTransport> {
    #[cfg(target_os = "windows")]
    { return Box::new(WindowsBluetooth); }
    #[cfg(target_os = "linux")]
    { return Box::new(LinuxBluetooth); }
    #[cfg(target_os = "macos")]
    { return Box::new(MacOsBluetooth); }
    #[allow(unreachable_code)]
    Box::new(super::bluetooth::UnsupportedBluetooth)
}

macro_rules! native_backend {
    ($name:ident, $platform:literal) => {
        #[async_trait]
        impl BluetoothTransport for $name {
            async fn support(&self) -> BluetoothSupport { BluetoothSupport::Unknown }

            async fn discover(&self) -> Result<Vec<BluetoothEndpoint>> {
                // Discovery is implemented by the platform adapter, not by the core.
                log::debug!("Bluetooth discovery requested on {}", $platform);
                Ok(Vec::new())
            }

            async fn advertise(&self, _endpoint: &BluetoothEndpoint) -> Result<()> {
                bail!("native Bluetooth advertising backend is not implemented on {}", $platform)
            }

            async fn connect(&self, _endpoint: &BluetoothEndpoint) -> Result<Box<dyn super::bluetooth::BluetoothByteStream>> {
                bail!("native Bluetooth stream backend is not implemented on {}", $platform)
            }

            async fn accept(&self) -> Result<Box<dyn super::bluetooth::BluetoothByteStream>> {
                bail!("native Bluetooth accept backend is not implemented on {}", $platform)
            }
        }
    };
}

#[cfg(target_os = "windows")]
native_backend!(WindowsBluetooth, "Windows");
#[cfg(target_os = "linux")]
native_backend!(LinuxBluetooth, "Linux/BlueZ");
#[cfg(target_os = "macos")]
native_backend!(MacOsBluetooth, "macOS/IOBluetooth");
