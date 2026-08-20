//! Native Bluetooth transport boundary.
//!
//! The core only knows how to consume a byte stream. OS backends are responsible
//! for discovering a paired phone and returning a connected stream.

use anyhow::{bail, Result};
use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothTransportKind {
    Rfcomm,
    L2cap,
}

#[derive(Debug, Clone)]
pub struct BluetoothPeer {
    pub device_id: String,
    pub address: String,
    pub channel: Option<u8>,
    pub kind: BluetoothTransportKind,
}

#[async_trait]
pub trait BluetoothTransport: Send + Sync {
    async fn discover(&self) -> Result<Vec<BluetoothPeer>>;
    async fn connect(&self, peer: &BluetoothPeer) -> Result<Box<dyn tokio::io::AsyncReadWrite + Unpin + Send>>;
}

pub struct UnsupportedBluetoothTransport;

#[async_trait]
impl BluetoothTransport for UnsupportedBluetoothTransport {
    async fn discover(&self) -> Result<Vec<BluetoothPeer>> {
        Ok(Vec::new())
    }

    async fn connect(&self, _peer: &BluetoothPeer) -> Result<Box<dyn tokio::io::AsyncReadWrite + Unpin + Send>> {
        bail!("native Bluetooth stream backend is not installed for this platform")
    }
}
