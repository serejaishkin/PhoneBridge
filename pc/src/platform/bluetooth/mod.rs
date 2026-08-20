#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothEndpoint {
    pub address: String,
    pub service_id: String,
    pub name: Option<String>,
}

pub trait BluetoothTransport: Send + Sync {
    fn supported(&self) -> bool;
}

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub use windows::WindowsBluetoothTransport;
