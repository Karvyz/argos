use anyhow::Result;
use comms::{AudioFrame, Comms, topics::Voice};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use earshot::Detector;
use std::sync::mpsc;
use tracing::{debug, error, info};

const TARGET_SAMPLE_RATE: u32 = 16_000;
const CHUNK_SIZE: usize = 256;
const THRESHOLD: f32 = 0.5;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("No input device available")]
    NoInputDevice,
    #[error("No supported input config on the default device")]
    NoSupportedConfig(#[source] cpal::Error),
    #[error("Failed to build input stream: {0}")]
    BuildStream(String),
    #[error("Failed to play input stream: {0}")]
    PlayStream(String),
    #[error("Audio capture channel closed unexpectedly")]
    ChannelClosed,
}

pub struct AudioStream {
    #[allow(dead_code)]
    stream: cpal::Stream,
    rx: mpsc::Receiver<Vec<f32>>,
    buffer: Vec<f32>,
    resample_ratio: f32,
}

impl AudioStream {
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoInputDevice)?;
        let config = device
            .default_input_config()
            .map_err(AudioError::NoSupportedConfig)?;

        if config.sample_format() != cpal::SampleFormat::F32 {
            return Err(AudioError::BuildStream(format!(
                "unsupported sample format: {:?}; expected f32",
                config.sample_format()
            )));
        }

        let device_sample_rate = config.sample_rate();
        let num_channels = config.channels() as usize;
        let resample_ratio = device_sample_rate as f32 / TARGET_SAMPLE_RATE as f32;
        let (tx, rx) = mpsc::channel::<Vec<f32>>();
        let err_fn = |err| error!("audio capture stream error: {}", err);
        let stream = device
            .build_input_stream::<f32, _, _>(
                config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = collapse_channels(data, num_channels);
                    if let Err(error) = tx.send(mono) {
                        debug!("audio capture receiver dropped: {:?}", error);
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| AudioError::BuildStream(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::PlayStream(e.to_string()))?;

        Ok(Self {
            stream,
            rx,
            buffer: Vec::with_capacity(CHUNK_SIZE * 4),
            resample_ratio,
        })
    }

    pub fn next_chunk(&mut self) -> Result<Vec<f32>, AudioError> {
        loop {
            while let Ok(samples) = self.rx.try_recv() {
                self.buffer.extend(samples);
            }

            let needed_input = (CHUNK_SIZE as f32 * self.resample_ratio).ceil() as usize;
            if self.buffer.len() >= needed_input {
                return Ok(self.resample_chunk(CHUNK_SIZE));
            }

            let samples = self.rx.recv().map_err(|_| AudioError::ChannelClosed)?;
            self.buffer.extend(samples);
        }
    }

    fn resample_chunk(&mut self, output_size: usize) -> Vec<f32> {
        let mut out = Vec::with_capacity(output_size);

        if (self.resample_ratio - 1.0).abs() < f32::EPSILON {
            out.extend(self.buffer.drain(..output_size.min(self.buffer.len())));
            return out;
        }

        for i in 0..output_size {
            let pos = i as f32 * self.resample_ratio;
            let idx = pos as usize;
            let frac = pos - idx as f32;
            let sample = if idx + 1 < self.buffer.len() {
                let a = self.buffer[idx];
                let b = self.buffer[idx + 1];
                a + (b - a) * frac
            } else {
                self.buffer[idx]
            };
            out.push(sample);
        }

        let consumed = (output_size as f32 * self.resample_ratio).ceil() as usize;
        self.buffer.drain(..consumed.min(self.buffer.len()));
        out
    }
}

fn collapse_channels(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        return interleaved.to_vec();
    }

    let frames = interleaved.len() / channels;
    let mut mono = Vec::with_capacity(frames);
    for frame in 0..frames {
        let start = frame * channels;
        let sum: f32 = interleaved[start..start + channels].iter().sum();
        mono.push(sum / channels as f32);
    }
    mono
}

pub async fn run(comms: Comms) -> Result<()> {
    let publisher = comms
        .publisher::<Voice>()
        .await
        .inspect_err(|error| error!("failed to create voice publisher: {:?}", error))?;
    let mut stream = AudioStream::new()
        .inspect_err(|error| error!("failed to create audio stream: {:?}", error))?;

    info!("listening for voice messages at {} Hz", TARGET_SAMPLE_RATE);
    let mut detector = Detector::default();
    let mut voice_message = Vec::new();
    let mut chunk_count = 0;
    loop {
        let chunk = stream
            .next_chunk()
            .inspect_err(|error| error!("failed to read audio chunk: {:?}", error))?;
        assert_eq!(chunk.len(), 256);

        let score = detector.predict_f32(&chunk);
        // Score is between 0-1; 0 = no voice, 1 = voice.
        let is_voice = score >= THRESHOLD;
        let rms = (chunk.iter().map(|&s| s.powi(2)).sum::<f32>() / chunk.len() as f32).sqrt();
        debug!(
            "processed audio chunk: chunk={}, len={}, rms={}, first={:?}, last={:?}, voice={}",
            chunk_count + 1,
            chunk.len(),
            rms,
            chunk.first().unwrap(),
            chunk.last().unwrap(),
            is_voice,
        );

        if is_voice {
            voice_message.extend(chunk);
        } else if !voice_message.is_empty() {
            let samples = std::mem::take(&mut voice_message);
            info!("sending voice message with {} samples", samples.len());
            publisher
                .send(AudioFrame {
                    sample_rate: TARGET_SAMPLE_RATE,
                    channels: 1,
                    samples,
                })
                .await
                .inspect_err(|error| error!("failed to send voice message: {:?}", error))?;
        }

        chunk_count += 1;
    }
}
