use super::route::{ConnectionRoute, RouteKind, RouteMemory, RouteTarget};
use super::route_store::RouteStore;
use anyhow::Result;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Chooses transport routes before the TLS layer and commits a preferred route only after authentication.
pub struct ConnectionCoordinator {
    memory: RouteMemory,
    route_store: Option<RouteStore>,
    connect_timeout: Duration,
}

impl ConnectionCoordinator {
    pub fn new(memory: RouteMemory, connect_timeout: Duration) -> Self {
        Self { memory, route_store: None, connect_timeout }
    }

    pub fn with_route_store(mut self, store: RouteStore) -> Self {
        if let Some(preferred) = store.load() { self.memory.set_preferred(preferred); }
        self.route_store = Some(store);
        self
    }

    pub fn memory(&self) -> &RouteMemory { &self.memory }

    /// Try routes in preferred-first order. TCP success alone does not persist a route.
    pub async fn connect_tcp_routes(&mut self, routes: Vec<ConnectionRoute>) -> Result<(RouteKind, TcpStream)> {
        let routes = self.memory.order(routes);
        let mut last_error = None;
        for route in routes {
            let kind = route.kind;
            let (host, port) = match route.target {
                RouteTarget::Tcp { host, port } => (host, port),
                RouteTarget::Bluetooth(_) => continue,
            };
            match timeout(self.connect_timeout, TcpStream::connect((host.as_str(), port))).await {
                Ok(Ok(stream)) => return Ok((kind, stream)),
                Ok(Err(error)) => last_error = Some(error.into()),
                Err(_) => last_error = Some(anyhow::anyhow!("connection attempt timed out")),
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("no TCP routes available")))
    }

    /// Commit the route only after the TLS/control session has authenticated the peer.
    pub fn mark_authenticated(&mut self, kind: RouteKind) -> Result<()> {
        self.memory.set_preferred(kind);
        if let Some(store) = &self.route_store { store.save(kind)?; }
        Ok(())
    }
}
