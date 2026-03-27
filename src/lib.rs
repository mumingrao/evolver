mod cli;
mod config;
mod evolution;
mod provider;

use std::env;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};
use config::AppConfig;
use evolution::EvolutionEngine;
use provider::{GenerationRequest, provider_from_settings};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let repo_root = env::current_dir()?;

    match cli.command {
        Commands::InitConfig { path, force } => {
            let target_path = path.unwrap_or_else(|| AppConfig::default_path(&repo_root));
            AppConfig::write_template(&target_path, force)?;
            println!("wrote config template to {}", target_path.display());
        }
        Commands::Status => {
            let config_path = AppConfig::default_path(&repo_root);
            let config = AppConfig::load_or_default(&repo_root)?;
            let engine = EvolutionEngine::new(&repo_root, config.evolution.clone());

            println!("repo: {}", repo_root.display());
            println!("config: {}", config_path.display());
            println!("provider: {}", config.provider.describe());
            println!("workspace: {}", engine.workspace_root().display());
            println!("candidates: {}", engine.count_candidates()?);
        }
        Commands::Prompt { prompt, system } => {
            let config = AppConfig::load_or_default(&repo_root)?;
            let provider = provider_from_settings(&config.provider)?;
            let response = provider
                .generate(GenerationRequest {
                    system_prompt: system.unwrap_or_else(|| {
                        "You are Evolver, a Rust coding assistant for the local repository."
                            .to_string()
                    }),
                    user_prompt: prompt,
                })
                .await?;

            println!("{response}");
        }
        Commands::Evolve { goal, apply } => {
            let config = AppConfig::load_or_default(&repo_root)?;
            let provider = provider_from_settings(&config.provider)?;
            let engine = EvolutionEngine::new(&repo_root, config.evolution.clone());
            let candidate = engine.stage(provider.as_ref(), &goal).await?;

            println!("staged candidate: {}", candidate.id);
            println!("path: {}", candidate.path.display());
            println!("summary: {}", candidate.summary);

            if apply {
                let applied = engine.apply(&candidate.id)?;
                println!("applied candidate into {}", applied.display());
            } else {
                println!("apply with: cargo run -- apply {}", candidate.id);
            }
        }
        Commands::Apply { candidate_id } => {
            let config = AppConfig::load_or_default(&repo_root)?;
            let engine = EvolutionEngine::new(&repo_root, config.evolution.clone());
            let applied = engine.apply(&candidate_id)?;
            println!("applied candidate into {}", applied.display());
        }
    }

    Ok(())
}
