use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

/// Minimal WebSocket server shell kept independent from the legacy protocol state.
///
/// The KDE Connect migration will eventually carry device/session state through
/// the common connection layer instead of exposing the old `SharedState` type.
pub struct WsServer {
    listener: TcpListener,
}

impl WsServer {
    pub async fn new(bind_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(bind_addr).await?;
        Ok(Self { listener })
    }

    pub async fn run(&self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    log::info!("WebSocket connection from: {}", addr);
                    tokio::spawn(async move {
                        if accept_async(stream).await.is_ok() {
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
