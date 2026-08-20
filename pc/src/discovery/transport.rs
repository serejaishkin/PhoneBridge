//! Transport advertisement shared by discovery and reconnect logic.
//! Network and Bluetooth are independent transports; authentication always stays
//! on the same TLS/control protocol.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Wifi,
    Hotspot,
    Bluetooth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportEndpoint {
    pub kind: TransportKind,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub address: Option<String>,
}

impl TransportEndpoint {
    pub fn wifi(host: impl Into<String>, port: u16) -> Self { Self { kind: TransportKind::Wifi, host: Some(host.into()), port: Some(port), address: None } }
    pub fn hotspot(host: impl Into<String>, port: u16) -> Self { Self { kind: TransportKind::Hotspot, host: Some(host.into()), port: Some(port), address: None } }
    pub fn bluetooth(address: impl Into<String>) -> Self { Self { kind: TransportKind::Bluetooth, host: None, port: None, address: Some(address.into()) } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportAdvertisement {
    pub transports: Vec<TransportEndpoint>,
}
