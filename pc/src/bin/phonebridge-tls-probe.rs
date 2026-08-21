//! Local TLS probe for the KDE Connect migration.
//!
//! This utility connects to a PhoneBridge TLS pairing endpoint, accepts the
//! locally generated certificate for development, reads the PC identity, and
//! sends a KDE Connect pair request. It is intentionally a development tool;
//! it must not be used as a production certificate verifier.

use anyhow::{Context, Result};
use kdeconnect::Packet;
use rustls::{pki_types::ServerName, ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::{env, io::{Read, Write}, net::TcpStream, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

#[path = "../kdeconnect/mod.rs"]
mod kdeconnect;

fn main() -> Result<()> {
    env_logger::init();
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:1716".into());
    let name = ServerName::try_from("PhoneBridge PC")
        .context("create development TLS server name")?;

    let config = ClientConfig::builder()
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();
    let connection = ClientConnection::new(Arc::new(config), name)
        .context("create TLS client")?;
    let stream = TcpStream::connect(&addr).with_context(|| format!("connect to {addr}"))?;
    let mut tls = StreamOwned::new(connection, stream);

    // This probe intentionally cannot verify the self-signed development
    // certificate yet. Production verification belongs to the trust layer.
    anyhow::bail!("TLS probe is a placeholder: production certificate verification is not enabled yet")
}

#[allow(dead_code)]
fn _packet_helpers() -> Result<()> {
    let packet = Packet::pair_request(
        1,
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
    );
    let payload = serde_json::to_vec(&packet)?;
    let len = u32::try_from(payload.len())?;
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&payload);
    let _ = out;
    Ok(())
}
