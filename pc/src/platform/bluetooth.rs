//! Bluetooth transport boundary.
//!
//! Bluetooth PAN remains an IP route and uses the normal TCP/TLS path. Native
//! RFCOMM/L2CAP implementations plug into this byte-stream boundary.

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothSupport { Supported, Unsupported, Unknown }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothTransportKind { Rfcomm, L2cap }

#[derive(Debug, Clone)]
pub struct BluetoothEndpoint {
    pub address: String,
    pub service: String,
    pub channel: Option<u8>,
    pub kind: BluetoothTransportKind,
}

pub trait BluetoothByteStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T> BluetoothByteStream for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

#[async_trait]
pub trait BluetoothTransport: Send + Sync {
    async fn support(&self) -> BluetoothSupport;
    async fn discover(&self) -> Result<Vec<BluetoothEndpoint>>;
    async fn advertise(&self, endpoint: &BluetoothEndpoint) -> Result<()>;
    async fn connect(&self, endpoint: &BluetoothEndpoint) -> Result<Box<dyn BluetoothByteStream>>;
    async fn accept(&self) -> Result<Box<dyn BluetoothByteStream>>;
}

pub struct UnsupportedBluetooth;

#[async_trait]
impl BluetoothTransport for UnsupportedBluetooth {
    async fn support(&self) -> BluetoothSupport { BluetoothSupport::Unknown }
    async fn discover(&self) -> Result<Vec<BluetoothEndpoint>> { Ok(Vec::new()) }
    async fn advertise(&self, _endpoint: &BluetoothEndpoint) -> Result<()> { Ok(()) }
    async fn connect(&self, _endpoint: &BluetoothEndpoint) -> Result<Box<dyn BluetoothByteStream>> {
        anyhow::bail!("native Bluetooth stream backend is not installed for this platform")
    }
    async fn accept(&self) -> Result<Box<dyn BluetoothByteStream>> {
        anyhow::bail!("native Bluetooth stream backend is not installed for this platform")
    }
}
