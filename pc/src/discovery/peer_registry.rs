use super::Announce;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

const PEER_TTL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct DiscoveredPc {
    pub announce: Announce,
    pub addr: SocketAddr,
    pub last_seen: Instant,
}

#[derive(Debug, Default)]
pub struct PeerRegistry {
    peers: HashMap<String, DiscoveredPc>,
}

impl PeerRegistry {
    pub fn upsert(&mut self, announce: Announce, addr: SocketAddr) {
        self.peers.insert(announce.device_id.clone(), DiscoveredPc { announce, addr, last_seen: Instant::now() });
    }
    pub fn get(&self, device_id: &str) -> Option<&DiscoveredPc> { self.peers.get(device_id) }
    pub fn prune(&mut self) { self.peers.retain(|_, peer| peer.last_seen.elapsed() <= PEER_TTL); }
    pub fn all(&self) -> impl Iterator<Item = &DiscoveredPc> { self.peers.values() }
}
