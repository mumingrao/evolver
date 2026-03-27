use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "evolver",
    version,
    about = "A self-extending Rust CLI foundation with staged LLM-driven evolution"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Write a starter config file to disk.
    InitConfig {
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Show the active provider and evolution workspace paths.
    Status,
    /// Send a direct prompt to the configured provider.
    Prompt {
        prompt: String,
        #[arg(long)]
        system: Option<String>,
    },
    /// Generate a staged candidate revision for the current project.
    Evolve {
        goal: String,
        #[arg(long)]
        apply: bool,
    },
    /// Apply a previously staged candidate into the repository.
    Apply { candidate_id: String },
}
