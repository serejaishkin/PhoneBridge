use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

/// Common transport boundary used by TCP and native Bluetooth links.
#[async_trait]
pub trait ByteStream: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T> ByteStream for T where T: AsyncRead + AsyncWrite + Send + Unpin {}

pub type BoxByteStream = Box<dyn ByteStream>;

/// Adapt a transport into the tokio byte-stream contract used by TLS.
pub fn boxed<S>(stream: S) -> BoxByteStream
where
    S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    Box::new(stream)
}

/// Wrap an arbitrary stream with a descriptive transport label for diagnostics.
pub struct LabeledStream<S> {
    inner: S,
    label: &'static str,
}

impl<S> LabeledStream<S> {
    pub fn new(inner: S, label: &'static str) -> Self { Self { inner, label } }
    pub fn label(&self) -> &'static str { self.label }
}

impl<S: AsyncRead + Unpin> AsyncRead for LabeledStream<S> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for LabeledStream<S> {
    fn poll_write(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &[u8]) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, buf)
    }
    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Keep this constructor fallible so future OS adapters can attach transport context.
pub fn labeled<S>(stream: S, label: &'static str) -> Result<LabeledStream<S>> {
    if label.is_empty() { return Err(anyhow::anyhow!("transport label cannot be empty")).context("creating byte stream"); }
    Ok(LabeledStream::new(stream, label))
}
