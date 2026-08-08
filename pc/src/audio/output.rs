use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::Arc;

pub struct AudioOutput {
    _stream: cpal::Stream,
    sender: Sender<Vec<i16>>,
}

impl AudioOutput {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("No default output device")?;
        let config = device.default_output_config()?;

        let (sender, receiver): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = bounded(10);
        let receiver = Arc::new(std::sync::Mutex::new(receiver));

        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), receiver.clone())?,
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), receiver.clone())?,
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        Ok(Self {
            _stream: stream,
            sender,
        })
    }

    fn build_stream<T: cpal::SizedSample + From<i16> + Send + 'static>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        receiver: Arc<std::sync::Mutex<Receiver<Vec<i16>>>>,
    ) -> Result<cpal::Stream, cpal::BuildStreamError> {
        let channels = config.channels as usize;
        device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let rx = receiver.lock().unwrap();
                if let Ok(frame) = rx.try_recv() {
                    for (i, sample) in data.iter_mut().enumerate() {
                        let frame_idx = i / channels;
                        *sample = if frame_idx < frame.len() {
                            T::from(frame[frame_idx])
                        } else {
                            T::from(0i16)
                        };
                    }
                } else {
                    for sample in data.iter_mut() {
                        *sample = T::from(0i16);
                    }
                }
            },
            move |err| log::error!("Audio output error: {}", err),
            None,
        )
    }

    pub fn send_frame(&self, frame: Vec<i16>) -> Result<(), crossbeam_channel::TrySendError<Vec<i16>>> {
        self.sender.try_send(frame)
    }
}
