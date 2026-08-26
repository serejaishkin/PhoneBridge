//! PC microphone relay: captures the default input device, Opus-encodes it and
//! streams `[seq u16 BE][opus]` packets to the phone (AudioPlaybackService
//! listens on UDP 5003). Packet format mirrors udp_server.rs in reverse.
//!
//! Everything audio-related is created and owned by one dedicated thread so no
//! `!Send` cpal types ever cross a boundary; stopping is cooperative via an
//! atomic flag.

use crate::audio::input::AudioInput;
use std::net::{IpAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const PHONE_MIC_PORT: u16 = 5003;

pub struct MicRelay {
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl MicRelay {
    /// Spawns the capture+send thread targeting `phone_ip:PHONE_MIC_PORT`.
    pub fn start(phone_ip: IpAddr) -> anyhow::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let target = std::net::SocketAddr::new(phone_ip, PHONE_MIC_PORT);

        let handle = std::thread::Builder::new()
            .name("phonebridge-mic-relay".into())
            .spawn(move || {
                let input = match AudioInput::new() {
                    Ok((input, rx)) => {
                        // Keep both handles alive on this thread for the whole session.
                        let _own_receiver_copy = rx.clone();
                        (input, rx)
                    }
                    Err(e) => {
                        log::error!("mic relay: cannot open input device ({e})");
                        return;
                    }
                };
                let (_input, rx) = input;

                let socket = match UdpSocket::bind(("0.0.0.0", 0)) {
                    Ok(socket) => socket,
                    Err(e) => {
                        log::error!("mic relay: cannot bind UDP socket ({e})");
                        return;
                    }
                };

                log::info!("mic relay started -> {target}");
                let mut seq: u16 = 0;
                while !stop_flag.load(Ordering::Relaxed) {
                    match rx.recv_timeout(Duration::from_millis(200)) {
                        Ok(mut packet) => {
                            seq = seq.wrapping_add(1);
                            let mut datagram = Vec::with_capacity(packet.len() + 2);
                            datagram.extend_from_slice(&seq.to_be_bytes());
                            datagram.append(&mut packet);
                            if let Err(e) = socket.send_to(&datagram, target) {
                                log::debug!("mic relay: send failed ({e})");
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    }
                }
                log::info!("mic relay stopped");
            })?;

        Ok(Self { stop, handle: Some(handle) })
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for MicRelay {
    fn drop(&mut self) {
        self.stop();
    }
}
