#[cfg(target_os = "windows")]
pub mod tray;
#[cfg(target_os = "windows")]
pub use tray::{TrayUI, AsyncTrayUI, TrayCommand};

#[cfg(not(target_os = "windows"))]
pub mod tray_stub;
#[cfg(not(target_os = "windows"))]
pub use tray_stub::{AsyncTrayUI, TrayCommand};
