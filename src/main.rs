use std::{thread::sleep, time::Duration};

use anyhow::Result;

mod argos;
mod model;

use argos::Argos;

#[tokio::main]
async fn main() -> Result<()> {
    let mut argos = Argos::new();
    argos.single();
    sleep(Duration::from_secs(1));
    Ok(())
}
