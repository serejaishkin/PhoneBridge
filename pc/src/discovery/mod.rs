//! UDP broadcast discovery в локальной сети.
//!
//! Сознательно НЕ используем BLE для discovery в этой версии (в отличие от
//! оригинального PhoneBridge v1) — см. AI_HANDOFF_GUI.md: если телефон и ПК
//! всё равно должны быть в одной Wi-Fi сети для медиа/сигналинга, BLE
//! добавляет сложность (рантайм-разрешения Android 12+, MTU) без реальной
//! выгоды. UDP broadcast на порту ниже работает как только оба устройства
//! в одной подсети — этого достаточно.

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
}

/// Периодически рассылает Announce в broadcast-адрес подсети.
pub async fn run_broadcaster(identity: Arc<Identity>) -> Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).await?;
    socket.set_broadcast(true)?;

    let announce = Announce {
        device_id: identity.device_id.clone(),
        device_name: hostname(),
        platform: current_platform().to_string(),
        pairing_port: crate::pairing::server::PAIRING_PORT,
    };
    let payload = serde_json::to_vec(&announce)?;

    loop {
        // 255.255.255.255 работает в большинстве домашних сетей; если нет —
        // TODO: определять broadcast-адрес конкретного интерфейса вместо global.
        if let Err(e) = socket
            .send_to(&payload, ("255.255.255.255", DISCOVERY_PORT))
            .await
        {
            log::warn!("discovery broadcast failed: {e}");
        }
        tokio::time::sleep(BROADCAST_INTERVAL).await;
    }
}

/// Слушает входящие Announce от телефонов (для будущего сценария "PC ищет телефон",
/// сейчас основной сценарий — телефон ищет PC, но симметричный listener пригодится).
pub async fn run_listener<F>(mut on_announce: F) -> Result<()>
where
    F: FnMut(Announce, std::net::SocketAddr) + Send + 'static,
{
    let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT)).await?;
    let mut buf = [0u8; 1024];
    loop {
        let (n, addr) = socket.recv_from(&mut buf).await?;
        let received: Vec<u8> = buf[..n].to_vec();
        let parsed: Result<Announce, _> = serde_json::from_slice(&received);
        match parsed {
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
