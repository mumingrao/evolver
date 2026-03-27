use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub provider: ProviderSettings,
    #[serde(default)]
    pub evolution: EvolutionSettings,
}

impl AppConfig {
    pub fn default_path(repo_root: &Path) -> PathBuf {
        match env::var("EVOLVER_CONFIG") {
            Ok(path) => PathBuf::from(path),
            Err(_) => repo_root.join(".evolver").join("config.toml"),
        }
    }

    pub fn load_or_default(repo_root: &Path) -> Result<Self> {
        let path = Self::default_path(repo_root);
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config at {}", path.display()))?;
        let config: Self = toml::from_str(&raw)
            .with_context(|| format!("invalid config at {}", path.display()))?;
        Ok(config)
    }

    pub fn write_template(path: &Path, force: bool) -> Result<()> {
        if path.exists() && !force {
            bail!(
                "refusing to overwrite existing config {}; pass --force to replace it",
                path.display()
            );
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        fs::write(path, Self::template())
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn template() -> String {
        [
            "# Evolver configuration",
            "# The default provider is offline-safe mock mode.",
            "",
            "[provider]",
            "kind = \"mock\"",
            "model = \"mock-evolver\"",
            "",
            "# Switch to an OpenAI-compatible endpoint when you are ready.",
            "# [provider]",
            "# kind = \"openai-compatible\"",
            "# model = \"your-model-id\"",
            "# api_base = \"https://api.openai.com/v1\"",
            "# api_key_env = \"OPENAI_API_KEY\"",
            "",
            "[evolution]",
            "workspace_dir = \".evolver\"",
            "",
        ]
        .join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ProviderSettings {
    Mock {
        #[serde(default = "default_mock_model")]
        model: String,
    },
    OpenAiCompatible {
        model: String,
        api_base: String,
        api_key_env: String,
        #[serde(default)]
        system_prompt: Option<String>,
    },
}

impl ProviderSettings {
    pub fn describe(&self) -> String {
        match self {
            Self::Mock { model } => format!("mock ({model})"),
            Self::OpenAiCompatible {
                model,
                api_base,
                api_key_env,
                ..
            } => format!("openai-compatible ({model}, {api_base}, key: {api_key_env})"),
        }
    }
}

impl Default for ProviderSettings {
    fn default() -> Self {
        Self::Mock {
            model: default_mock_model(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionSettings {
    #[serde(default = "default_workspace_dir")]
    pub workspace_dir: PathBuf,
}

impl Default for EvolutionSettings {
    fn default() -> Self {
        Self {
            workspace_dir: default_workspace_dir(),
        }
    }
}

fn default_mock_model() -> String {
    "mock-evolver".to_string()
}

fn default_workspace_dir() -> PathBuf {
    PathBuf::from(".evolver")
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn template_mentions_workspace_dir() {
        let template = AppConfig::template();
        assert!(template.contains("workspace_dir"));
        assert!(template.contains("kind = \"mock\""));
    }
}
