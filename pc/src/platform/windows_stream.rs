//! Adapts a WinRT RFCOMM StreamSocket to the common Tokio byte-stream boundary.

#![cfg(windows)]

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio::task::JoinHandle;
use windows::Storage::Streams::{DataReader, DataWriter, InputStreamOptions};
use windows::Networking::Sockets::StreamSocket;

use crate::connection::ByteStream;

const BRIDGE_BUFFER: usize = 16 * 1024;
const DUPLEX_CAPACITY: usize = 64 * 1024;

/// A Tokio stream backed by a native Windows RFCOMM StreamSocket.
pub struct WindowsSocketStream {
    stream: DuplexStream,
    bridge: Option<JoinHandle<()>>,
}

impl WindowsSocketStream {
    /// Start bidirectional WinRT↔Tokio forwarding without exposing WinRT types to core code.
    pub async fn from_socket(socket: StreamSocket) -> Result<Self> {
        let (mut app, mut bridge) = tokio::io::duplex(DUPLEX_CAPACITY);
        let input = socket.InputStream()?;
        let output = socket.OutputStream()?;
        let bridge_task = tokio::spawn(async move {
            let reader = DataReader::CreateDataReader(&input);
            let _ = reader.SetInputStreamOptions(InputStreamOptions::Partial);
            let writer = DataWriter::CreateDataWriter(&output);
            let (mut app_read, mut app_write) = tokio::io::split(bridge);

            let read_task = async {
                let mut buffer = vec![0u8; BRIDGE_BUFFER];
                loop {
                    let loaded = reader.LoadAsync(BRIDGE_BUFFER as u32).ok()?.await.ok()?;
                    if loaded == 0 { break; }
                    let n = loaded as usize;
                    if reader.ReadBytes(&mut buffer[..n]).is_err() { break; }
                    if app_write.write_all(&buffer[..n]).await.is_err() { break; }
                }
                Some(())
            };

            let write_task = async {
                let mut buffer = vec![0u8; BRIDGE_BUFFER];
                loop {
                    let n = match app_read.read(&mut buffer).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => n,
                    };
                    if writer.WriteBytes(&buffer[..n]).is_err() { break; }
                    if writer.StoreAsync().ok()?.await.is_err() { break; }
                }
                Some(())
            };

            let _ = tokio::join!(read_task, write_task);
            let _ = writer.FlushAsync().ok().and_then(|operation| operation.await.ok());
        });

        Ok(Self { stream: app, bridge: Some(bridge_task) })
    }

    pub fn into_inner(mut self) -> DuplexStream {
        if let Some(task) = self.bridge.take() { task.abort(); }
        self.stream
    }
}

impl Drop for WindowsSocketStream {
    fn drop(&mut self) {
        if let Some(task) = self.bridge.take() { task.abort(); }
    }
}

impl tokio::io::AsyncRead for WindowsSocketStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for WindowsSocketStream {
    fn poll_write(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &[u8]) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl ByteStream for WindowsSocketStream {}
