# kagent

A minimal CLI code agent in Rust. Give it a task; it calls tools and loops until it has an answer.

## Setup

```sh
cargo build --release
cp .env.example .env
# fill in your API key and provider in .env
```

## Usage

```sh
# Interactive REPL
cargo run

# Single-shot
cargo run -- "what files are in the current directory?"

# Override model or endpoint at runtime
cargo run -- --model gpt-4o --base-url https://api.openai.com/v1
```

## Configuration

Copy `.env.example` to `.env` and set the vars for your provider:

| Variable | Default | Description |
|---|---|---|
| `KAGENT_API_KEY` | *(required)* | API key |
| `KAGENT_BASE_URL` | `https://api.anthropic.com` | API base URL |
| `KAGENT_MODEL` | `claude-haiku-4-5` | Model name |
| `KAGENT_PROVIDER` | `anthropic` | `anthropic` or `openai` |

`KAGENT_PROVIDER=openai` works with any OpenAI-compatible endpoint (OpenAI, OpenRouter, Ollama, etc.).

## Built-in Tools

| Tool | Description |
|---|---|
| `read_file` | Read a file |
| `write_file` | Write a file |
| `list_files` | List a directory |
| `run_shell` | Run a shell command |
| `search_files` | Grep across files (`pattern`, `path`, optional `glob`) |

## Adding a Tool

1. Create `src/tools/mytool.rs`, implement the `Tool` trait:

```rust
pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Does something useful." }
    fn parameters(&self) -> Value { json!({ "type": "object", "properties": { ... }, "required": [...] }) }
    async fn call(&self, args: Value) -> anyhow::Result<Value> { ... }
}
```

2. Add `pub mod mytool;` to `src/tools/mod.rs`.
3. Register it in `src/main.rs`: `registry.register(MyTool);`

## Architecture

```
src/
├── main.rs       # clap CLI + rustyline REPL
├── config.rs     # env var loading
├── llm.rs        # HTTP client for Anthropic and OpenAI wire formats
├── agent.rs      # tool-call loop with persistent in-session history
└── tools/
    ├── mod.rs        # Tool trait + ToolRegistry
    ├── filesystem.rs
    ├── shell.rs
    └── search.rs
```

The agent keeps conversation history for the duration of a REPL session. Each new `cargo run` starts fresh.
