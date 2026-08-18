//! Идентичность узла для TLS-пейринга.
//!
//! Идея (позаимствована у KDE Connect, но реализована самостоятельно — см.
//! обсуждение лицензии в AI_HANDOFF): у каждого устройства (PC/телефон) есть
//! свой самоподписанный сертификат, который живёт на диске годами. Пейринг —
//! это не "логин/пароль", а один раз подтверждённое доверие к конкретному
//! отпечатку (fingerprint) сертификата. TLS дальше просто шифрует канал,
//! аутентификация — на уровне "я уже видел этот fingerprint и доверяю ему".

use anyhow::{Context, Result};
use rcgen::generate_simple_self_signed;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Identity {
    pub device_id: String,
    pub cert_pem: String,
    pub key_pem: String,
    pub cert_der: Vec<u8>,
}

impl Identity {
    /// SHA-256 отпечаток сертификата в hex — то, что пользователь сверяет глазами
    /// при первом пейринге (см. pairing::trust::short_code).
    pub fn fingerprint_hex(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.cert_der);
        hex::encode(hasher.finalize())
    }

    /// Загрузить идентичность с диска, либо сгенерировать новую и сохранить.
    pub fn load_or_create(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir).context("creating identity dir")?;
        let cert_path = dir.join("cert.pem");
        let key_path = dir.join("key.pem");
        let id_path = dir.join("device_id.txt");

        if cert_path.exists() && key_path.exists() && id_path.exists() {
            let cert_pem = fs::read_to_string(&cert_path)?;
            let key_pem = fs::read_to_string(&key_path)?;
            let device_id = fs::read_to_string(&id_path)?.trim().to_string();
            let cert_der = pem_to_der(&cert_pem)?;
            return Ok(Self {
                device_id,
                cert_pem,
                key_pem,
                cert_der,
            });
        }

        let device_id = format!("pb2-{}", uuid_v4_like());
        // subject alt name не используется для реальной проверки домена — у нас
        // доверие идёт по fingerprint, а не по имени, но rcgen требует хотя бы один SAN.
        let certified = generate_simple_self_signed(vec![device_id.clone()])
            .context("generating self-signed certificate")?;
        let cert_pem = certified.cert.pem();
        let key_pem = certified.key_pair.serialize_pem();
        let cert_der = certified.cert.der().to_vec();

        fs::write(&cert_path, &cert_pem)?;
        fs::write(&key_path, &key_pem)?;
        fs::write(&id_path, &device_id)?;

        Ok(Self {
            device_id,
            cert_pem,
            key_pem,
            cert_der,
        })
    }
}

fn pem_to_der(pem: &str) -> Result<Vec<u8>> {
    let mut reader = std::io::Cursor::new(pem.as_bytes());
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("parsing cert.pem")?;
    let first = certs.into_iter().next().context("cert.pem is empty")?;
    Ok(first.to_vec())
}

/// Простой генератор случайного ID без внешней зависимости на `uuid` crate,
/// формат не является настоящим UUID, но уникален и достаточен как device_id.
fn uuid_v4_like() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn default_identity_dir() -> PathBuf {
    dirs_next_home().join(".phonebridge2")
}

fn dirs_next_home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
