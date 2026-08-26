//! UDP receiver for the Android -> PC media-audio stream.
//!
//! Packet format matches android/.../AudioCaptureService.kt:
//! `[seq u16 big-endian][opus encoded bytes]`, 48 kHz mono PCM after decoding.
//!
//! Threading note: `cpal::Stream` is `!Send`, so the output device lives on a
//! dedicated parked OS thread while this task only touches channels and the
//! jitter buffer, keeping the whole future `Send`.

use crate::audio::{decoder::OpusDecoder, jitter_buffer::JitterBuffer, output::AudioOutput};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex as AsyncMutex;

pub const MEDIA_PORT: u16 = 5001;

/// Shared marker of stream liveness: `Some(instant)` of the last received
/// packet. The UI ticker treats a 2 s silence as "stream stopped".
#[derive(Debug, Clone, Default)]
pub struct StreamActivity(Arc<Mutex<Option<Instant>>>);

impl StreamActivity {
    pub fn is_active(&self) -> bool {
        match self.0.lock().unwrap().as_ref() {
            Some(last) => last.elapsed() < Duration::from_secs(2),
            None => false,
        }
    }

    fn mark(&self) {
        *self.0.lock().unwrap() = Some(Instant::now());
    }
}

pub struct UdpServer {
    socket: UdpSocket,
    jitter: Arc<AsyncMutex<JitterBuffer>>,
    decoder: Arc<AsyncMutex<OpusDecoder>>,
    frame_tx: Sender<Vec<i16>>,
    activity: StreamActivity,
}

impl UdpServer {
    pub async fn new(
        bind_addr: &str,
        jitter: Arc<AsyncMutex<JitterBuffer>>,
        decoder: OpusDecoder,
        activity: StreamActivity,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(bind_addr).await?;

        // cpal::Stream is !Send, so the output device must be created AND owned
        // by one dedicated thread; only the crossbeam sender crosses over.
        let (tx_out, rx_out) = std::sync::mpsc::channel::<Result<Sender<Vec<i16>>, String>>();
        std::thread::Builder::new()
            .name("phonebridge-audio-out".into())
            .spawn(move || match AudioOutput::new() {
                Ok(audio_out) => {
                    let frame_tx = audio_out.sender_clone();
                    let _ = tx_out.send(Ok(frame_tx));
                    // Park forever holding the stream; the cpal callback keeps
                    // pulling frames from the channel on this thread.
                    loop { std::thread::park(); }
                }
                Err(e) => {
                    let _ = tx_out.send(Err(e.to_string()));
                }
            })?;
        let frame_tx = rx_out.recv()??;

        Ok(Self { socket, jitter, decoder: Arc::new(AsyncMutex::new(decoder)), frame_tx, activity })
    }

    /// Receive-decode-playback loop. Runs until the socket errors fatally.
    pub async fn run(&self) {
        let mut buf = vec![0u8; 1500];
        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, _addr)) => {
                    if len < 4 {
                        continue;
                    }
                    self.activity.mark();
                    let seq = u16::from_be_bytes([buf[0], buf[1]]);
                    let opus_data = &buf[2..len];

                    let mut pcm = vec![0i16; 960];
                    let mut dec = self.decoder.lock().await;
                    if let Ok(decoded) = dec.decode(opus_data, &mut pcm) {
                        pcm.truncate(decoded);
                        drop(dec);

                        let mut jit = self.jitter.lock().await;
                        jit.push(seq, pcm);

                        if let Some(frame) = jit.pop() {
                            // Full channel just means the device buffer is ahead;
                            // dropping a frame beats blocking the receive loop.
                            let _ = self.frame_tx.try_send(frame);
                        }
                    }
                }
                Err(e) => {
                    log::error!("UDP receive error: {}", e);
                }
            }
        }
    }
}
