use anyhow::Result;
use argos::comms;

mod mic;
mod motors;
mod speaker;
mod video;

#[tokio::main]
async fn main() -> Result<()> {
    let session = comms::open_session().await;

    // One independent task per topic; the Zenoh session is cheaply cloneable.
    let handles = vec![
        tokio::spawn(video::run(session.clone())),
        tokio::spawn(mic::run(session.clone())),
        tokio::spawn(speaker::run(session.clone())),
        tokio::spawn(motors::run(session.clone())),
    ];

    for h in handles {
        if let Err(e) = h.await? {
            eprintln!("task error: {e}");
        }
    }
    Ok(())
}
