use reqwest::Client;
use serde_json::{json, Value};

pub enum Provider {
    Anthropic,
    OpenAI,
}

impl Provider {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "openai" => Self::OpenAI,
            _        => Self::Anthropic,
        }
    }
}

pub struct LLMClient {
    client:   Client,
    api_key:  String,
    base_url: String,
    pub model: String,
    provider: Provider,
}

impl LLMClient {
    pub fn new(api_key: String, base_url: String, model: String, provider: Provider) -> Self {
        Self { client: Client::new(), api_key, base_url, model, provider }
    }

    // Agent always passes Anthropic-format messages and tools.
    // This method normalises to the right wire format and returns an Anthropic-shaped response.
    pub async fn chat(&self, system: &str, messages: &[Value], tools: &[Value]) -> anyhow::Result<Value> {
        match self.provider {
            Provider::Anthropic => self.chat_anthropic(system, messages, tools).await,
            Provider::OpenAI    => self.chat_openai(system, messages, tools).await,
        }
    }

    async fn chat_anthropic(&self, system: &str, messages: &[Value], tools: &[Value]) -> anyhow::Result<Value> {
        let resp = self.client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model":    &self.model,
                "max_tokens": 4096,
                "system":   system,
                "messages": messages,
                "tools":    tools,
            }))
            .send().await?;

        let status = resp.status();
        let body: Value = resp.json().await?;
        if !status.is_success() {
            let msg = body["error"]["message"].as_str().unwrap_or("unknown error");
            anyhow::bail!("API {status}: {msg}");
        }
        Ok(body)
    }

    async fn chat_openai(&self, system: &str, messages: &[Value], tools: &[Value]) -> anyhow::Result<Value> {
        let oai_tools = Self::anthropic_tools_to_openai(tools);
        let mut oai_messages = vec![json!({ "role": "system", "content": system })];
        oai_messages.extend(Self::anthropic_messages_to_openai(messages));

        let mut payload = json!({ "model": &self.model, "messages": oai_messages });
        if !oai_tools.is_empty() {
            payload["tools"] = json!(oai_tools);
        }

        let resp = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send().await?;

        let status = resp.status();
        let body: Value = resp.json().await?;
        if !status.is_success() {
            let msg = body["error"]["message"].as_str().unwrap_or("unknown error");
            anyhow::bail!("API {status}: {msg}");
        }
        Ok(Self::openai_response_to_anthropic(body))
    }

    // { name, description, input_schema } → { type, function: { name, description, parameters } }
    fn anthropic_tools_to_openai(tools: &[Value]) -> Vec<Value> {
        tools.iter().map(|t| json!({
            "type": "function",
            "function": {
                "name":        t["name"],
                "description": t["description"],
                "parameters":  t["input_schema"],
            }
        })).collect()
    }

    // Convert Anthropic-format message history to OpenAI format.
    fn anthropic_messages_to_openai(messages: &[Value]) -> Vec<Value> {
        let mut out = Vec::new();
        for msg in messages {
            let role    = msg["role"].as_str().unwrap_or("");
            let content = &msg["content"];
            match role {
                "user" => {
                    if content.is_string() {
                        out.push(json!({ "role": "user", "content": content }));
                    } else if let Some(blocks) = content.as_array() {
                        for b in blocks {
                            match b["type"].as_str().unwrap_or("") {
                                "tool_result" => out.push(json!({
                                    "role":         "tool",
                                    "tool_call_id": b["tool_use_id"],
                                    "content":      b["content"],
                                })),
                                "text" => out.push(json!({ "role": "user", "content": b["text"] })),
                                _ => {}
                            }
                        }
                    }
                }
                "assistant" => {
                    if let Some(blocks) = content.as_array() {
                        let text = blocks.iter()
                            .find(|b| b["type"] == "text")
                            .and_then(|b| b["text"].as_str())
                            .unwrap_or("");

                        let tool_calls: Vec<Value> = blocks.iter()
                            .filter(|b| b["type"] == "tool_use")
                            .map(|b| json!({
                                "id":   b["id"],
                                "type": "function",
                                "function": {
                                    "name":      b["name"],
                                    "arguments": serde_json::to_string(&b["input"]).unwrap_or_default(),
                                }
                            }))
                            .collect();

                        if tool_calls.is_empty() {
                            out.push(json!({ "role": "assistant", "content": text }));
                        } else {
                            out.push(json!({
                                "role":       "assistant",
                                "content":    if text.is_empty() { Value::Null } else { json!(text) },
                                "tool_calls": tool_calls,
                            }));
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }

    // Normalise an OpenAI response to the Anthropic shape the agent loop expects.
    fn openai_response_to_anthropic(body: Value) -> Value {
        let choice        = &body["choices"][0];
        let finish_reason = choice["finish_reason"].as_str().unwrap_or("");
        let message       = &choice["message"];

        let mut content = Vec::new();

        if let Some(text) = message["content"].as_str() {
            if !text.is_empty() {
                content.push(json!({ "type": "text", "text": text }));
            }
        }

        if let Some(calls) = message["tool_calls"].as_array() {
            for tc in calls {
                let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                let input: Value = serde_json::from_str(args_str).unwrap_or(json!({}));
                content.push(json!({
                    "type":  "tool_use",
                    "id":    tc["id"],
                    "name":  tc["function"]["name"],
                    "input": input,
                }));
            }
        }

        let stop_reason = if finish_reason == "tool_calls" { "tool_use" } else { "end_turn" };
        json!({ "stop_reason": stop_reason, "content": content })
    }
}
