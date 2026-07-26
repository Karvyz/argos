use anyhow::Result;

mod argos;
mod core;
mod model;

use argos::Argos;
use colored::Colorize;
use rustyline::{Config, error::ReadlineError};
use tokio::sync::mpsc::{self, Sender};

use crate::{argos::Action, core::Core};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(10);
    let mut argos = Argos::new(rx).await;
    let core = Core::new("http://192.168.1.201:8080", tx);
    let x = tokio::spawn(ads(core));
    argos.run_ms_async().await;
    x.await?;
    println!("Goodbye");
    Ok(())
}

async fn ads(mut core: Core) {
    loop {
        match parse().await {
            CmdRes::Ok(s) => core.ask(&s).await,
            CmdRes::None => (),
            CmdRes::Exit => {
                core.exit().await;
                break;
            }
        }
    }
}

enum CmdRes {
    Ok(String),
    None,
    Exit,
}

async fn parse() -> CmdRes {
    let config = Config::builder().edit_mode(rustyline::EditMode::Vi).build();
    let mut rl = rustyline::DefaultEditor::with_config(config).unwrap();
    let readline = rl.readline(&format!("{} ", ">>".cyan().bold()));
    match readline {
        Ok(line) => CmdRes::Ok(line),
        Err(ReadlineError::Interrupted) => CmdRes::None,
        Err(ReadlineError::Eof) => CmdRes::Exit,
        Err(err) => {
            println!("Error: {:?}", err);
            CmdRes::None
        }
    }
}
