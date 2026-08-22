//! Platform-neutral UI backend interface.

use crate::protocol::HfpSupport;
use async_trait::async_trait;

#[async_trait]
pub trait UiBackend: Send + Sync {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>);
    async fn notify_call_ended(&self);
    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>);
    async fn update_hfp_status(&self, status: HfpSupport);
    async fn notify_sms_received(&self, address: &str, body: &str, timestamp: i64);
}

pub struct HeadlessUi;

#[async_trait]
impl UiBackend for HeadlessUi {
    async fn notify_incoming_call(&self, caller_name: Option<&str>, caller_number: Option<&str>) {
        log::info!("[UI] incoming call: {} ({})", caller_name.unwrap_or("unknown"), caller_number.unwrap_or("no number"));
    }

    async fn notify_call_ended(&self) {
        log::info!("[UI] call ended");
    }

    async fn update_connection_status(&self, connected: bool, peer_name: Option<&str>) {
        log::info!("[UI] connection status: {} ({})", if connected { "connected" } else { "disconnected" }, peer_name.unwrap_or("-"));
    }

    async fn update_hfp_status(&self, status: HfpSupport) {
        log::info!("[UI] HFP support: {:?}", status);
    }

    async fn notify_sms_received(&self, address: &str, body: &str, timestamp: i64) {
        log::info!("[UI] SMS received: from={address} timestamp={timestamp}: {body}");
    }
}
