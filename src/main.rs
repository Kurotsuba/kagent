mod agent;
mod config;
mod llm;
mod tools;

use agent::Agent;
use clap::Parser;
use config::Config;
use llm::{LLMClient, Provider};
use tools::ToolRegistry;
use tools::filesystem::{ListFiles, ReadFile, WriteFile};
use tools::search::SearchFiles;
use tools::shell::RunShell;

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

    let llm = LLMClient::new(cfg.api_key, cfg.base_url, cfg.model, Provider::from_str(&cfg.provider));

    let mut registry = ToolRegistry::new();
    registry.register(ReadFile);
    registry.register(WriteFile);
    registry.register(ListFiles);
    registry.register(RunShell);
    registry.register(SearchFiles);

    let mut agent = Agent::new(llm, registry, "You are a helpful coding agent.");

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
                match agent.run(&line).await {
                    Ok(answer) => println!("{answer}"),
                    Err(e) => eprintln!("error: {e}"),
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}
