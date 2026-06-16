# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build          # compile
cargo run            # run interactive REPL
cargo run -- "task"  # single-shot mode
cargo run -- --model llama3.2 --base-url http://localhost:11434/v1
```

## Architecture

`kagent` is a CLI code agent that loops with an LLM until it produces a final answer (no tool calls). Four env vars configure it (loaded from `.env` via `dotenvy`): `KAGENT_API_KEY`, `KAGENT_BASE_URL`, `KAGENT_MODEL`, and `KAGENT_PROVIDER` (`anthropic` or `openai`).

### Agent loop (`src/agent.rs`)

Core loop:
1. Call LLM with current message history and registered tools
2. If response has tool calls → dispatch each via `ToolRegistry`, push results as tool messages, repeat
3. If no tool calls → print response, break

Conversation history is kept in `Agent.messages` (in-memory, per session).

### LLM client (`src/llm.rs`)

Uses `reqwest` for raw HTTP — no OpenAI SDK. Supports two providers:
- `Anthropic` — posts to `/v1/messages` with `x-api-key` / `anthropic-version` headers
- `OpenAI` — posts to `/chat/completions` with `Authorization: Bearer` header

The agent always stores messages and tools in Anthropic format. `LLMClient::chat()` converts to the right wire format and normalises the response back before returning.

### Tool system (`src/tools/`)

`Tool` trait in `mod.rs` — implement four methods (`name`, `description`, `parameters` as JSON Schema, `call`), register with `ToolRegistry::register()`. Adding a new tool = new file + one `register()` call.

Built-in tools: `read_file`, `write_file`, `list_files` (`filesystem.rs`), `run_shell` (`shell.rs`), `search_files` (`search.rs`).

### Supporting modules

- `src/config.rs` — reads env vars / `.env`
- `src/main.rs` — `clap` CLI args + `rustyline` REPL
