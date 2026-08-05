use std::num::NonZero;

use anyhow::Result;
use comms::{Comms, topics::SpeakerAudio};
use rodio::{Player, buffer::SamplesBuffer};

pub async fn run(comms: Comms) -> Result<()> {
    let subscriber = comms.subscriber::<SpeakerAudio>().await?;

    let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = Player::connect_new(&sink.mixer());

    loop {
        let frame = match subscriber.recv().await {
            Ok(frame) => frame,
            Err(comms::Error::InvalidPayload { .. }) => continue,
            Err(error) => return Err(error.into()),
        };
        if frame.samples.is_empty() {
            continue;
        }
        let source = SamplesBuffer::new(
            NonZero::new(comms::AUDIO_CHANNELS).unwrap(),
            NonZero::new(comms::AUDIO_SAMPLE_RATE).unwrap(),
            frame.samples,
        );
        player.append(source);
    }
}
