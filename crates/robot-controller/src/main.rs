use anyhow::Result;

mod mic;
mod motors;
mod speaker;
mod video;

#[tokio::main]
async fn main() -> Result<()> {
    let comms = comms::Comms::open().await?;

    // One independent task per topic; the comms handle is cheaply cloneable.
    let handles = vec![
        tokio::spawn(video::run(comms.clone())),
        tokio::spawn(mic::run(comms.clone())),
        tokio::spawn(speaker::run(comms.clone())),
        tokio::spawn(motors::run(comms)),
    ];

    for h in handles {
        if let Err(e) = h.await? {
            eprintln!("task error: {e}");
        }
    }
    Ok(())
}
