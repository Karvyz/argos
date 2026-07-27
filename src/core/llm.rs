use futures::StreamExt;
use rig_core::{
    agent::{Agent, MultiTurnStreamItem, Text},
    client::CompletionClient,
    memory::InMemoryConversationMemory,
    providers::llamafile::{Client, CompletionModel, LLAMA_CPP},
    streaming::{StreamedAssistantContent, StreamingPrompt},
    tool::Tool,
};
use serde_json::json;
use tokio::sync::mpsc::{Sender, error::SendError};

use crate::{argos::Action, core::tts::TTS};

pub struct LLM {
    client: Client,
    agent: Agent<CompletionModel>,
    tx: Sender<Action>,
}

impl LLM {
    pub async fn new(url: &str, tx: Sender<Action>) -> Self {
        let client = Client::from_url(url).unwrap();
        let agent = Self::agent(&client).await;
        Self { client, agent, tx }
    }

    pub async fn new_agent(&mut self) {
        self.agent = Self::agent(&self.client).await
    }

    async fn agent(client: &Client) -> Agent<CompletionModel> {
        let memory = InMemoryConversationMemory::new();
        client
            .agent(LLAMA_CPP)
            .memory(memory)
            .preamble("You are Argos, my faithfull robot dog.")
            .tools(vec![Box::new(ToolTTS::new(TTS::run().await))])
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

struct ToolTTS {
    tts_tx: Sender<String>,
}

impl ToolTTS {
    pub fn new(tx: Sender<String>) -> Self {
        ToolTTS { tts_tx: tx }
    }

    async fn send_to_tts(&self, text: String) -> Result<String, SendError<String>> {
        self.tts_tx.send(text).await?;
        Ok("Ok".to_string())
    }
}

impl Tool for ToolTTS {
    const NAME: &'static str = "tts";
    type Error = SendError<String>;
    type Args = String;
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
