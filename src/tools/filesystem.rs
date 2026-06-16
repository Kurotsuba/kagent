use async_trait::async_trait;
use serde_json::{json, Value};

use crate::tools::Tool;

pub struct ReadFile;
pub struct WriteFile;
pub struct ListFiles;

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read the contents of a file at the given path" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file" }
            },
            "required": ["path"]
        })
    }
    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        let path = args["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("missing argument: path"))?;
        let content = tokio::fs::read_to_string(path).await?;
        Ok(Value::String(content))
    }
}

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Write content to a file at the given path" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file" },
                "content": { "type": "string", "description": "Content to write" }
            },
            "required": ["path", "content"]
        })
    }
    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        let path = args["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("missing argument: path"))?;
        let content = args["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("missing argument: content"))?;
        tokio::fs::write(path, content).await?;
        Ok(Value::String(format!("wrote {} bytes to {path}", content.len())))
    }
}

#[async_trait]
impl Tool for ListFiles {
    fn name(&self) -> &str { "list_files" }
    fn description(&self) -> &str { "List files and directories at the given path" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the directory" }
            },
            "required": ["path"]
        })
    }
    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        let path = args["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("missing argument: path"))?;
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            names.push(Value::String(entry.file_name().to_string_lossy().into_owned()));
        }
        Ok(Value::Array(names))
    }
}
