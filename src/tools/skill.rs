use anyhow::Ok;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

use super::Tool;
use crate::skills::SkillRegistry;

pub struct UseSkill(pub Arc<SkillRegistry>);

#[async_trait]
impl Tool for UseSkill {
    fn name(&self) -> &str {
        "use_skill"
    }

    fn description(&self) -> &str {
        "Invoke a named skill. Available skills are listed in the system prompt."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "skill_name": {
                    "type": "string",
                    "description": "The name of the skill to invoke"
                },
                "args": {
                    "type": "string",
                    "description": "Arguments to pass to the skill, substituted for {{args}}"
                }
            },
            "required": ["skill_name"]
        })
    }

    async fn call(&self, args: Value) -> anyhow::Result<Value> {
        let skill_name = args["skill_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing skill_name"))?;
        let skill_args = args["args"].as_str().unwrap_or("");

        match self.0.render(skill_name, skill_args) {
            Some(prompt) => Ok(json!(prompt)),
            None => Ok(json!(format!("error: unknown skill '{skill_name}'"))),
        }
    }
}
