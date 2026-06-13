use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::json;

use crate::llm::{LlmError, LlmProvider};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Request timeout for provider HTTP calls. A stalled-but-connected endpoint
/// must not hang `watch` indefinitely (the LLM layer is non-fatal by contract).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(base_url: Option<&str>, api_key_env: &str) -> Result<Self, LlmError> {
        let api_key = std::env::var(api_key_env).map_err(|_| LlmError::MissingApiKey {
            env_var: api_key_env.to_string(),
        })?;
        let base_url = base_url
            .unwrap_or(DEFAULT_BASE_URL)
            .trim_end_matches('/')
            .to_string();
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(LlmError::Http)?;
        Ok(Self {
            client,
            api_key,
            base_url,
        })
    }
}

impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn complete(&self, model: &str, system: &str, user: &str) -> Result<String, LlmError> {
        let body = json!({
            "model": model,
            "max_tokens": 1024,
            "system": system,
            "messages": [{"role": "user", "content": user}]
        });

        let url = format!("{}/messages", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(LlmError::Http)?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(LlmError::Api {
                status: status.as_u16(),
                body: body_text,
            });
        }

        let json: serde_json::Value = resp.json().map_err(LlmError::Http)?;
        let text = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| LlmError::BadOutput("missing content[0].text".to_string()))?;
        Ok(text.to_string())
    }
}
