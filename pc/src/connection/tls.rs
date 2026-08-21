//! Transport-independent TLS boundary for PhoneBridge control sessions.
//!
//! The transport below may be TCP, Bluetooth RFCOMM, or another AsyncRead/AsyncWrite source.

use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tokio_rustls::rustls::{pki_types::CertificateDer, ServerConfig};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

/// Build the server-side TLS acceptor from the persistent PC certificate and key.
///
/// Certificate generation/storage stays outside this module so identity rotation and
/// fingerprint pinning remain explicit policy decisions rather than transport behavior.
pub fn server_acceptor(
    certificate_der: Vec<u8>,
    private_key_der: Vec<u8>,
) -> Result<TlsAcceptor> {
    let certificate = CertificateDer::from(certificate_der);
    let private_key = tokio_rustls::rustls::pki_types::PrivateKeyDer::try_from(private_key_der)
        .map_err(|_| anyhow::anyhow!("unsupported private key encoding"))?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], private_key)
        .context("failed to build PhoneBridge TLS server configuration")?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Perform a bounded TLS handshake over any PhoneBridge byte stream.
///
/// This deliberately returns a generic `TlsStream<IO>` so the control protocol does not
/// need to know whether the underlying connection came from Wi-Fi, hotspot, or Bluetooth.
pub async fn accept<IO>(
    acceptor: &TlsAcceptor,
    io: IO,
    handshake_timeout: Duration,
) -> Result<TlsStream<IO>>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    timeout(handshake_timeout, acceptor.accept(io))
        .await
        .context("PhoneBridge TLS handshake timed out")?
        .context("PhoneBridge TLS handshake failed")
}
