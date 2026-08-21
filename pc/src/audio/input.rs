use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use crossbeam_channel::{bounded, Receiver, Sender};
use opus::Encoder;
use std::sync::Arc;

pub struct AudioInput {
    _stream: cpal::Stream,
    receiver: Receiver<Vec<u8>>,
    encoder: Arc<parking_lot::Mutex<Encoder>>,
}

impl AudioInput {
    pub fn new() -> Result<(Self, Receiver<Vec<u8>>), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No default input device")?;
        let config = device.default_input_config()?;

        let encoder = Encoder::new(48000, opus::Channels::Mono, opus::Application::Voip)?;
        let encoder = Arc::new(parking_lot::Mutex::new(encoder));

        let (pcm_tx, pcm_rx): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = bounded(32);
        let (opus_tx, opus_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = bounded(32);

        let enc_clone = encoder.clone();
        std::thread::spawn(move || {
            while let Ok(frame) = pcm_rx.recv() {
                let mut out = vec![0u8; 1500];
                let mut enc = enc_clone.lock();
                if let Ok(len) = enc.encode(&frame, &mut out) {
                    out.truncate(len);
                    let _ = opus_tx.try_send(out);
                }
            }
        });

        let pcm_tx_clone = pcm_tx.clone();
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => {
                Self::build_stream::<i16>(&device, &config.clone().into(), pcm_tx_clone)?
            }
            cpal::SampleFormat::F32 => {
                Self::build_stream::<f32>(&device, &config.clone().into(), pcm_tx_clone)?
            }
            cpal::SampleFormat::I32 => {
                Self::build_stream::<i32>(&device, &config.clone().into(), pcm_tx_clone)?
            }
            _ => return Err("Unsupported sample format".into()),
        };
        stream.play()?;

        Ok((
            Self {
                _stream: stream,
                receiver: opus_rx.clone(),
                encoder,
            },
            opus_rx,
        ))
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        pcm_tx: Sender<Vec<i16>>,
    ) -> Result<cpal::Stream, cpal::BuildStreamError>
    where
        T: Sample + cpal::SizedSample + Send + 'static,
        i16: FromSample<T>,
    {
        let channels = config.channels as usize;
        device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut mono = Vec::with_capacity(data.len() / channels);
                for chunk in data.chunks(channels) {
                    let sum: i32 = chunk
                        .iter()
                        .map(|sample| i16::from_sample(*sample) as i32)
                        .sum();
                    mono.push((sum / channels as i32) as i16);
                }
                // Buffer 20ms frames (960 samples @ 48kHz).
                // For simplicity we send every callback chunk; a production path
                // should add a fixed-size jitter/framing buffer before Opus encoding.
                let _ = pcm_tx.try_send(mono);
            },
            move |err| log::error!("Audio input error: {}", err),
            None,
        )
    }

    pub fn recv_opus(&self) -> Result<Vec<u8>, crossbeam_channel::TryRecvError> {
        self.receiver.try_recv()
    }
}
