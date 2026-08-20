//! Bluetooth transport boundary.
//!
//! The control protocol is TLS and byte-stream based. On platforms where
//! Bluetooth PAN is exposed as a network interface, normal TCP discovery is
//! preferred. Native RFCOMM/L2CAP implementations plug into this boundary
//! without changing pairing or application messages.

use async_trait::async_trait;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothSupport { Supported, Unsupported, Unknown }

#[derive(Debug, Clone)]
pub struct BluetoothEndpoint {
    pub address: String,
    pub service: String,
}

#[async_trait]
pub trait BluetoothTransport: Send + Sync {
    async fn support(&self) -> BluetoothSupport;
    async fn advertise(&self, endpoint: &BluetoothEndpoint) -> Result<()>;
    async fn accept(&self) -> Result<Box<dyn BluetoothStream>>;
}

#[async_trait]
pub trait BluetoothStream: Send + Sync {
    async fn close(&mut self) -> Result<()>;
}

pub struct UnsupportedBluetooth;

#[async_trait]
impl BluetoothTransport for UnsupportedBluetooth {
    async fn support(&self) -> BluetoothSupport { BluetoothSupport::Unknown }
    async fn advertise(&self, _endpoint: &BluetoothEndpoint) -> Result<()> { Ok(()) }
    async fn accept(&self) -> Result<Box<dyn BluetoothStream>> { anyhow::bail!("native Bluetooth RFCOMM transport is not installed for this platform") }
}
