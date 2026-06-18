mod agent;
mod config;
mod llm;
mod skills;
mod tools;

use std::sync::Arc;

use agent::Agent;
use clap::Parser;
use config::Config;
use llm::{LLMClient, Provider};
use tools::ToolRegistry;
use tools::browser::{FetchUrl, FetchUrlJs};
use tools::filesystem::{ListFiles, ReadFile, WriteFile};
use tools::search::SearchFiles;
use tools::shell::RunShell;

use crate::skills::SkillRegistry;
use crate::tools::skill::UseSkill;

#[derive(Parser)]
struct Cli {
    task: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    base_url: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut cfg = Config::from_env()?;

    if let Some(m) = cli.model {
        cfg.model = m;
    }
    if let Some(u) = cli.base_url {
        cfg.base_url = u;
    }

    let llm = LLMClient::new(
        cfg.api_key,
        cfg.base_url,
        cfg.model,
        Provider::from_str(&cfg.provider),
    );

    let mut registry = ToolRegistry::new();
    registry.register(ReadFile);
    registry.register(WriteFile);
    registry.register(ListFiles);
    registry.register(RunShell);
    registry.register(SearchFiles);
    registry.register(FetchUrl);
    registry.register(FetchUrlJs);

    let skills = Arc::new(SkillRegistry::load_dir(std::path::Path::new("skills")));
    registry.register(UseSkill(Arc::clone(&skills)));

    let skill_list = {
        let list = skills.list();
        if list.is_empty() {
            String::new()
        } else {
            let entries: Vec<String> = list
                .iter()
                .map(|s| format!("{} ({})", s.name, s.description))
                .collect();
            format!(
                "\nAvailable skills: {}.\nUse the use_skill tool to invoke one.",
                entries.join(", ")
            )
        }
    };

    let system = format!("You are a helpful coding agent.{skill_list}");
    let mut agent = Agent::new(llm, registry, system);

    if let Some(task) = cli.task {
        let answer = agent.run(&task).await?;
        println!("{answer}");
        return Ok(());
    }

    // REPL
    let mut rl = rustyline::DefaultEditor::new()?;
    loop {
        match rl.readline("kagent> ") {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                rl.add_history_entry(&line)?;
                let prompt = if line.starts_with('/') {
                    let rest = &line[1..];
                    let (name, args) = rest.split_once(' ').unwrap_or((rest, ""));
                    match skills.render(name, args) {
                        Some(p) => p,
                        None => { eprintln!("unknown skill: {name}"); continue; }
                    }
                } else {
                    line.clone()
                };
                match agent.run(&prompt).await {
                    Ok(answer) => println!("{answer}"),
                    Err(e) => eprintln!("error: {e}"),
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}
