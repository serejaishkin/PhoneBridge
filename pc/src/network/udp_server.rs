use crate::audio::{decoder::OpusDecoder, jitter_buffer::JitterBuffer, output::AudioOutput};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

pub struct UdpServer {
    socket: UdpSocket,
    jitter: Arc<Mutex<JitterBuffer>>,
    decoder: Arc<Mutex<OpusDecoder>>,
    audio_out: Arc<Mutex<AudioOutput>>,
}

impl UdpServer {
    pub async fn new(
        bind_addr: &str,
        jitter: Arc<Mutex<JitterBuffer>>,
        decoder: OpusDecoder,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(bind_addr).await?;
        let audio_out = AudioOutput::new()?;
        Ok(Self {
            socket,
            jitter,
            decoder: Arc::new(Mutex::new(decoder)),
            audio_out: Arc::new(Mutex::new(audio_out)),
        })
    }

    pub async fn run(&self) {
        let mut buf = vec![0u8; 1500];
        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, _addr)) => {
                    if len < 4 {
                        continue;
                    }
                    let seq = u16::from_be_bytes([buf[0], buf[1]]);
                    let opus_data = &buf[2..len];

                    let mut pcm = vec![0i16; 960];
                    let mut dec = self.decoder.lock().await;
                    if let Ok(decoded) = dec.decode(opus_data, &mut pcm) {
                        pcm.truncate(decoded);
                        let mut jit = self.jitter.lock().await;
                        jit.push(seq, pcm);

                        if let Some(frame) = jit.pop() {
                            let out = self.audio_out.lock().await;
                            let _ = out.send_frame(frame);
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
