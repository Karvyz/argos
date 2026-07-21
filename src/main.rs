use anyhow::Result;

mod argos;
mod coord;
mod model;

use argos::Argos;

#[tokio::main]
async fn main() -> Result<()> {
    let mut argos = Argos::new();
    argos.run_ms_async().await;
    Ok(())
}
