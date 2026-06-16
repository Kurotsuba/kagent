use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use super::Tool;

pub struct SearchFiles;

#[async_trait]
impl Tool for SearchFiles {
    fn name(&self) -> &str { "search_files" }

    fn description(&self) -> &str {
        "Search for a pattern across files in a directory. Returns matching lines with file paths and line numbers."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The search pattern (regex supported)."
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in."
                },
                "glob": {
                    "type": "string",
                    "description": "File glob to restrict which files are searched, e.g. '*.rs'. Searches all files if omitted."
                }
            },
            "required": ["pattern", "path"]
        })
    }

    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing 'pattern'"))?;
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing 'path'"))?;

        let mut cmd = Command::new("grep");
        cmd.args(["-rn", "--color=never"]);

        if let Some(glob) = args["glob"].as_str() {
            cmd.args(["--include", glob]);
        }

        cmd.arg(pattern).arg(path);

        let output = cmd.output().await?;

        Ok(json!({
            "matches": String::from_utf8_lossy(&output.stdout).trim_end().to_string(),
            "stderr":  String::from_utf8_lossy(&output.stderr).trim_end().to_string(),
        }))
    }
}
