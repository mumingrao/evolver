# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build, Test, and Format Commands

```bash
cargo fmt              # Format all crates
cargo test             # Run all unit tests
cargo build            # Build the workspace
cargo clippy           # Lint (warn on clippy::all + clippy::pedantic, with ~30 allow exceptions in main.rs)
```

Run `cargo fmt` and `cargo test` before opening a PR.

## Workspace Structure

This is a Cargo workspace (resolver 2, edition 2024, MSRV 1.87) with three members:

- **Root binary crate** (`src/main.rs`, `src/lib.rs`) — CLI entrypoint via `clap` derive. `src/lib.rs` re-exports `evolver_api` and `evolver_runtime` as `api` and `runtime`. The binary is named `evolver` (package name is `evolverlabs`).
- **`crates/evolver-api`** — Trait definitions and shared types. The architectural foundation.
- **`crates/evolver-runtime`** — Planned runtime subsystems (security, observability, gateway, cron, SOP, skills, hardware, TUI). Currently only re-exports `evolver_api`.

## Trait Architecture

The `evolver-api` crate defines the plugin interface for every major subsystem. All traits use `async_trait` and require `Send + Sync`:

| Trait | Module | Purpose |
|---|---|---|
| `Tool` | `tool.rs` | Agent-callable tools with `name()`, `description()`, `parameters_schema()`, `execute(args)`, and a convenience `spec()` |
| `Memory` | `memory_traits.rs` | Storage backend: `store`, `recall`, `get`, `list`, `forget`, `count`, `health_check`. Also `purge_namespace`, `purge_session`, `store_procedural`, `recall_namespaced`, `export`, `store_with_metadata` with default fallbacks. |
| `Channel` | `channel.rs` | Bidirectional messaging (Slack, Discord, terminal) with draft/edit/reaction/pin/approval support |
| `Observer` | `observability_traits.rs` | Telemetry: `record_event`, `record_metric`, `flush` with large event/metric enums covering the full agent lifecycle |
| `Peripheral` | `peripherals_traits.rs` | Hardware/IoT devices: `connect`, `disconnect`, `health_check`, `tools()` |
| `RuntimeAdapter` | `runtime_traits.rs` | Execution environment: shell/filesystem access, storage path, memory budget, `build_shell_command` |
| Provider | `provider.rs` | **Placeholder** — empty file, intended for LLM backend abstraction |

Key types: `TurnEvent` (agent loop enum: `Chunk`, `Thinking`, `ToolCall`, `ToolResult`), `ToolResult`, `ToolSpec`, `MemoryEntry`, `MemoryCategory`, `MediaAttachment`, `MediaKind`, `ObserverEvent`, `ObserverMetric`.

The API crate also defines two `tokio::task_local!` statics: `TOOL_LOOP_THREAD_ID` and `TOOL_CHOICE_OVERRIDE`.

## Key Conventions

- `anyhow::Result` for fallible functions, `thiserror` for library error types
- Tests live in `#[cfg(test)] mod tests` blocks alongside implementation
- Generated artifacts live under `.evolver/` and must stay untracked
- 4-space indentation, `snake_case` for modules/functions, `PascalCase` for types
- Companion file: `AGENTS.md` has additional CLI usage and PR guidelines
