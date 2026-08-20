mod call;
mod connection;
mod discovery;
mod pairing;
mod platform;
mod protocol;
mod ui;

use pairing::identity::Identity;
use pairing::server::PairingServer;
use pairing::trust::TrustStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use ui::{BasicUi, UiBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let identity_dir = pairing::identity::default_identity_dir();
    let identity = Arc::new(Identity::load_or_create(&identity_dir)?);
    log::info!("device_id={} fingerprint={} short_code={}", identity.device_id, identity.fingerprint_hex(), pairing::trust::short_code(&identity.fingerprint_hex()));

    let trust_store = Arc::new(Mutex::new(TrustStore::load(&identity_dir)?));
    let shared_state = call::SharedState::new();
    let ui = Arc::new(BasicUi::new());
    let ui_trait: Arc<dyn UiBackend> = ui.clone();

    let platform = platform::current();
    let hfp_status = platform.hfp_support();
    *shared_state.hfp_support.lock().await = hfp_status;
    ui_trait.update_hfp_status(hfp_status).await;
    ui_trait.update_connection_status(false, None).await;
    log::info!("platform={:?}, hfp_support={:?}", platform.kind(), hfp_status);

    let pairing_server = PairingServer::new(identity.clone(), trust_store.clone())?;
    let pairing_task = tokio::spawn(pairing_server.run());
    let discovery_task = tokio::spawn(discovery::run_broadcaster(identity.clone()));

    log::info!("phonebridge2 PC daemon started");
    tokio::select! {
        res = pairing_task => log::error!("pairing server task exited: {:?}", res),
        res = discovery_task => log::error!("discovery task exited: {:?}", res),
    }
    Ok(())
}
