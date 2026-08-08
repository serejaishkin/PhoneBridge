use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, BLECharacteristic, WriteType};
use btleplug::platform::Manager;
use uuid::Uuid;
use std::time::Duration;
use tokio::time;

const PHONEBRIDGE_SERVICE_UUID: Uuid = uuid::uuid!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");
const MANUFACTURER_ID: u16 = 0xFFFF;

pub struct BleDiscovery {
    manager: Manager,
}

#[derive(Debug, Clone)]
pub struct PhoneBridgeDevice {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub rssi: i16,
}

impl BleDiscovery {
    pub async fn new() -> Result<Self, btleplug::Error> {
        let manager = Manager::new().await?;
        Ok(Self { manager })
    }

    pub async fn scan(&self, timeout_secs: u64) -> Result<Vec<PhoneBridgeDevice>, Box<dyn std::error::Error>> {
        let adapters = self.manager.adapters().await?;
        let mut devices = Vec::new();

        for adapter in adapters {
            adapter.start_scan(ScanFilter::default()).await?;
            time::sleep(Duration::from_secs(timeout_secs)).await;
            let peripherals = adapter.peripherals().await?;

            for p in peripherals {
                if let Ok(Some(props)) = p.properties().await {
                    let name = props.local_name.unwrap_or_default();
                    if !name.starts_with("PhoneBridge") {
                        continue;
                    }

                    // Parse manufacturer data for IP + port
                    if let Some((_, data)) = props.manufacturer_data.get(&MANUFACTURER_ID) {
                        if data.len() >= 6 {
                            let ip = format!("{}.{}.{}.{}", data[0], data[1], data[2], data[3]);
                            let port = u16::from_be_bytes([data[4], data[5]]);
                            devices.push(PhoneBridgeDevice {
                                name,
                                ip,
                                port,
                                rssi: props.rssi.unwrap_or(0) as i16,
                            });
                        }
                    }
                }
            }
            adapter.stop_scan().await?;
        }

        Ok(devices)
    }

    pub async fn find_first(&self, timeout_secs: u64) -> Result<Option<PhoneBridgeDevice>, Box<dyn std::error::Error>> {
        let devices = self.scan(timeout_secs).await?;
        Ok(devices.into_iter().next())
    }
}
