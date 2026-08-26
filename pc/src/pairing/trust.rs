//! Trust-on-first-use storage: device_id -> certificate fingerprint hex.
//!
//! On first connection from an unknown device_id the user sees a human-readable
//! short code (see `short_code`) on both screens; only after an explicit
//! confirmation does the entry land in this file.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrustStore {
    /// device_id -> fingerprint_hex
    peers: HashMap<String, String>,
    #[serde(skip)]
    path: PathBuf,
}

impl TrustStore {
    pub fn load(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir).context("creating trust store dir")?;
        let path = dir.join("trusted_peers.json");
        if !path.exists() {
            return Ok(Self { peers: HashMap::new(), path });
        }
        let raw = fs::read_to_string(&path)?;
        // The file format is a bare id->fingerprint JSON object.
        let peers: HashMap<String, String> =
            serde_json::from_str(&raw).context("parsing trusted_peers.json")?;
        Ok(Self { peers, path })
    }

    fn save(&self) -> Result<()> {
        let raw = serde_json::to_string_pretty(&self.peers)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    pub fn is_trusted(&self, device_id: &str, fingerprint_hex: &str) -> bool {
        self.peers
            .get(device_id)
            .map(|stored| stored.eq_ignore_ascii_case(fingerprint_hex))
            .unwrap_or(false)
    }

    /// Explicitly trust this device_id + fingerprint. Only called after the
    /// user confirmed the short code visually on both devices.
    pub fn trust(&mut self, device_id: &str, fingerprint_hex: &str) -> Result<()> {
        self.peers.insert(device_id.to_string(), fingerprint_hex.to_lowercase());
        self.save()
    }

    pub fn revoke(&mut self, device_id: &str) -> Result<()> {
        if self.peers.remove(device_id).is_some() {
            self.save()?;
        }
        Ok(())
    }
}

/// Human-readable pairing code, e.g. "3F9A-7B21": first 4 bytes of the SHA-256
/// certificate fingerprint as hex, grouped by 4 characters. The Android side
/// computes it with the same algorithm.
pub fn short_code(fingerprint_hex: &str) -> String {
    let upper = fingerprint_hex.to_uppercase();
    let chunk: String = upper.chars().take(8).collect();
    format!("{}-{}", &chunk[0..4], &chunk[4..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_store_round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("pb-trust-{}-{:?}", std::process::id(), std::thread::current().id()));
        let _ = fs::remove_dir_all(&dir);
        let mut store = TrustStore::load(&dir).unwrap();
        assert!(!store.is_trusted("dev", "aa"));
        store.trust("dev", "aabbccdd").unwrap();
        drop(store);

        let reloaded = TrustStore::load(&dir).unwrap();
        assert!(reloaded.is_trusted("dev", "aabbccdd"));
        assert!(reloaded.is_trusted("dev", "AABBCCDD"), "comparison must be case-insensitive");
        assert!(!reloaded.is_trusted("dev", "ffee00"));
        assert!(!reloaded.is_trusted("other", "aabbccdd"));

        let mut reloaded = reloaded;
        reloaded.revoke("dev").unwrap();
        assert!(!TrustStore::load(&dir).unwrap().is_trusted("dev", "aabbccdd"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn short_code_groups_first_four_bytes() {
        assert_eq!(short_code("3f9a7b21ff"), "3F9A-7B21");
    }
}
