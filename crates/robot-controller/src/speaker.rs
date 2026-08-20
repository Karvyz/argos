use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use comms::{Comms, topics::SpeakerAudio};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tracing::{debug, error, info, warn};

pub async fn run(comms: Comms) -> Result<()> {
    let subscriber = comms
        .subscriber::<SpeakerAudio>()
        .await
        .inspect_err(|error| error!("failed to create speaker subscriber: {:?}", error))?;

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("no output device available"))?;
    let supported_config = device
        .default_output_config()
        .inspect_err(|error| error!("failed to get speaker output config: {:?}", error))?;
    if supported_config.sample_format() != cpal::SampleFormat::F32 {
        return Err(anyhow::anyhow!(
            "unsupported output sample format: {:?}; expected f32",
            supported_config.sample_format()
        ));
    }

    let output_config: cpal::StreamConfig = supported_config.into();
    let output_sample_rate = output_config.sample_rate;
    if output_config.channels != 1 {
        return Err(anyhow::anyhow!(
            "unsupported output channel count: {}; expected mono",
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
                for sample in data {
                    *sample = queued.pop_front().unwrap_or(0.0);
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

        if frame.channels != 1 {
            warn!(
                "unsupported speaker frame channel count: {}; expected mono",
                frame.channels
            );
            continue;
        }
        if frame.sample_rate != output_sample_rate {
            warn!(
                "speaker frame sample rate {} does not match output device sample rate {}",
                frame.sample_rate, output_sample_rate
            );
            continue;
        }

        info!(
            "queueing speaker audio: samples={}, channels={}, sample_rate={}",
            frame.samples.len(),
            frame.channels,
            frame.sample_rate,
        );

        samples.lock().unwrap().extend(frame.samples);
    }
}
