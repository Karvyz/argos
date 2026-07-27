use colored::Colorize;
use rustyline::{Config, error::ReadlineError};
use tokio::sync::mpsc::Sender;

use crate::{argos::Action, core::llm::LLM};

mod llm;
mod tts;

enum CmdRes {
    Ok(String),
    None,
    Exit,
}

pub struct Core {
    llm: LLM,
    tx: Sender<Action>,
}

impl Core {
    pub async fn new(url: &str, tx: Sender<Action>) -> Self {
        Core {
            llm: LLM::new(url, tx.clone()).await,
            tx,
        }
    }

    pub async fn run(&mut self) {
        loop {
            match Self::parse().await {
                CmdRes::Ok(s) => self.llm.ask(&s).await,
                CmdRes::None => (),
                CmdRes::Exit => {
                    self.tx.send(Action::Exit).await.unwrap();
                    break;
                }
            }
        }
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
}
