//! UDP broadcast discovery в локальной сети.

use crate::pairing::identity::Identity;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;

pub const DISCOVERY_PORT: u16 = 17592;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announce {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
    pub pairing_port: u16,
    /// SHA-256 fingerprint of the PC persistent identity certificate.
    pub fingerprint: String,
}

pub async fn run_broadcaster(identity: Arc<Identity>) -> Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).await?;
    socket.set_broadcast(true)?;

    let announce = Announce {
        device_id: identity.device_id.clone(),
        device_name: hostname(),
        platform: current_platform().to_string(),
        pairing_port: crate::pairing::server::PAIRING_PORT,
        fingerprint: identity.fingerprint_hex(),
    };
    let payload = serde_json::to_vec(&announce)?;

    loop {
        if let Err(e) = socket
            .send_to(&payload, ("255.255.255.255", DISCOVERY_PORT))
            .await
        {
            log::warn!("discovery broadcast failed: {e}");
        }
        tokio::time::sleep(BROADCAST_INTERVAL).await;
    }
}

pub async fn run_listener<F>(mut on_announce: F) -> Result<()>
where
    F: FnMut(Announce, std::net::SocketAddr) + Send + 'static,
{
    let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT)).await?;
    let mut buf = [0u8; 1024];
    loop {
        let (n, addr) = socket.recv_from(&mut buf).await?;
        match serde_json::from_slice::<Announce>(&buf[..n]) {
            Ok(announce) => on_announce(announce, addr),
            Err(e) => log::debug!("bad discovery packet from {addr}: {e}"),
        }
    }
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "phonebridge-pc".to_string())
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}
