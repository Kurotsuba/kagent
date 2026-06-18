pub mod filesystem;
pub mod search;
pub mod shell;
pub mod skill;

use async_trait::async_trait;
use serde_json::Value;

use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    async fn call(&self, args: Value) -> anyhow::Result<Value>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: impl Tool + 'static) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    pub fn to_anthropic_tools(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "input_schema": t.parameters(),
                })
            })
            .collect()
    }

    pub async fn dispatch(&self, name: &str, args: Value) -> anyhow::Result<String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown tool: {name}"))?;
        let result = tool.call(args).await?;
        Ok(serde_json::to_string(&result)?)
    }
}
