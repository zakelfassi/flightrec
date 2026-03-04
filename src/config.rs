use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::utils::expand_tilde;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WatchConfig {
    pub roots: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FilterConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DaemonConfig {
    pub interval_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputConfig {
    pub json_log_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub watch: WatchConfig,
    pub filter: FilterConfig,
    pub daemon: DaemonConfig,
    pub llm: LlmConfig,
    pub output: OutputConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            watch: WatchConfig {
                roots: vec![
                    "~/.openclaw".to_string(),
                    "~/clawd".to_string(),
                    "~/tac-monorepo".to_string(),
                ],
            },
            filter: FilterConfig {
                include: vec![
                    "**/*.md".to_string(),
                    "**/*.rs".to_string(),
                    "**/*.toml".to_string(),
                    "**/*.json".to_string(),
                    "**/*.yml".to_string(),
                    "**/*.yaml".to_string(),
                    "**/*.rb".to_string(),
                    "**/*.py".to_string(),
                    "**/*.ts".to_string(),
                    "**/*.tsx".to_string(),
                    "**/*.js".to_string(),
                ],
                exclude: vec![
                    "**/.git/**".to_string(),
                    "**/node_modules/**".to_string(),
                    "**/*.log".to_string(),
                    "**/.DS_Store".to_string(),
                    "**/tmp/**".to_string(),
                    "**/.cache/**".to_string(),
                    "**/.next/**".to_string(),
                    "**/target/**".to_string(),
                ],
            },
            daemon: DaemonConfig {
                interval_seconds: 60,
            },
            llm: LlmConfig {
                enabled: false,
                provider: "anthropic".to_string(),
                model: "claude-haiku-4-5".to_string(),
            },
            output: OutputConfig {
                json_log_dir: "~/.agentscope/logs".to_string(),
            },
        }
    }
}

pub fn load_config() -> Result<Config> {
    let config_path = expand_tilde("~/.agentscope/config.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}
