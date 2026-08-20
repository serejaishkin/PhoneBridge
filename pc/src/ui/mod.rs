//! Platform-neutral UI contract. Platform GUI code must depend on this layer.

use crate::protocol::HfpSupport;
use async_trait::async_trait;

mod basic;
mod native;
pub use basic::{BasicUi, UiState};

#[async_trait]
pub trait UiBackend: Send + Sync {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>);
    async fn notify_call_ended(&self);
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>);
    async fn update_hfp_status(&self, status: HfpSupport);
}

pub struct HeadlessUi;

#[async_trait]
impl UiBackend for HeadlessUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) { log::info!("[UI] incoming call: {} ({})", caller_name.unwrap_or("unknown"), caller_number.unwrap_or("no number")); }
    async fn notify_call_ended(&self) { log::info!("[UI] call ended"); }
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) { log::info!("[UI] connection status: {} ({})", if connected { "connected" } else { "disconnected" }, peer_name.unwrap_or("-")); }
    async fn update_hfp_status(&self, status: HfpSupport) { log::info!("[UI] HFP support: {:?}", status); }
}
