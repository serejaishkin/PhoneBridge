//! Trust-on-first-use хранилище: device_id -> fingerprint сертификата, которому
//! мы доверяем. При первом подключении неизвестного device_id пользователю
//! показывается human-readable код (short_code) для сверки со вторым устройством
//! (аналог safety number / номера сопряжения по Bluetooth), и только после
//! явного подтверждения запись попадает в этот файл.

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
            return Ok(Self {
                peers: HashMap::new(),
                path,
            });
        }
        let raw = fs::read_to_string(&path)?;
        let mut store: Self = serde_json::from_str(&raw).unwrap_or_default();
        store.path = path;
        Ok(store)
    }

    fn save(&self) -> Result<()> {
        let raw = serde_json::to_string_pretty(&self.peers)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    pub fn is_trusted(&self, device_id: &str, fingerprint_hex: &str) -> bool {
        self.peers
            .get(device_id)
            .map(|fp| fp == fingerprint_hex)
            .unwrap_or(false)
    }

    /// Явно доверять этому device_id+fingerprint. Вызывается только после того,
    /// как пользователь подтвердил short_code глазами на обоих устройствах.
    pub fn trust(&mut self, device_id: &str, fingerprint_hex: &str) -> Result<()> {
        self.peers
            .insert(device_id.to_string(), fingerprint_hex.to_string());
        self.save()
    }

    pub fn revoke(&mut self, device_id: &str) -> Result<()> {
        self.peers.remove(device_id);
        self.save()
    }
}

/// Человекочитаемый код для сверки при пейринге, например "3F9A-7B21".
/// Android-сторона должна вычислять его по тому же алгоритму (первые 4 байта
/// SHA-256 отпечатка сертификата, hex, сгруппированные по 4 символа).
pub fn short_code(fingerprint_hex: &str) -> String {
    let upper = fingerprint_hex.to_uppercase();
    let chunk: String = upper.chars().take(8).collect();
    format!("{}-{}", &chunk[0..4], &chunk[4..8])
}
