pub mod prompt;
pub mod providers;

use crate::{
    config::LlmConfig,
    diff::{DiffEvent, DiffSummary},
    utils::now_iso,
};

/// Transport-only trait: send a prompt, get text back.
pub trait LlmProvider {
    fn name(&self) -> &str;
    fn complete(&self, model: &str, system: &str, user: &str) -> Result<String, LlmError>;
}

/// Errors from the LLM layer.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("missing API key: environment variable {env_var} is not set")]
    MissingApiKey { env_var: String },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },

    #[error("unexpected LLM output: {0}")]
    BadOutput(String),

    #[error("unknown provider: {0}")]
    UnknownProvider(String),
}

/// Build a boxed provider from a loaded `LlmConfig`.
pub fn provider_from_config(cfg: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
    match cfg.provider.as_str() {
        "anthropic" => {
            let key_env = cfg.api_key_env.as_deref().unwrap_or("ANTHROPIC_API_KEY");
            Ok(Box::new(providers::anthropic::AnthropicProvider::new(
                key_env,
            )?))
        }
        "openai" => {
            let key_env = cfg.api_key_env.as_deref();
            Ok(Box::new(providers::openai::OpenAiProvider::new(
                cfg.base_url.as_deref(),
                key_env,
                "openai",
            )?))
        }
        "openai-compatible" => {
            let key_env = cfg.api_key_env.as_deref();
            Ok(Box::new(providers::openai::OpenAiProvider::new(
                cfg.base_url.as_deref(),
                key_env,
                "openai-compatible",
            )?))
        }
        "ollama" => Ok(Box::new(providers::ollama::OllamaProvider::new(
            cfg.base_url.as_deref(),
        ))),
        other => Err(LlmError::UnknownProvider(other.to_string())),
    }
}

/// Strip optional ```json … ``` fences, then parse as JSON.
fn parse_json_output(raw: &str) -> Result<serde_json::Value, LlmError> {
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .map(|s| s.trim_end_matches("```").trim())
        .unwrap_or(trimmed);
    serde_json::from_str(stripped)
        .map_err(|e| LlmError::BadOutput(format!("JSON parse error: {e}")))
}

/// Call the LLM and parse the response into a `DiffSummary`.
pub fn summarize_diff(event: &DiffEvent, cfg: &LlmConfig) -> Result<DiffSummary, LlmError> {
    let provider = provider_from_config(cfg)?;
    let (system, user) = prompt::render(event, cfg.max_changes_per_prompt);
    let raw = provider.complete(&cfg.model, &system, &user)?;
    let json = parse_json_output(&raw)?;

    let short = json["short"]
        .as_str()
        .ok_or_else(|| LlmError::BadOutput("missing 'short' field".to_string()))?
        .to_string();

    let actions = json["actions"]
        .as_array()
        .ok_or_else(|| LlmError::BadOutput("missing 'actions' array".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let intent_guess = json["intent_guess"].as_str().map(|s| s.to_string());

    Ok(DiffSummary {
        llm_provider: provider.name().to_string(),
        model: cfg.model.clone(),
        generated_at: now_iso(),
        short,
        actions,
        intent_guess,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_json() {
        let raw = r#"{"short":"test","actions":["a"],"intent_guess":null}"#;
        let v = parse_json_output(raw).unwrap();
        assert_eq!(v["short"], "test");
    }

    #[test]
    fn parse_fenced_json() {
        let raw = "```json\n{\"short\":\"test\",\"actions\":[],\"intent_guess\":\"x\"}\n```";
        let v = parse_json_output(raw).unwrap();
        assert_eq!(v["short"], "test");
        assert_eq!(v["intent_guess"], "x");
    }

    #[test]
    fn parse_bare_fence_json() {
        let raw = "```\n{\"short\":\"s\",\"actions\":[\"a\",\"b\"],\"intent_guess\":null}\n```";
        let v = parse_json_output(raw).unwrap();
        assert_eq!(v["actions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn malformed_json_is_bad_output() {
        let err = parse_json_output("not json at all").unwrap_err();
        assert!(matches!(err, LlmError::BadOutput(_)));
    }
}
