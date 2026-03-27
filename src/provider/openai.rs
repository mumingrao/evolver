use std::env;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use super::{GenerationRequest, LlmProvider};

pub struct OpenAiCompatibleProvider {
    client: Client,
    model: String,
    api_base: String,
    api_key_env: String,
    system_prompt: Option<String>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        model: String,
        api_base: String,
        api_key_env: String,
        system_prompt: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            model,
            api_base,
            api_key_env,
            system_prompt,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn generate(&self, request: GenerationRequest) -> Result<String> {
        let api_key = env::var(&self.api_key_env)
            .with_context(|| format!("missing API key env var {}", self.api_key_env))?;

        let system_prompt = match &self.system_prompt {
            Some(provider_prompt) if !provider_prompt.is_empty() => {
                format!("{provider_prompt}\n\n{}", request.system_prompt)
            }
            _ => request.system_prompt,
        };

        let url = format!("{}/chat/completions", self.api_base.trim_end_matches('/'));
        let response = self
            .client
            .post(url)
            .bearer_auth(api_key)
            .json(&json!({
                "model": self.model,
                "temperature": 0.2,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": request.user_prompt}
                ]
            }))
            .send()
            .await?
            .error_for_status()?;

        let payload: ChatCompletionResponse = response.json().await?;
        let message = payload
            .choices
            .into_iter()
            .next()
            .context("provider returned no choices")?
            .message
            .content;

        Ok(message)
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}
