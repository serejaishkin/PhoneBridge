use crate::protocol::SharedState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

pub struct WsServer {
    listener: TcpListener,
    state: Arc<tokio::sync::Mutex<SharedState>>,
}

impl WsServer {
    pub async fn new(bind_addr: &str, state: Arc<tokio::sync::Mutex<SharedState>>) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(bind_addr).await?;
        Ok(Self { listener, state })
    }

    pub async fn run(&self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    log::info!("WebSocket connection from: {}", addr);
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Ok(ws_stream) = accept_async(stream).await {
                            log::info!("WebSocket established with {}", addr);
                        }
                    });
                }
                Err(e) => {
                    log::error!("WebSocket accept error: {}", e);
                }
            }
        }
    }
}
