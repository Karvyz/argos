use anyhow::Result;
use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};

mod argos;
mod core;
mod model;

use crate::{argos::Action, core::Core};
use argos::Argos;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(10);
    let comms = comms::Comms::open().await?;
    let mut argos = Argos::new(rx, comms.clone()).await;
    let x = init_core("http://192.168.1.201:8080".to_string(), tx, comms).await;
    argos.run_ms_async().await;
    x.await?;
    println!("Goodbye");
    Ok(())
}

async fn init_core(url: String, tx: Sender<Action>, comms: comms::Comms) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut core = Core::new(&url, tx, comms).await;
        core.run().await
    })
}
