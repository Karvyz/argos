use futures::StreamExt;
use rig_core::{
    agent::{Agent, MultiTurnStreamItem, Text},
    client::CompletionClient,
    memory::InMemoryConversationMemory,
    providers::llamafile::{Client, CompletionModel, LLAMA_CPP},
    streaming::{StreamedAssistantContent, StreamingPrompt},
};
use tokio::sync::mpsc::Sender;

use crate::argos::Action;

pub struct Core {
    client: Client,
    agent: Agent<CompletionModel>,
    tx: Sender<Action>,
}

impl Core {
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

    pub async fn exit(&mut self) {
        self.tx.send(Action::Exit).await.unwrap();
    }
}
