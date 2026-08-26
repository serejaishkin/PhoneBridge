//! macOS implementation of HFP client-role support detection.
//!
//! Unlike generic PC Bluetooth dongles, Apple ships every Mac with a built-in
//! controller whose stack implements the Hands-Free profile out of the box.
//! Hardware variance (the reason Windows/Linux need evidence-based checks) does
//! not exist here, so the decision table is simple:
//!
//! 1. `IOBluetoothHostController.defaultController` returns nil -> Unsupported:
//!    no Bluetooth radio, call audio cannot work.
//! 2. Controller present -> Supported.
//!
//! This keeps parity with hfp_windows.rs / hfp_linux.rs semantics without a
//! pointless SDP walk over paired devices.

use crate::protocol::HfpSupport;
use objc2_io_bluetooth::IOBluetoothHostController;

pub async fn detect() -> HfpSupport {
    // ObjC runtime calls block briefly; keep them off the Tokio workers.
    match tokio::task::spawn_blocking(detect_blocking).await {
        Ok(support) => support,
        Err(e) => {
            log::warn!("HFP detection task panicked: {e}");
            HfpSupport::Unknown
        }
    }
}

fn detect_blocking() -> HfpSupport {
    let support = unsafe {
        match IOBluetoothHostController::defaultController() {
            Some(_) => HfpSupport::Supported,
            None => HfpSupport::Unsupported,
        }
    };
    log::info!("macOS HFP support detection result: {support:?}");
    support
}
