//! Windows native RFCOMM transport using the WinRT Bluetooth stack.
//!
//! This module deliberately stops at StreamSocket. The common PhoneBridge
//! byte-stream adapter is kept separate so TLS and pairing remain unchanged.

#![cfg(windows)]

use anyhow::{Context, Result};
use windows::Devices::Bluetooth::Rfcomm::{RfcommDeviceService, RfcommServiceId};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Networking::Sockets::{SocketProtectionLevel, StreamSocket};
use windows::Storage::Streams::{DataReader, InputStreamOptions};

use super::bluetooth::{BluetoothEndpoint, BluetoothSupport, BluetoothTransportKind};

const SERVICE_UUID: RfcommServiceId = RfcommServiceId::SerialPort();

#[derive(Debug, Clone)]
pub struct WindowsBluetoothDevice {
    pub id: String,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsBluetoothTransport;

impl WindowsBluetoothTransport {
    pub fn new() -> Self { Self }

    /// Enumerate paired devices exposing the PhoneBridge RFCOMM service.
    pub async fn discover(&self) -> Result<Vec<WindowsBluetoothDevice>> {
        let selector = RfcommDeviceService::GetDeviceSelector(SERVICE_UUID)?;
        let devices = DeviceInformation::FindAllAsync(&selector)?.await?;
        let mut result = Vec::with_capacity(devices.Size()? as usize);

        for index in 0..devices.Size()? {
            let device = devices.GetAt(index)?;
            let name = device.Name()?.to_string_lossy();
            result.push(WindowsBluetoothDevice {
                id: device.Id()?.to_string_lossy(),
                name,
                address: String::new(),
            });
        }
        Ok(result)
    }

    /// Open a native RFCOMM StreamSocket for a discovered device/service.
    pub async fn connect(&self, device_id: &str) -> Result<StreamSocket> {
        let service = RfcommDeviceService::FromIdAsync(device_id)?.await?
            .context("RFCOMM service is no longer available")?;
        let socket = StreamSocket::new()?;
        socket.ConnectAsync(
            &service.ConnectionHostName()?,
            &service.ConnectionServiceName()?,
            SocketProtectionLevel::BluetoothEncryptionAllowNullAuthentication,
        )?.await?;
        Ok(socket)
    }

    /// Read a small diagnostic sample from the socket without consuming the
    /// PhoneBridge framing layer. This is useful for future transport probing.
    pub async fn read_probe(&self, socket: &StreamSocket) -> Result<Vec<u8>> {
        let reader = DataReader::CreateDataReader(&socket.InputStream()?);
        reader.SetInputStreamOptions(InputStreamOptions::Partial)?;
        let loaded = reader.LoadAsync(1024)?.await?;
        if loaded == 0 { return Ok(Vec::new()); }
        let mut bytes = vec![0u8; loaded as usize];
        reader.ReadBytes(&mut bytes)?;
        Ok(bytes)
    }

    pub fn endpoint(&self, device: &WindowsBluetoothDevice) -> BluetoothEndpoint {
        BluetoothEndpoint {
            address: device.address.clone(),
            service: SERVICE_UUID.AsString().to_string_lossy(),
            channel: None,
            kind: BluetoothTransportKind::Rfcomm,
        }
    }

    pub fn support(&self) -> BluetoothSupport { BluetoothSupport::Supported }
}
