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
        )?)),
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

/// Strip optional code fences, parse JSON, and extract the `(short, actions,
/// intent_guess)` triple that every valid LLM response must contain.
///
/// Returns `LlmError::BadOutput` if:
/// - the JSON cannot be parsed,
/// - `"short"` is missing or non-string,
/// - `"actions"` is missing or non-array,
/// - **any** entry in `"actions"` is not a string.
fn extract_summary_fields(raw: &str) -> Result<(String, Vec<String>, Option<String>), LlmError> {
    let json = parse_json_output(raw)?;

    let short = json["short"]
        .as_str()
        .ok_or_else(|| LlmError::BadOutput("missing 'short' field".to_string()))?
        .to_string();

    let actions_arr = json["actions"]
        .as_array()
        .ok_or_else(|| LlmError::BadOutput("missing 'actions' array".to_string()))?;

    let actions: Vec<String> = actions_arr
        .iter()
        .map(|v| {
            v.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| LlmError::BadOutput(format!("non-string action entry: {v}")))
        })
        .collect::<Result<_, LlmError>>()?;

    let intent_guess = json["intent_guess"].as_str().map(|s| s.to_string());

    Ok((short, actions, intent_guess))
}

/// Call the LLM and parse the response into a `DiffSummary`.
pub fn summarize_diff(event: &DiffEvent, cfg: &LlmConfig) -> Result<DiffSummary, LlmError> {
    let provider = provider_from_config(cfg)?;
    let (system, user) = prompt::render(event, cfg.max_changes_per_prompt);
    let raw = provider.complete(&cfg.model, &system, &user)?;
    let (short, actions, intent_guess) = extract_summary_fields(&raw)?;

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

    // ── extract_summary_fields (production parser) ───────────────────────────

    #[test]
    fn extract_plain_response() {
        let raw =
            r#"{"short":"did things","actions":["step one","step two"],"intent_guess":"refactor"}"#;
        let (short, actions, intent) = extract_summary_fields(raw).unwrap();
        assert_eq!(short, "did things");
        assert_eq!(actions, vec!["step one", "step two"]);
        assert_eq!(intent.as_deref(), Some("refactor"));
    }

    #[test]
    fn extract_fenced_response() {
        let raw =
            "```json\n{\"short\":\"ok\",\"actions\":[\"a\",\"b\"],\"intent_guess\":\"x\"}\n```";
        let (short, actions, intent) = extract_summary_fields(raw).unwrap();
        assert_eq!(short, "ok");
        assert_eq!(actions, vec!["a", "b"]);
        assert_eq!(intent.as_deref(), Some("x"));
    }

    #[test]
    fn non_string_action_entry_is_bad_output() {
        // Any numeric or object entry in `actions` must be rejected, not silently dropped.
        let raw = r#"{"short":"x","actions":[42],"intent_guess":null}"#;
        let err = extract_summary_fields(raw).unwrap_err();
        assert!(
            matches!(err, LlmError::BadOutput(_)),
            "expected BadOutput, got {err:?}"
        );
    }

    #[test]
    fn mixed_action_types_is_bad_output() {
        let raw = r#"{"short":"x","actions":["valid",{"nested":"obj"}],"intent_guess":null}"#;
        let err = extract_summary_fields(raw).unwrap_err();
        assert!(matches!(err, LlmError::BadOutput(_)));
    }

    #[test]
    fn missing_short_field_is_bad_output() {
        let raw = r#"{"actions":["a"],"intent_guess":null}"#;
        let err = extract_summary_fields(raw).unwrap_err();
        assert!(matches!(err, LlmError::BadOutput(_)));
    }

    #[test]
    fn missing_actions_field_is_bad_output() {
        let raw = r#"{"short":"x","intent_guess":null}"#;
        let err = extract_summary_fields(raw).unwrap_err();
        assert!(matches!(err, LlmError::BadOutput(_)));
    }
}
