//! Persistent TLS identity, certificate fingerprinting, and trusted-peer storage.
//!
//! This is the security boundary for the KDE Connect migration. The private
//! key stays on the PC and the certificate fingerprint is stable across
//! restarts. Pairing approval is stored separately from the TLS certificate.

use anyhow::{Context, Result};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone)]
pub struct LocalTlsIdentity {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub fingerprint: String,
}

impl LocalTlsIdentity {
    pub fn load_or_create(app_dir: impl AsRef<Path>) -> Result<Self> {
        let dir = app_dir.as_ref();
        fs::create_dir_all(dir).context("create PhoneBridge security directory")?;
        let cert_path = dir.join("identity.der");
        let key_path = dir.join("identity.key");

        let (cert, key) = if cert_path.exists() && key_path.exists() {
            (fs::read(&cert_path)?, fs::read(&key_path)?)
        } else {
            let subject = hostname();
            let generated = generate_simple_self_signed(vec![subject])?;
            let cert = generated.cert.der().to_vec();
            let key = generated.key_pair.serialize_der();
            write_private_file(&cert_path, &cert)?;
            write_private_file(&key_path, &key)?;
            (cert, key)
        };

        let fingerprint = certificate_fingerprint(&cert);
        Ok(Self { cert, key, fingerprint })
    }

    pub fn rustls_certificate(&self) -> CertificateDer<'static> {
        CertificateDer::from(self.cert.clone())
    }

    pub fn rustls_private_key(&self) -> PrivateKeyDer<'static> {
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.key.clone()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedPeer {
    pub device_id: String,
    pub device_name: String,
    pub fingerprint: String,
    pub paired_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustStore {
    pub peers: Vec<TrustedPeer>,
}

impl TrustStore {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_vec_pretty(self)?;
        write_private_file(path, &data)
    }

    pub fn is_trusted(&self, device_id: &str, fingerprint: &str) -> bool {
        self.peers.iter().any(|p| p.device_id == device_id && p.fingerprint == fingerprint)
    }

    pub fn trust(&mut self, peer: TrustedPeer) {
        self.peers.retain(|p| p.device_id != peer.device_id);
        self.peers.push(peer);
    }

    pub fn revoke(&mut self, device_id: &str) {
        self.peers.retain(|p| p.device_id != device_id);
    }
}

pub fn certificate_fingerprint(cert: &[u8]) -> String {
    let digest = Sha256::digest(cert);
    digest.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":")
}

pub fn default_security_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("PhoneBridge");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join("phonebridge");
    }
    PathBuf::from(".phonebridge")
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "PhoneBridge PC".into())
}

fn write_private_file(path: &Path, data: &[u8]) -> Result<()> {
    fs::write(path, data).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn generated_identity_is_stable() {
        let dir = std::env::temp_dir().join(format!("phonebridge-tls-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        let first = LocalTlsIdentity::load_or_create(&dir).unwrap();
        let second = LocalTlsIdentity::load_or_create(&dir).unwrap();
        assert_eq!(first.fingerprint, second.fingerprint);
        let _ = fs::remove_dir_all(dir);
    }
}
