use phonebridge::{
    audio::{decoder::OpusDecoder, encoder::*, input::AudioInput, jitter_buffer::JitterBuffer, output::AudioOutput},
    discovery::BleDiscovery,
    network::{udp_server::UdpServer, udp_sender::UdpSender, ws_server::WsServer},
    protocol::SharedState,
    shairport::ShairportManager,
    ui::{AsyncTrayUI, TrayCommand},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("PhoneBridge V1.3 starting...");

    let state = Arc::new(Mutex::new(SharedState::new()));

    // AirPlay (iOS media fallback)
    let shairport = ShairportManager::new();
    match shairport.detect_receiver() {
        Some(receiver) => {
            info!("AirPlay receiver detected: {}", receiver.name());
            if let Err(e) = shairport.start() {
                log::warn!("Failed to start AirPlay: {}", e);
            }
        }
        None => {
            log::warn!("No AirPlay receiver. iOS media via AirPlay disabled.");
        }
    }

    // Tray UI (Windows)
    let (tray, _tray_tx) = AsyncTrayUI::new()?;

    // Audio pipeline: Android → PC
    let jitter = Arc::new(Mutex::new(JitterBuffer::new(5, 20)));
    let audio_out = AudioOutput::new()?;
    let decoder = OpusDecoder::new()?;
    let udp_server = UdpServer::new("0.0.0.0:5001", jitter.clone(), decoder).await?;
    tokio::spawn(async move { udp_server.run().await });

    // WebSocket signaling
    let ws_server = WsServer::new("0.0.0.0:5000", state.clone()).await?;
    tokio::spawn(async move { ws_server.run().await });

    // BLE Discovery + PC Mic → Android
    let discovery = BleDiscovery::new().await?;
    info!("Scanning for PhoneBridge devices via BLE...");

    let device = discovery.find_first(10).await?;
    let (mic_target, ws_target) = if let Some(dev) = device {
        info!("Found device: {} @ {}:{}", dev.name, dev.ip, dev.port);
        let mic_target: SocketAddr = format!("{}:5003", dev.ip).parse()?;
        let ws_target = format!("ws://{}:5000", dev.ip);
        (mic_target, ws_target)
    } else {
        log::warn!("No BLE device found. Falling back to manual IP (192.168.137.2:5003)");
        let mic_target: SocketAddr = "192.168.137.2:5003".parse()?;
        let ws_target = "ws://192.168.137.2:5000".to_string();
        (mic_target, ws_target)
    };

    // PC Microphone → Android (UDP 5003)
    let (audio_input, opus_rx) = AudioInput::new()?;
    let mut udp_sender = UdpSender::new("0.0.0.0:0", mic_target, opus_rx).await?;
    tokio::spawn(async move {
        udp_sender.run().await;
    });
    info!("PC microphone streaming to {}", mic_target);

    info!("PhoneBridge V1.3 ready. UDP in:5001, UDP out:5003, WS:5000, AirPlay:5002");

    // Main event loop
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let commands = tray.poll().await;
        for cmd in commands {
            match cmd {
                TrayCommand::AnswerCall => info!("Tray: Answer Call"),
                TrayCommand::EndCall => info!("Tray: End Call"),
                TrayCommand::ToggleMute => info!("Tray: Toggle Mute"),
                TrayCommand::OpenSettings => info!("Tray: Open Settings"),
                TrayCommand::Quit => {
                    info!("Tray: Quit requested");
                    return Ok(());
                }
            }
        }
    }
}
