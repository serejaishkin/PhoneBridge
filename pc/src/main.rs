mod call;
mod discovery;
mod pairing;
mod protocol;
mod sms;
mod ui;

use pairing::identity::Identity;
use pairing::server::PairingServer;
use pairing::trust::TrustStore;
use sms::{SmsController, SmsStore};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;
use ui::desktop::{DesktopState, DesktopUi, PhoneBridgeApp};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let identity_dir = pairing::identity::default_identity_dir();
    let identity = Arc::new(Identity::load_or_create(&identity_dir)?);
    log::info!("device_id={} fingerprint={} short_code={}", identity.device_id, identity.fingerprint_hex(), pairing::trust::short_code(&identity.fingerprint_hex()));

    let trust_store = Arc::new(TokioMutex::new(TrustStore::load(&identity_dir)?));
    let shared_state = call::SharedState::new();
    let sms_controller = SmsController::new();
    let sms_store = Arc::new(TokioMutex::new(SmsStore::new()));
    let desktop_state = Arc::new(Mutex::new(DesktopState::default()));
    let ui = Arc::new(DesktopUi::new(desktop_state.clone()));

    let hfp_status = call::check_hfp_support().await;
    *shared_state.hfp_support.lock().await = hfp_status;
    ui.update_hfp_status(hfp_status).await;

    let pairing_server = PairingServer::new(identity.clone(), trust_store.clone(), ui.clone(), sms_controller.clone(), sms_store.clone())?;
    let pairing_task = tokio::spawn(pairing_server.run());
    let discovery_task = tokio::spawn(discovery::run_broadcaster(identity.clone()));

    let gui_state = desktop_state.clone();
    let gui_store = sms_store.clone();
    let gui_controller = sms_controller.clone();
    let runtime_handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        let options = eframe::NativeOptions::default();
        let _ = eframe::run_native("PhoneBridge", options, Box::new(move |_cc| Ok(Box::new(PhoneBridgeApp::new(gui_state, gui_store, gui_controller, runtime_handle)))));
    });

    log::info!("phonebridge2 PC daemon + desktop GUI started");
    tokio::select! {
        res = pairing_task => log::error!("pairing server task exited: {:?}", res),
        res = discovery_task => log::error!("discovery task exited: {:?}", res),
    }
    Ok(())
}
