use super::{BluetoothEndpoint, BluetoothTransport};
use anyhow::{anyhow, Result};
use windows::Devices::Bluetooth::Rfcomm::{RfcommDeviceService, RfcommServiceId};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Networking::Sockets::StreamSocket;

/// Windows Bluetooth RFCOMM transport using the WinRT Bluetooth stack.
/// The resulting StreamSocket carries bytes; PhoneBridge TLS remains above this layer.
pub struct WindowsBluetoothTransport;

impl WindowsBluetoothTransport {
    pub fn new() -> Self { Self }

    pub fn service_selector() -> windows::core::Result<windows::core::HSTRING> {
        RfcommDeviceService::GetDeviceSelector(&RfcommServiceId::SerialPort())
    }

    pub async fn discover(&self) -> Result<Vec<BluetoothEndpoint>> {
        let selector = Self::service_selector()?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.await?;
        let mut result = Vec::with_capacity(devices.Size()? as usize);
        for index in 0..devices.Size()? {
            let device = devices.GetAt(index)?;
            let id = device.Id()?.to_string_lossy();
            let name = device.Name()?.to_string_lossy();
            result.push(BluetoothEndpoint {
                address: id.clone(),
                service_id: RfcommServiceId::SerialPort()?.to_string_lossy(),
                name: if name.is_empty() { None } else { Some(name) },
            });
        }
        Ok(result)
    }

    /// Open an RFCOMM StreamSocket to a discovered Bluetooth service.
    pub async fn connect(&self, endpoint: &BluetoothEndpoint) -> Result<StreamSocket> {
        let service = RfcommDeviceService::FromIdAsync(&windows::core::HSTRING::from(&endpoint.address))?.await?;
        let socket = StreamSocket::new()?;
        socket.ConnectAsync(&service.ConnectionHostName()?, &service.ConnectionServiceName()?)?.await?;
        Ok(socket)
    }

    pub fn ensure_endpoint(endpoint: &BluetoothEndpoint) -> Result<()> {
        if endpoint.address.trim().is_empty() { return Err(anyhow!("Bluetooth endpoint has no device id")); }
        if endpoint.service_id.trim().is_empty() { return Err(anyhow!("Bluetooth endpoint has no service id")); }
        Ok(())
    }
}

impl Default for WindowsBluetoothTransport {
    fn default() -> Self { Self::new() }
}

impl BluetoothTransport for WindowsBluetoothTransport {
    fn supported(&self) -> bool { true }
}
