use super::route::RouteKind;
use std::fs;
use std::path::PathBuf;

/// Persists only the transport preference, never credentials or private keys.
pub struct RouteStore {
    path: PathBuf,
}

impl RouteStore {
    pub fn new(path: impl Into<PathBuf>) -> Self { Self { path: path.into() } }

    pub fn load(&self) -> Option<RouteKind> {
        let value = fs::read_to_string(&self.path).ok()?.trim().to_ascii_lowercase();
        match value.as_str() {
            "wifi" => Some(RouteKind::Wifi),
            "hotspot" => Some(RouteKind::Hotspot),
            "bluetooth_pan" => Some(RouteKind::BluetoothPan),
            "bluetooth_native" => Some(RouteKind::BluetoothNative),
            "manual" => Some(RouteKind::Manual),
            _ => None,
        }
    }

    pub fn save(&self, route: RouteKind) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() { fs::create_dir_all(parent)?; }
        fs::write(&self.path, route_name(route))
    }
}

fn route_name(route: RouteKind) -> &'static str {
    match route {
        RouteKind::Wifi => "wifi\n",
        RouteKind::Hotspot => "hotspot\n",
        RouteKind::BluetoothPan => "bluetooth_pan\n",
        RouteKind::BluetoothNative => "bluetooth_native\n",
        RouteKind::Manual => "manual\n",
    }
}
