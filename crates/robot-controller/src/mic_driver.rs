use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const CHUNK_SIZE: usize = 256;

/// Default file name used when saving captured audio to disk.
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

/// A stream of 16 kHz mono audio chunks captured from the default microphone.
///
/// Each call to [`next_chunk`](Self::next_chunk) returns a `Vec<i16>` containing
/// exactly 256 samples, which represents 16 ms of audio at the target rate.
///
/// The struct handles sample format conversion (f32/u16 → i16), channel
/// down-mixing to mono, and resampling to 16 kHz automatically.
pub struct AudioStream {
    /// Kept alive for the entire lifetime of `AudioStream` – dropping it stops
    /// the underlying audio capture.
    #[allow(dead_code)]
    stream: cpal::Stream,
    /// Receiver for raw audio buffers sent from the cpal callback thread.
    rx: mpsc::Receiver<Vec<i16>>,
    /// Internal ring buffer of raw (device-rate) mono i16 samples.
    buffer: Vec<i16>,
    /// Ratio = device_sample_rate / target_sample_rate. Used for resampling.
    resample_ratio: f64,
}

impl AudioStream {
    /// Initialises the default microphone and starts capturing audio.
    ///
    /// The device is queried for its default input configuration. If the default
    /// config's sample rate is not 16 kHz the stream will automatically resample
    /// using linear interpolation.
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoInputDevice)?;

        let config = device
            .default_input_config()
            .map_err(AudioError::NoSupportedConfig)?;

        let device_sample_rate = config.sample_rate();
        let num_channels = config.channels() as usize;
        let resample_ratio = device_sample_rate as f64 / TARGET_SAMPLE_RATE as f64;

        let (tx, rx) = mpsc::channel::<Vec<i16>>();

        let err_fn = |err| eprintln!("[audio] capture stream error: {err}");

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream::<f32, _, _>(
                config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = collapse_channels_f32(data, num_channels);
                    let _ = tx.send(mono);
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream::<i16, _, _>(
                config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mono = collapse_channels_i16(data, num_channels);
                    let _ = tx.send(mono);
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream::<u16, _, _>(
                config.into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let mono = collapse_channels_u16(data, num_channels);
                    let _ = tx.send(mono);
                },
                err_fn,
                None,
            ),
            other => {
                return Err(AudioError::BuildStream(format!(
                    "unsupported sample format: {other:?}"
                )));
            }
        }
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

    /// Blocks until a full 256-sample chunk at 16 kHz is available.
    ///
    /// The returned vector always contains exactly [`CHUNK_SIZE`] (256) `i16`
    /// samples.
    pub fn next_chunk(&mut self) -> Result<Vec<i16>, AudioError> {
        loop {
            // Drain as many pending buffers from the channel as possible.
            while let Ok(samples) = self.rx.try_recv() {
                self.buffer.extend(samples);
            }

            // How many raw (device-rate) samples are needed to produce one
            // output chunk of `CHUNK_SIZE` samples at the target rate?
            let needed_input = (CHUNK_SIZE as f64 * self.resample_ratio).ceil() as usize;

            if self.buffer.len() >= needed_input {
                return Ok(self.resample_chunk(CHUNK_SIZE));
            }

            // Block until more data arrives.
            let samples = self.rx.recv().map_err(|_| AudioError::ChannelClosed)?;
            self.buffer.extend(samples);
        }
    }

    // ── helpers ──────────────────────────────────────────────────────────

    /// Resample (or pass-through) the oldest samples in `self.buffer` to
    /// produce a chunk of `output_size` samples at 16 kHz.
    ///
    /// Uses linear interpolation for fractional resampling ratios and
    /// simple decimation for integer ratios.
    fn resample_chunk(&mut self, output_size: usize) -> Vec<i16> {
        let mut out = Vec::with_capacity(output_size);

        if (self.resample_ratio - 1.0).abs() < f64::EPSILON {
            // No resampling needed – just copy the first `output_size` samples.
            out.extend(self.buffer.drain(..output_size.min(self.buffer.len())));
            return out;
        }

        for i in 0..output_size {
            let pos = i as f64 * self.resample_ratio;
            let idx = pos as usize;
            let frac = pos - idx as f64;

            let sample = if idx + 1 < self.buffer.len() {
                let a = self.buffer[idx] as f32;
                let b = self.buffer[idx + 1] as f32;
                (a + (b - a) * frac as f32) as i16
            } else {
                self.buffer[idx]
            };
            out.push(sample);
        }

        let consumed = (output_size as f64 * self.resample_ratio).ceil() as usize;
        let drain_end = consumed.min(self.buffer.len());
        self.buffer.drain(..drain_end);

        out
    }
}

// ── channel-downmixing helpers ───────────────────────────────────────────

/// Interleave → mono by averaging all channels for each frame, then converting
/// to i16.
fn collapse_channels_f32(interleaved: &[f32], channels: usize) -> Vec<i16> {
    if channels == 1 {
        return interleaved
            .iter()
            .map(|&s| (s * i16::MAX as f32) as i16)
            .collect();
    }
    let frames = interleaved.len() / channels;
    let mut mono = Vec::with_capacity(frames);
    for frame in 0..frames {
        let start = frame * channels;
        let sum: f32 = interleaved[start..start + channels].iter().sum();
        let avg = sum / channels as f32;
        mono.push((avg * i16::MAX as f32) as i16);
    }
    mono
}

fn collapse_channels_i16(interleaved: &[i16], channels: usize) -> Vec<i16> {
    if channels == 1 {
        return interleaved.to_vec();
    }
    let frames = interleaved.len() / channels;
    let mut mono = Vec::with_capacity(frames);
    for frame in 0..frames {
        let start = frame * channels;
        let sum: i32 = interleaved[start..start + channels]
            .iter()
            .map(|&s| s as i32)
            .sum();
        mono.push((sum / channels as i32) as i16);
    }
    mono
}

fn collapse_channels_u16(interleaved: &[u16], channels: usize) -> Vec<i16> {
    if channels == 1 {
        return interleaved
            .iter()
            .map(|&s| (s as i32 - 32_768) as i16)
            .collect();
    }
    let frames = interleaved.len() / channels;
    let mut mono = Vec::with_capacity(frames);
    for frame in 0..frames {
        let start = frame * channels;
        let sum: u32 = interleaved[start..start + channels]
            .iter()
            .map(|&s| s as u32)
            .sum();
        let avg = sum / channels as u32;
        mono.push((avg as i32 - 32_768) as i16);
    }
    mono
}

/// Writes a collection of 16 kHz mono `i16` samples to a WAV file.
///
/// The samples are typically the concatenation of chunks returned by
/// [`next_chunk`](AudioStream::next_chunk). If `path` is `None` the default
/// file name ([`DEFAULT_OUTPUT_FILE`]) is used.
pub fn save_wav(samples: &[i16], path: Option<&str>) -> Result<(), hound::Error> {
    let path = path.unwrap_or(DEFAULT_OUTPUT_FILE);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for &s in samples {
        writer.write_sample(s)?;
    }
    writer.finalize()
}
