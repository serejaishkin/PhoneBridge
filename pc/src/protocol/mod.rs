use std::collections::HashMap;

pub struct SharedState {
    pub connected: bool,
    pub call_active: bool,
    pub device_name: Option<String>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            connected: false,
            call_active: false,
            device_name: None,
        }
    }
}
