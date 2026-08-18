use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use anyhow::Result;
use comms::Comms;
use earshot::Detector;
use std::sync::mpsc;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const CHUNK_SIZE: usize = 256;

pub const DEFAULT_OUTPUT_FILE: &str = "mic_capture.wav";

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
        let err_fn = |err| eprintln!("[audio] capture stream error: {err}");
        let stream = device
            .build_input_stream::<f32, _, _>(
                config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = collapse_channels(data, num_channels);
                    let _ = tx.send(mono);
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

pub fn save_wav(samples: &[f32], path: Option<&str>) -> Result<(), hound::Error> {
    let path = path.unwrap_or(DEFAULT_OUTPUT_FILE);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for &sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()
}

/// Mic capture is not implemented yet (no driver wired). Placeholder task so the
/// `robot/mic` topic slot exists and the controller keeps its one-task-per-topic shape.
pub async fn run(_comms: Comms) -> Result<()> {
    let mut stream = AudioStream::new().expect("Fail to capture audio stream");

    println!("Capturing audio — 10 chunks of 256 samples at 16 kHz...");
    let mut detector = Detector::default();
    let mut recording: Vec<f32> = Vec::new();
    let mut i = 0;
    loop {
        let chunk = stream.next_chunk().unwrap();
        assert_eq!(chunk.len(), 256);

        recording.extend(chunk.iter().copied());

        let score = detector.predict_f32(&chunk);
        // Score is between 0-1; 0 = no voice, 1 = voice.
        let voice = match score >= 0.5 {
            true => "voice",
            false => "nothing",
        };
        let rms = (chunk.iter().map(|&s| s.powi(2)).sum::<f32>() / chunk.len() as f32).sqrt();
        println!(
            "chunk {:>2}  |  len={}  |  RMS={:.6}  |  first={:+}  last={:+} | {}",
            i + 1,
            chunk.len(),
            rms,
            chunk.first().unwrap(),
            chunk.last().unwrap(),
            voice,
        );
        i += 1;

        if i >= 1000 {
            break;
        }
    }

    let path = DEFAULT_OUTPUT_FILE;
    save_wav(&recording, Some(path))?;
    println!("Saved {} samples to {}", recording.len(), path);
    Ok(())
}
