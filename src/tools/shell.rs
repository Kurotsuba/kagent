use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use super::Tool;

pub struct RunShell;

#[async_trait]
impl Tool for RunShell {
    fn name(&self) -> &str { "run_shell" }

    fn description(&self) -> &str {
        "Run a shell command and return stdout, stderr, and exit code."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute."
                }
            },
            "required": ["command"]
        })
    }

    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        let command = args["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing 'command'"))?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await?;

        Ok(json!({
            "stdout":    String::from_utf8_lossy(&output.stdout).trim_end().to_string(),
            "stderr":    String::from_utf8_lossy(&output.stderr).trim_end().to_string(),
            "exit_code": output.status.code(),
        }))
    }
}
