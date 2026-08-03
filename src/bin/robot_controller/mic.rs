use anyhow::Result;
use zenoh::Session;

/// Mic capture is not implemented yet (no driver wired). Placeholder task so the
/// `robot/mic` topic slot exists and the controller keeps its one-task-per-topic shape.
pub async fn run(_session: Session) -> Result<()> {
    eprintln!("mic: capture driver not implemented yet");
    Ok(())
}
