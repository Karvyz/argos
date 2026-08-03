use anyhow::Result;
use argos::comms;
use rodio::{Player, buffer::SamplesBuffer, nz};
use zenoh::Session;

pub async fn run(session: Session) -> Result<()> {
    let subscriber = session
        .declare_subscriber(comms::keys::SPEAKER)
        .await
        .map_err(anyhow::Error::msg)?;

    let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = Player::connect_new(&sink.mixer());

    while let Ok(sample) = subscriber.recv_async().await {
        let samples = comms::audio::decode(sample.payload());
        if samples.is_empty() {
            continue;
        }
        let source = SamplesBuffer::new(nz!(1), nz!(24000), samples);
        player.append(source);
    }
    Ok(())
}
