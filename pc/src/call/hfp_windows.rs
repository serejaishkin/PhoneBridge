//! Windows implementation of HFP client-role support detection.
//!
//! There is no public WinRT API that directly answers "does this PC's Bluetooth
//! stack support the Hands-Free (client) role". The practical, evidence-based
//! check (consistent with NEXT_STEPS.md) is:
//!
//! 1. No default Bluetooth adapter, or BR/EDR ("Classic") not supported
//!    -> Unsupported: call audio via HFP is impossible.
//! 2. Some paired device already exposes an RFCOMM Hands-Free service through
//!    this machine's stack -> Supported (the HF endpoint demonstrably exists).
//! 3. Adapter present but nothing proven yet -> Unknown: pair a phone first;
//!    UI must present this as "needs manual verification", not as an error.

use crate::protocol::HfpSupport;
use windows::Devices::Bluetooth::Rfcomm::RfcommServiceId;
use windows::Devices::Bluetooth::{BluetoothAdapter, BluetoothCacheMode, BluetoothDevice};
use windows::Devices::Enumeration::DeviceInformation;
use windows::core::GUID;

/// Standard Bluetooth Hands-Free service UUID (111E).
const HANDS_FREE_SERVICE_UUID: u128 = 0x0000111e_0000_1000_8000_00805f9b34fb;

pub async fn detect() -> HfpSupport {
    // WinRT completion handlers block; keep them off the Tokio worker threads.
    match tokio::task::spawn_blocking(detect_blocking).await {
        Ok(support) => support,
        Err(e) => {
            log::warn!("HFP detection task panicked: {e}");
            HfpSupport::Unknown
        }
    }
}

fn detect_blocking() -> HfpSupport {
    // No usable Bluetooth radio means call audio cannot work at all.
    let adapter = match BluetoothAdapter::GetDefaultAsync().and_then(|op| op.join()) {
        Ok(adapter) => adapter,
        Err(e) => {
            log::info!("no default Bluetooth adapter ({}); HFP unsupported", e);
            return HfpSupport::Unsupported;
        }
    };
    match detect_with_adapter(&adapter) {
        Ok(support) => {
            log::info!("Windows HFP support detection result: {support:?}");
            support
        }
        Err(e) => {
            log::warn!("Windows HFP detection failed, reporting Unknown: {e}");
            HfpSupport::Unknown
        }
    }
}

fn detect_with_adapter(adapter: &BluetoothAdapter) -> windows::core::Result<HfpSupport> {
    if !adapter.IsClassicSupported()? {
        return Ok(HfpSupport::Unsupported);
    }

    // Enumerate every paired device, then read each device's cached RFCOMM
    // service list. A Hands-Free entry proves this Windows stack instantiated
    // the HF endpoint during pairing.
    let selector = BluetoothDevice::GetDeviceSelectorFromPairingState(true)?;
    let paired = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.join()?;
    let hands_free = RfcommServiceId::FromUuid(GUID::from_u128(HANDS_FREE_SERVICE_UUID))?;

    let total = paired.Size()?;
    for i in 0..total {
        let info = paired.GetAt(i)?;
        let id = info.Id()?;
        let Ok(device) = BluetoothDevice::FromIdAsync(&id).and_then(|op| op.join()) else {
            continue; // LE-only or non-classic entry
        };
        let Ok(services) = device.GetRfcommServicesWithCacheModeAsync(BluetoothCacheMode::Cached).and_then(|op| op.join()) else {
            continue;
        };
        let list = services.Services()?;
        for s in 0..list.Size()? {
            let service = list.GetAt(s)?;
            if service.ServiceId()? == hands_free {
                return Ok(HfpSupport::Supported);
            }
        }
    }

    Ok(HfpSupport::Unknown)
}
