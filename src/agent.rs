use anyhow::Result;
use serde_json::{Value, json};

use crate::llm::LLMClient;
use crate::tools::ToolRegistry;

pub struct Agent {
    llm: LLMClient,
    tools: ToolRegistry,
    system: String,
    messages: Vec<Value>,
}

impl Agent {
    pub fn new(llm: LLMClient, tools: ToolRegistry, system: impl Into<String>) -> Self {
        Self {
            llm,
            tools,
            system: system.into(),
            messages: Vec::new(),
        }
    }

    pub async fn run(&mut self, user_input: &str) -> Result<String> {
        let tool_defs = self.tools.to_anthropic_tools();
        // let mut messages: Vec<Value> = vec![json!({ "role": "user", "content": user_input })];

        self.messages
            .push(json!({"role": "user", "content": user_input}));
        loop {
            let response = self
                .llm
                .chat(&self.system, &self.messages, &tool_defs)
                .await?;

            let stop_reason = response["stop_reason"].as_str().unwrap_or("");
            let content = response["content"].clone();

            self.messages
                .push(json!({"role": "assistant", "content": content}));

            if stop_reason != "tool_use" {
                let text = content
                    .as_array()
                    .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text"))
                    .and_then(|b| b["text"].as_str())
                    .unwrap_or("")
                    .to_string();
                return Ok(text);
            }

            let tool_results: Vec<Value> = {
                let calls: Vec<&Value> = content
                    .as_array()
                    .map(|blocks| blocks.iter().filter(|b| b["type"] == "tool_use").collect())
                    .unwrap_or_default();

                let mut results = Vec::new();
                for call in calls {
                    let id = call["id"].as_str().unwrap_or("");
                    let name = call["name"].as_str().unwrap_or("");
                    let args = call["input"].clone();

                    eprintln!(
                        "→ {name}: {}",
                        serde_json::to_string(&call["input"]).unwrap_or_default()
                    );
                    let output = match self.tools.dispatch(name, args).await {
                        Ok(r) => r,
                        Err(e) => format!("error: {e}"),
                    };

                    results.push(json!({
                        "type": "tool_result",
                        "tool_use_id": id,
                        "content": output,
                    }));
                }

                results
            };

            self.messages
                .push(json!({"role": "user", "content": tool_results}));
        }
    }
}
