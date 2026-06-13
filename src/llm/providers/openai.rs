use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::json;

use crate::llm::{LlmError, LlmProvider};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_API_KEY_ENV: &str = "OPENAI_API_KEY";

/// Request timeout for provider HTTP calls. A stalled-but-connected endpoint
/// must not hang `watch` indefinitely (the LLM layer is non-fatal by contract).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    provider_name: String,
}

impl OpenAiProvider {
    pub fn new(
        base_url: Option<&str>,
        api_key_env: Option<&str>,
        provider_name: &str,
    ) -> Result<Self, LlmError> {
        let key_env = api_key_env.unwrap_or(DEFAULT_API_KEY_ENV);
        let api_key = std::env::var(key_env).map_err(|_| LlmError::MissingApiKey {
            env_var: key_env.to_string(),
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
            provider_name: provider_name.to_string(),
        })
    }
}

impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn complete(&self, model: &str, system: &str, user: &str) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        });

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
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
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::BadOutput("missing choices[0].message.content".to_string()))?;
        Ok(text.to_string())
    }
}
