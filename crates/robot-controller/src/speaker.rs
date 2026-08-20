use std::num::NonZero;

use anyhow::Result;
use comms::{Comms, topics::SpeakerAudio};
use rodio::{Player, buffer::SamplesBuffer};
use tracing::{debug, error, info};

pub async fn run(comms: Comms) -> Result<()> {
    let subscriber = comms
        .subscriber::<SpeakerAudio>()
        .await
        .inspect_err(|error| error!(error = ?error, "failed to create speaker subscriber"))?;

    let sink = rodio::DeviceSinkBuilder::open_default_sink()
        .inspect_err(|error| error!(error = ?error, "failed to open speaker output"))?;

    let player = Player::connect_new(&sink.mixer());
    info!("speaker ready");

    loop {
        let frame = match subscriber.recv().await {
            Ok(frame) => frame,
            Err(comms::Error::InvalidPayload { .. }) => {
                debug!("ignoring invalid speaker payload");
                continue;
            }
            Err(error) => {
                error!(error = ?error, "failed to receive speaker audio");
                return Err(error.into());
            }
        };

        if frame.samples.is_empty() {
            debug!("ignoring empty speaker frame");
            continue;
        }

        info!(
            samples = frame.samples.len(),
            channels = frame.channels,
            sample_rate = frame.sample_rate,
            "queueing speaker audio"
        );

        let source = SamplesBuffer::new(
            NonZero::new(frame.channels).unwrap(),
            NonZero::new(frame.sample_rate).unwrap(),
            frame.samples,
        );

        player.append(source);
    }
}
