mod mock;
mod openai;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::ProviderSettings;

pub use mock::MockProvider;
pub use openai::OpenAiCompatibleProvider;

#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub system_prompt: String,
    pub user_prompt: String,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, request: GenerationRequest) -> Result<String>;
}

pub fn provider_from_settings(settings: &ProviderSettings) -> Result<Box<dyn LlmProvider>> {
    match settings {
        ProviderSettings::Mock { model } => Ok(Box::new(MockProvider::new(model.clone()))),
        ProviderSettings::OpenAiCompatible {
            model,
            api_base,
            api_key_env,
            system_prompt,
        } => Ok(Box::new(OpenAiCompatibleProvider::new(
            model.clone(),
            api_base.clone(),
            api_key_env.clone(),
            system_prompt.clone(),
        ))),
    }
}
