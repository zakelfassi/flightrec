use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::json;

use crate::llm::{LlmError, LlmProvider};

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Request timeout for provider HTTP calls. A stalled-but-connected endpoint
/// must not hang `watch` indefinitely (the LLM layer is non-fatal by contract).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct OllamaProvider {
    client: Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: Option<&str>) -> Result<Self, LlmError> {
        let base_url = base_url
            .unwrap_or(DEFAULT_BASE_URL)
            .trim_end_matches('/')
            .to_string();
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(LlmError::Http)?;
        Ok(Self { client, base_url })
    }
}

impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn complete(&self, model: &str, system: &str, user: &str) -> Result<String, LlmError> {
        let url = format!("{}/api/chat", self.base_url);
        let body = json!({
            "model": model,
            "stream": false,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        });

        let resp = self
            .client
            .post(&url)
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
        let text = json["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::BadOutput("missing message.content".to_string()))?;
        Ok(text.to_string())
    }
}
