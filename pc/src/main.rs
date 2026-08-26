mod audio;
mod call;
mod discovery;
mod network;
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
use ui::UiBackend;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // winit 0.30 (behind eframe) requires the event loop to run on the OS main
    // thread; launching it from a spawned thread panics on Windows. Layout:
    // the Tokio runtime owns all background network tasks on its worker
    // threads, while this main thread is reserved for the GUI event loop.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let identity_dir = pairing::identity::default_identity_dir();
    let identity = Arc::new(Identity::load_or_create(&identity_dir)?);
    log::info!(
        "device_id={} fingerprint={} short_code={}",
        identity.device_id,
        identity.fingerprint_hex(),
        pairing::trust::short_code(&identity.fingerprint_hex())
    );

    let trust_store = Arc::new(TokioMutex::new(TrustStore::load(&identity_dir)?));
    let shared_state = call::SharedState::new();
    let sms_controller = SmsController::new();
    let sms_store = Arc::new(TokioMutex::new(SmsStore::new()));
    let desktop_state = Arc::new(Mutex::new(DesktopState::default()));
    let ui = Arc::new(DesktopUi::new(desktop_state.clone()));

    // Show the PC pairing code in the top bar so the user can compare it with
    // the phone during first-time pairing.
    desktop_state.lock().unwrap().local_code = pairing::trust::short_code(&identity.fingerprint_hex());

    // Async startup work runs on the runtime; results are pushed into the UI.
    {
        let ui = ui.clone();
        let shared_state = shared_state.clone();
        runtime.spawn(async move {
            let status = call::check_hfp_support().await;
            *shared_state.hfp_support.lock().await = status;
            ui.update_hfp_status(status).await;
        });
    }

    let pairing_server = PairingServer::new(identity.clone(), trust_store.clone(), ui.clone(), sms_controller.clone(), sms_store.clone())?;
    runtime.spawn(async move {
        if let Err(e) = pairing_server.run().await {
            log::error!("pairing server task exited: {e:#}");
        }
    });

    {
        let identity = identity.clone();
        runtime.spawn(async move {
            if let Err(e) = discovery::run_broadcaster(identity).await {
                log::error!("discovery task exited: {e:#}");
            }
        });
    }

    // Media audio receiver: Android captures playback via MediaProjection,
    // Opus-encodes it and streams UDP packets ([seq u16 BE][opus]) here.
    // Runs for the whole daemon lifetime; silence on the port is normal while
    // the phone is not casting.
    let stream_activity = network::udp_server::StreamActivity::default();
    {
        runtime.spawn({
            let activity = stream_activity.clone();
            async move {
                let jitter = Arc::new(tokio::sync::Mutex::new(audio::jitter_buffer::JitterBuffer::new(4, 32)));
                let decoder = match audio::decoder::OpusDecoder::new() {
                    Ok(decoder) => decoder,
                    Err(e) => {
                        log::error!("media audio disabled, opus decoder init failed: {e}");
                        return;
                    }
                };
                let server = match network::udp_server::UdpServer::new("0.0.0.0:5001", jitter, decoder, activity).await {
                    Ok(server) => server,
                    Err(e) => {
                        log::error!("media audio disabled, cannot bind UDP :5001 ({e})");
                        return;
                    }
                };
                log::info!("media audio listening on udp/:5001");
                server.run().await;
            }
        });
    }

    // Reflect stream liveness in the UI: active while packets keep arriving.
    {
        let state = desktop_state.clone();
        let activity = stream_activity.clone();
        runtime.spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let streaming = activity.is_active();
                let mut guard = state.lock().unwrap();
                if guard.streaming != streaming {
                    guard.streaming = streaming;
                }
            }
        });
    }

    log::info!("phonebridge2 PC daemon + desktop GUI started");
    let gui_state = desktop_state.clone();
    let gui_store = sms_store.clone();
    let gui_controller = sms_controller.clone();
    let runtime_handle = runtime.handle().clone();

    // Block the main thread on the GUI event loop until the window closes.
    let options = eframe::NativeOptions::default();
    let result = eframe::run_native(
        "PhoneBridge",
        options,
        Box::new(move |_cc| Ok(Box::new(PhoneBridgeApp::new(gui_state, gui_store, gui_controller, runtime_handle)))),
    );

    // Window closed: stop network tasks before exiting.
    runtime.shutdown_timeout(std::time::Duration::from_millis(500));
    result.map_err(|e| anyhow::anyhow!("eframe event loop failed: {e:?}"))
}
