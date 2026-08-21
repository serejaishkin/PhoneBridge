//! Adapts a WinRT RFCOMM StreamSocket to the common Tokio byte-stream boundary.

#![cfg(windows)]

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio::task::JoinHandle;
use windows::Networking::Sockets::StreamSocket;
use windows::Storage::Streams::{DataReader, DataWriter, InputStreamOptions};

/// A Tokio stream backed by a native Windows RFCOMM StreamSocket.
pub struct WindowsSocketStream {
    stream: DuplexStream,
    bridge: Option<JoinHandle<()>>,
}

impl WindowsSocketStream {
    /// Start bidirectional WinRT↔Tokio forwarding without exposing WinRT types to core code.
    pub async fn from_socket(socket: StreamSocket) -> Result<Self> {
        let (app, bridge) = tokio::io::duplex(64 * 1024);
        let input = socket.InputStream()?;
        let output = socket.OutputStream()?;
        let bridge_task = tokio::spawn(async move {
            let reader = DataReader::CreateDataReader(&input);
            let _ = reader.SetInputStreamOptions(InputStreamOptions::Partial);
            let writer = DataWriter::CreateDataWriter(&output);
            let (mut app_read, mut app_write) = tokio::io::split(bridge);

            let read_task = async {
                let mut buffer = vec![0u8; 16 * 1024];
                loop {
                    let loaded = match reader.LoadAsync(buffer.len() as u32) {
                        Ok(operation) => match operation.await { Ok(value) => value, Err(_) => break },
                        Err(_) => break,
                    };
                    if loaded == 0 { break; }
                    let n = loaded as usize;
                    if reader.ReadBytes(&mut buffer[..n]).is_err() { break; }
                    if app_write.write_all(&buffer[..n]).await.is_err() { break; }
                }
            };

            let write_task = async {
                let mut buffer = vec![0u8; 16 * 1024];
                loop {
                    let n = match app_read.read(&mut buffer).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => n,
                    };
                    if writer.WriteBytes(&buffer[..n]).is_err() { break; }
                    match writer.StoreAsync() {
                        Ok(operation) => { if operation.await.is_err() { break; } }
                        Err(_) => break,
                    }
                }
            };

            let _ = tokio::join!(read_task, write_task);
            if let Ok(operation) = writer.FlushAsync() { let _ = operation.await; }
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
