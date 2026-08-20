use super::route::{ConnectionRoute, RouteKind, RouteMemory, RouteTarget};
use crate::connection::timeout::ConnectionTimeouts;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Chooses a transport route and returns a connected TCP stream for the TLS layer.
/// Native Bluetooth transports are intentionally handled by the platform boundary.
pub struct ConnectionCoordinator {
    memory: RouteMemory,
    timeouts: ConnectionTimeouts,
}

impl ConnectionCoordinator {
    pub fn new(memory: RouteMemory, timeouts: ConnectionTimeouts) -> Self { Self { memory, timeouts } }

    pub fn memory(&self) -> &RouteMemory { &self.memory }

    pub async fn connect_tcp_routes(&mut self, routes: Vec<ConnectionRoute>) -> Result<(RouteKind, TcpStream)> {
        let routes = self.memory.order(routes);
        let mut last_error = None;
        for route in routes {
            let kind = route.kind;
            let target = match route.target {
                RouteTarget::Tcp { host, port } => (host, port),
                RouteTarget::Bluetooth(_) => continue,
            };
            match timeout(self.timeouts.connect, TcpStream::connect((&*target.0, target.1))).await {
                Ok(Ok(stream)) => {
                    self.memory.set_preferred(kind);
                    return Ok((kind, stream));
                }
                Ok(Err(error)) => last_error = Some(error.into()),
                Err(_) => last_error = Some(anyhow::anyhow!("connection attempt timed out")),
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no TCP routes available")))
    }
}
