use std::collections::HashMap;
use std::path::Path;
// use std::sync::Arc;

pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
}

pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

// pub type SharedSkillRegistry = Arc<SkillRegistry>;

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn load_dir(path: &Path) -> Self {
        let mut registry = Self::new();
        let entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return registry,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_file = path.join("SKILL.md");
            let content = match std::fs::read_to_string(&skill_file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Some(skill) = parse_skill(&content) {
                registry.skills.insert(skill.name.clone(), skill);
            }
        }
        registry
    }

    // pub fn get(&self, name: &str) -> Option<&Skill> {
    //     self.skills.get(name)
    // }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn render(&self, name: &str, args: &str) -> Option<String> {
        let skill = self.skills.get(name)?;
        Some(skill.body.replace("{{args}}", args))
    }
}

fn parse_skill(content: &str) -> Option<Skill> {
    let mut lines = content.trim_start().lines();

    if lines.next()?.trim() != "---" {
        return None;
    }

    let mut name = None;
    let mut description = None;
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in lines {
        if in_frontmatter {
            if line.trim() == "---" {
                in_frontmatter = false;
                continue;
            }
            if let Some(v) = line.strip_prefix("name:") {
                name = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().to_string());
            }
        } else {
            body_lines.push(line);
        }
    }

    Some(Skill {
        name: name?,
        description: description.unwrap_or_default(),
        body: body_lines.join("\n").trim().to_string(),
    })
}
