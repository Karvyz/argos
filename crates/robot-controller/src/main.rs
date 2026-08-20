use anyhow::Result;
use std::sync::Arc;
use tracing::error;

mod gate;
mod mic;
mod motors;
mod speaker;
mod video;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,zenoh=warn")),
        )
        .init();

    let comms = comms::Comms::open()
        .await
        .inspect_err(|error| error!("failed to open communications: {:?}", error))?;

    // Shared gate lets the speaker task mute the mic while it emits sound.
    let gate = Arc::new(gate::SpeakerGate::new());

    // One independent task per topic; the comms handle is cheaply cloneable.
    let handles = vec![
        tokio::spawn(video::run(comms.clone())),
        tokio::spawn(mic::run(comms.clone(), gate.clone())),
        tokio::spawn(speaker::run(comms.clone(), gate)),
        tokio::spawn(motors::run(comms)),
    ];

    for h in handles {
        if let Err(error) = h
            .await
            .inspect_err(|error| error!("worker task panicked: {:?}", error))?
        {
            error!("worker task failed: {}", error);
        }
    }
    Ok(())
}
