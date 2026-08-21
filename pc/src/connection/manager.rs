use crate::protocol::Message;
use super::state::ConnectionState;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::time::{interval, Duration};
use tokio_rustls::server::TlsStream;

pub type ControlStream<IO> = TlsStream<IO>;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

pub struct ConnectionManager {
    state_tx: watch::Sender<ConnectionState>,
    outbound_tx: mpsc::Sender<Message>,
    outbound_rx: Arc<Mutex<Option<mpsc::Receiver<Message>>>>,
}

impl ConnectionManager {
    pub fn new(queue: usize) -> (Self, watch::Receiver<ConnectionState>) {
        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);
        let (outbound_tx, outbound_rx) = mpsc::channel(queue);
        (Self { state_tx, outbound_tx, outbound_rx: Arc::new(Mutex::new(Some(outbound_rx))) }, state_rx)
    }
    pub fn sender(&self) -> mpsc::Sender<Message> { self.outbound_tx.clone() }
    pub fn set_state(&self, state: ConnectionState) { self.state_tx.send_replace(state); }

    /// Serve the encrypted control protocol over any TLS stream whose inner transport
    /// implements Tokio AsyncRead/AsyncWrite. This keeps Wi-Fi and Bluetooth identical above TLS.
    pub async fn serve<IO, F>(&self, stream: ControlStream<IO>, on_message: F) -> Result<()>
    where
        IO: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
        F: Fn(Message) + Send + Sync + 'static,
    {
        self.state_tx.send_replace(ConnectionState::Connected);
        let (reader, mut writer) = tokio::io::split(stream);
        let mut lines = BufReader::new(reader).lines();
        let mut outbound = self.outbound_rx.lock().await.take().context("connection manager already serving")?;
        let callback = Arc::new(on_message);
        let mut heartbeat = interval(HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                line = lines.next_line() => {
                    match line? {
                        Some(line) => {
                            let message = Message::from_line(&line)?;
                            let close = matches!(message, Message::Disconnect { .. });
                            callback(message);
                            if close { break; }
                        }
                        None => break,
                    }
                }
                outgoing = outbound.recv() => {
                    match outgoing {
                        Some(message) => {
                            let close = matches!(message, Message::Disconnect { .. });
                            writer.write_all(message.to_line()?.as_bytes()).await?;
                            writer.flush().await?;
                            if close { let _ = writer.shutdown().await; break; }
                        }
                        None => break,
                    }
                }
                _ = heartbeat.tick() => {
                    writer.write_all(Message::Ping.to_line()?.as_bytes()).await?;
                    writer.flush().await?;
                }
            }
        }
        self.state_tx.send_replace(ConnectionState::Disconnected);
        Ok(())
    }
}
