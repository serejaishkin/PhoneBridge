use crate::protocol::Message;
use super::state::ConnectionState;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_rustls::server::TlsStream;

pub type ControlStream = TlsStream<TcpStream>;

pub struct ConnectionManager {
    state_tx: watch::Sender<ConnectionState>,
    outbound_tx: mpsc::Sender<Message>,
    outbound_rx: Arc<Mutex<Option<mpsc::Receiver<Message>>>>,
}

impl ConnectionManager {
    pub fn new(queue: usize) -> (Self, watch::Receiver<ConnectionState>) {
        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);
        let (outbound_tx, outbound_rx) = mpsc::channel(queue);
        (
            Self {
                state_tx,
                outbound_tx,
                outbound_rx: Arc::new(Mutex::new(Some(outbound_rx))),
            },
            state_rx,
        )
    }

    pub fn sender(&self) -> mpsc::Sender<Message> {
        self.outbound_tx.clone()
    }

    pub fn set_state(&self, state: ConnectionState) {
        self.state_tx.send_replace(state);
    }

    pub async fn serve<F>(&self, stream: ControlStream, on_message: F) -> Result<()>
    where
        F: Fn(Message) + Send + Sync + 'static,
    {
        self.state_tx.send_replace(ConnectionState::Connected);
        let (reader, mut writer) = tokio::io::split(stream);
        let mut lines = BufReader::new(reader).lines();
        let mut outbound = self
            .outbound_rx
            .lock()
            .await
            .take()
            .context("connection manager already serving")?;
        let callback = Arc::new(on_message);

        loop {
            tokio::select! {
                line = lines.next_line() => {
                    match line? {
                        Some(line) => callback(Message::from_line(&line)?),
                        None => break,
                    }
                }
                outgoing = outbound.recv() => {
                    match outgoing {
                        Some(message) => writer.write_all(message.to_line()?.as_bytes()).await?,
                        None => break,
                    }
                }
            }
        }

        self.state_tx.send_replace(ConnectionState::Disconnected);
        Ok(())
    }
}
