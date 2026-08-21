//! Windows native RFCOMM transport using the WinRT Bluetooth stack.
//!
//! This module owns the incoming/outgoing RFCOMM listener and deliberately
//! exposes only StreamSocket at the platform boundary. TLS and pairing stay
//! transport-independent above this layer.

#![cfg(windows)]

use anyhow::{Context, Result};
use windows::Devices::Bluetooth::Rfcomm::{RfcommDeviceService, RfcommServiceId, RfcommServiceProvider};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Networking::Sockets::{SocketProtectionLevel, StreamSocket, StreamSocketListener, StreamSocketListenerConnectionReceivedEventArgs};
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

pub struct WindowsBluetoothListener {
    provider: RfcommServiceProvider,
    listener: StreamSocketListener,
}

impl WindowsBluetoothTransport {
    pub fn new() -> Self { Self }

    /// Enumerate paired devices exposing the PhoneBridge RFCOMM service.
    pub async fn discover(&self) -> Result<Vec<WindowsBluetoothDevice>> {
        let selector = RfcommDeviceService::GetDeviceSelector(SERVICE_UUID)?;
        let devices = DeviceInformation::FindAllAsync(&selector)?.await?;
        let mut result = Vec::with_capacity(devices.Size()? as usize);
        for index in 0..devices.Size()? {
            let device = devices.GetAt(index)?;
            result.push(WindowsBluetoothDevice {
                id: device.Id()?.to_string_lossy(),
                name: device.Name()?.to_string_lossy(),
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

    /// Start advertising the PhoneBridge RFCOMM service and return the listener.
    /// Windows exposes incoming RFCOMM connections through StreamSocketListener.
    pub async fn listen(&self) -> Result<WindowsBluetoothListener> {
        let provider = RfcommServiceProvider::CreateAsync(SERVICE_UUID)?.await?;
        let listener = StreamSocketListener::new()?;
        listener.BindServiceNameAsyncWithProtectionLevel(
            &provider.ServiceId()?.AsString()?,
            SocketProtectionLevel::BluetoothEncryptionAllowNullAuthentication,
        )?.await?;
        // Keep radio discoverability explicit. Pairing/trust is still performed by
        // PhoneBridge over TLS; Bluetooth discoverability must never imply trust.
        provider.StartAdvertisingWithRadioDiscoverability(&listener, true)?;
        Ok(WindowsBluetoothListener { provider, listener })
    }

    /// Install a synchronous callback for callers that only need the native socket.
    pub fn on_connection<F>(&self, listener: &WindowsBluetoothListener, callback: F) -> Result<()>
    where
        F: Fn(StreamSocket) + Send + Sync + 'static,
    {
        let callback = std::sync::Arc::new(callback);
        listener.listener.ConnectionReceived(&windows::Foundation::TypedEventHandler::new(
            move |_sender: Option<&StreamSocketListener>, args: Option<&StreamSocketListenerConnectionReceivedEventArgs>| {
                if let Some(args) = args {
                    if let Ok(socket) = args.Socket() { callback(socket); }
                }
                Ok(())
            },
        ))?;
        Ok(())
    }

    /// Install an asynchronous callback and spawn each accepted socket on the
    /// supplied Tokio runtime. This is the handoff point to TLS + ControlSession.
    pub fn on_connection_async<F, Fut>(
        &self,
        listener: &WindowsBluetoothListener,
        runtime: tokio::runtime::Handle,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(StreamSocket) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let callback = std::sync::Arc::new(callback);
        listener.listener.ConnectionReceived(&windows::Foundation::TypedEventHandler::new(
            move |_sender: Option<&StreamSocketListener>, args: Option<&StreamSocketListenerConnectionReceivedEventArgs>| {
                if let Some(args) = args {
                    if let Ok(socket) = args.Socket() {
                        let callback = callback.clone();
                        runtime.spawn(async move {
                            if let Err(error) = callback(socket).await {
                                log::warn!("Windows RFCOMM incoming session failed: {}", error);
                            }
                        });
                    }
                }
                Ok(())
            },
        ))?;
        Ok(())
    }

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

impl WindowsBluetoothListener {
    pub fn service_id(&self) -> String {
        self.provider.ServiceId().map(|id| id.AsString().to_string_lossy()).unwrap_or_default()
    }

    pub fn stop(&self) -> Result<()> {
        self.provider.StopAdvertising()?;
        self.listener.Close();
        Ok(())
    }
}
