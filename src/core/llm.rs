use futures::StreamExt;
use rig_core::{
    agent::{Agent, MultiTurnStreamItem, Text},
    client::CompletionClient,
    memory::InMemoryConversationMemory,
    providers::llamafile::{Client, CompletionModel, LLAMA_CPP},
    streaming::{StreamedAssistantContent, StreamingPrompt},
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::argos::Action;

pub struct LLM {
    client: Client,
    agent: Agent<CompletionModel>,
    tx: Sender<Action>,
}

impl LLM {
    pub fn new(url: &str, tx: Sender<Action>) -> Self {
        let client = Client::from_url(url).unwrap();
        let agent = Self::agent(&client);
        Self { client, agent, tx }
    }

    pub fn new_agent(&mut self) {
        self.agent = Self::agent(&self.client)
    }

    fn agent(client: &Client) -> Agent<CompletionModel> {
        let memory = InMemoryConversationMemory::new();
        client
            .agent(LLAMA_CPP)
            .memory(memory)
            .preamble("You are Argos, my faithfull robot dog.")
            // .tools()
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

#[derive(Deserialize, Serialize)]
struct ToolTTS;

impl Tool for ToolTTS {
    const NAME: &'static str = "tts";
    type Error = std::io::Error;
    type Args = String;
    type Output = ();

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
        test()
    }
}

async fn test() -> Result<(), std::io::Error> {
    todo!()
}
