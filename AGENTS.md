# Repository Guidelines

## Project Structure & Module Organization
`evolver` is a Rust CLI crate. Keep top-level files limited to Rust and repo metadata: `Cargo.toml`, `Cargo.lock`, `.gitignore`, and this guide.

- `src/main.rs`: binary entrypoint
- `src/lib.rs`: command dispatch and app wiring
- `src/cli.rs`: `clap` command definitions
- `src/config.rs`: config loading and template generation
- `src/evolution.rs`: staged self-evolution workflow
- `src/provider/`: LLM backends such as `mock.rs` and `openai.rs`

Generated candidates live under `.evolver/` and must stay untracked.

## Build, Test, and Development Commands
Use Cargo for all local workflows:

- `cargo fmt`: format the crate
- `cargo test`: run unit tests
- `cargo run -- status`: inspect active config, provider, and candidate workspace
- `cargo run -- init-config`: write `.evolver/config.toml`
- `cargo run -- prompt "..."`: send a direct prompt to the configured provider
- `cargo run -- evolve "goal"`: stage a candidate revision under `.evolver/candidates/`
- `cargo run -- apply <candidate-id>`: copy a staged candidate into the live repo

Run `cargo fmt` and `cargo test` before opening a PR.

## Coding Style & Naming Conventions
Follow standard Rust conventions: 4-space indentation, `snake_case` for modules/functions, `PascalCase` for types, and focused modules instead of oversized files. Prefer `anyhow::Result` for top-level error flow and keep provider-specific code isolated under `src/provider/`.

Use `cargo fmt` as the formatting authority. Add comments only where control flow or safety constraints are not obvious from the code.

## Testing Guidelines
Place unit tests next to implementation with `#[cfg(test)]` modules, as in `src/config.rs` and `src/evolution.rs`. Test names should describe behavior, for example `rejects_parent_segments`.

New evolution or provider logic should include at least one success-path test and one safety or validation test.

## Commit & Pull Request Guidelines
There is no established commit history yet, so use short imperative subjects such as `Add OpenAI-compatible provider` or `Validate candidate bundle paths`.

PRs should include a concise summary, test notes, and sample CLI output when command behavior changes. If a change affects staged evolution, mention how `.evolver/` artifacts were validated and confirm they were not committed.
