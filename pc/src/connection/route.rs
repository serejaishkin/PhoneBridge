use crate::platform::bluetooth::BluetoothEndpoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum RouteKind {
    Wifi,
    Hotspot,
    BluetoothPan,
    BluetoothNative,
    Manual,
}

#[derive(Debug, Clone)]
pub enum RouteTarget {
    Tcp { host: String, port: u16 },
    Bluetooth(BluetoothEndpoint),
}

#[derive(Debug, Clone)]
pub struct ConnectionRoute {
    pub kind: RouteKind,
    pub target: RouteTarget,
    pub fingerprint: String,
}

impl ConnectionRoute {
    pub fn tcp(kind: RouteKind, host: impl Into<String>, port: u16, fingerprint: impl Into<String>) -> Self {
        Self { kind, target: RouteTarget::Tcp { host: host.into(), port }, fingerprint: fingerprint.into() }
    }
}

#[derive(Debug, Default)]
pub struct RouteMemory {
    preferred: Option<RouteKind>,
}

impl RouteMemory {
    pub fn set_preferred(&mut self, kind: RouteKind) { self.preferred = Some(kind); }
    pub fn preferred(&self) -> Option<RouteKind> { self.preferred }

    /// Stable ordering: the last successful route is attempted first, then the fallbacks.
    pub fn order(&self, mut routes: Vec<ConnectionRoute>) -> Vec<ConnectionRoute> {
        if let Some(preferred) = self.preferred {
            routes.sort_by_key(|route| if route.kind == preferred { 0 } else { 1 });
        }
        routes
    }
}
