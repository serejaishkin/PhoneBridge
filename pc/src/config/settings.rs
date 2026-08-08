pub struct Settings {
    pub udp_port: u16,
    pub ws_port: u16,
    pub airplay_port: u16,
    pub target_latency_ms: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            udp_port: 5001,
            ws_port: 5000,
            airplay_port: 5002,
            target_latency_ms: 100,
        }
    }
}
