use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use comms::{Comms, topics::SpeakerAudio};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tracing::{debug, error, info, warn};

use crate::gate::SpeakerGate;

pub async fn run(comms: Comms, gate: Arc<SpeakerGate>) -> Result<()> {
    let subscriber = comms
        .subscriber::<SpeakerAudio>()
        .await
        .inspect_err(|error| error!("failed to create speaker subscriber: {:?}", error))?;

    let host = cpal::default_host();
    let device = host.default_output_device();
    if device.is_none() {
        error!("no output device available");
        return Err(anyhow::anyhow!("no output device available"));
    }
    let device = device.unwrap();
    let supported_config = device
        .default_output_config()
        .inspect_err(|error| error!("failed to get speaker output config: {:?}", error))?;
    if supported_config.sample_format() != cpal::SampleFormat::F32 {
        error!(
            "unsupported output sample format: {:?}; expected f32",
            supported_config.sample_format()
        );
        return Err(anyhow::anyhow!(
            "unsupported output sample format: {:?}; expected f32",
            supported_config.sample_format()
        ));
    }

    let output_config: cpal::StreamConfig = supported_config.into();
    let output_sample_rate = output_config.sample_rate;
    if output_config.channels != 2 {
        error!(
            "unsupported output channel count: {}; expected stereo",
            output_config.channels
        );
        return Err(anyhow::anyhow!(
            "unsupported output channel count: {}; expected stereo",
            output_config.channels
        ));
    }

    let samples = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let callback_samples = Arc::clone(&samples);
    let stream = device
        .build_output_stream::<f32, _, _>(
            output_config.clone(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut queued = callback_samples.lock().unwrap();

                let mut emitted = false;
                for sample in data {
                    *sample = queued.pop_front().unwrap_or(0.0);
                    emitted = emitted || *sample != 0.0;
                }
                if emitted {
                    gate.report_playing();
                }
            },
            |error| error!("speaker output stream error: {}", error),
            None,
        )
        .inspect_err(|error| error!("failed to build speaker output stream: {:?}", error))?;
    stream
        .play()
        .inspect_err(|error| error!("failed to start speaker output stream: {:?}", error))?;

    info!("speaker ready");

    loop {
        let frame = match subscriber.recv().await {
            Ok(frame) => frame,
            Err(comms::Error::InvalidPayload { .. }) => {
                debug!("ignoring invalid speaker payload");
                continue;
            }
            Err(error) => {
                error!("failed to receive speaker audio: {:?}", error);
                return Err(error.into());
            }
        };

        if frame.samples.is_empty() {
            warn!("ignoring empty speaker frame");
            continue;
        }

        // Output is stereo, so accept mono or stereo input and convert as needed.
        let mut pending = frame.samples;
        if frame.channels == 1 {
            // Convert mono samples into interleaved stereo by duplicating each sample.
            let mut stereo = Vec::<f32>::with_capacity(pending.len() * 2);
            for sample in pending {
                stereo.push(sample);
                stereo.push(sample);
            }
            pending = stereo;
        } else if frame.channels != 2 {
            warn!(
                "unsupported speaker frame channel count: {}; expected mono or stereo",
                frame.channels
            );
            continue;
        }
        if frame.sample_rate != output_sample_rate {
            pending = resample(pending, 2, frame.sample_rate, output_sample_rate);
        }

        info!(
            "queueing speaker audio: samples={}, channels={}, sample_rate={}",
            pending.len(),
            frame.channels,
            frame.sample_rate,
        );

        samples.lock().unwrap().extend(pending);
    }
}

/// Resample interleaved audio from one sample rate to another using linear
/// interpolation between frames. Each frame holds `channels` interleaved samples.
fn resample(interleaved: Vec<f32>, channels: usize, from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return interleaved;
    }

    let ratio = to_rate as f32 / from_rate as f32;
    let frames_in = interleaved.len() / channels;
    let frames_out = (frames_in as f32 * ratio).ceil() as usize;
    let mut out = Vec::<f32>::with_capacity(frames_out * channels);

    for frame in 0..frames_out {
        let pos = frame as f32 / ratio;
        let idx = pos as usize;
        let frac = pos - idx as f32;
        for ch in 0..channels {
            let a = interleaved[idx * channels + ch];
            let b = if idx + 1 < frames_in {
                interleaved[(idx + 1) * channels + ch]
            } else {
                a
            };
            out.push(a + (b - a) * frac);
        }
    }
    out
}
