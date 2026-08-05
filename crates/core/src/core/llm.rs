use futures::StreamExt;
use rig_core::{
    agent::{Agent, MultiTurnStreamItem, Text},
    client::CompletionClient,
    memory::InMemoryConversationMemory,
    providers::llamafile::{Client, CompletionModel, LLAMA_CPP},
    streaming::{StreamedAssistantContent, StreamingPrompt},
    tool::Tool,
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc::{Sender, error::SendError};
use zenoh::Session;

use crate::{argos::Action, core::tts::TTS};

const SYSTEM_PROMPT: &str = "You are Argos, the robot dog. Act like a sort of Jarvis. Use the TTS tool as your only way to communicate.";

pub struct LLM {
    client: Client,
    agent: Agent<CompletionModel>,
    tx: Sender<Action>,
}

impl LLM {
    pub async fn new(url: &str, tx: Sender<Action>, session: Session) -> Self {
        let client = Client::from_url(url).unwrap();
        let agent = Self::agent(&client, session).await;
        Self { client, agent, tx }
    }

    async fn agent(client: &Client, session: Session) -> Agent<CompletionModel> {
        let memory = InMemoryConversationMemory::new();
        client
            .agent(LLAMA_CPP)
            .memory(memory)
            .preamble(SYSTEM_PROMPT)
            .tools(vec![Box::new(ToolTTS::new(TTS::run(session).await))])
            .default_max_turns(10)
            .build()
    }

    pub async fn ask(&self, prompt: &str) {
        let mut stream = self.agent.stream_prompt(prompt).conversation("conv").await;
        while let Some(content) = stream.next().await {
            match content {
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(
                    Text { text, .. },
                ))) => {
                    print!("{text}");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Reasoning(reasoning),
                )) => {
                    let reasoning = reasoning.display_text();
                    print!("{reasoning}");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
                Ok(MultiTurnStreamItem::FinalResponse(_)) => println!(),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
                _ => {}
            };
        }
    }
}

#[derive(Deserialize)]
struct TTSArgs {
    text: String,
}

struct ToolTTS {
    tts_tx: Sender<String>,
}

impl ToolTTS {
    pub fn new(tx: Sender<String>) -> Self {
        ToolTTS { tts_tx: tx }
    }

    async fn send_to_tts(&self, args: TTSArgs) -> Result<String, SendError<String>> {
        println!("TTS call: {}", args.text);
        self.tts_tx.send(args.text).await?;
        Ok("Ok".to_string())
    }
}

impl Tool for ToolTTS {
    const NAME: &'static str = "tts";
    type Error = SendError<String>;
    type Args = TTSArgs;
    type Output = String;

    fn description(&self) -> String {
        "Speaks the given text".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The text to speak"
                }
            },
            "required": ["text"],
        })
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + rig_core::wasm_compat::WasmCompatSend
    {
        self.send_to_tts(args)
    }
}
