//! Configuration management for browsing-rs

use crate::browser::profile::BrowserProfile;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Configuration for LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API key for the LLM service
    pub api_key: Option<String>,
    /// Model name to use
    pub model: Option<String>,
    /// Temperature for generation
    pub temperature: Option<f64>,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
}

/// Configuration for the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of steps
    pub max_steps: Option<u32>,
    /// Whether to use vision
    pub use_vision: Option<bool>,
    /// System prompt override
    pub system_prompt: Option<String>,
}

/// Main configuration structure (streamlined)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Browser profile configuration (unified)
    pub browser_profile: BrowserProfile,
    /// LLM configuration
    pub llm: LlmConfig,
    /// Agent configuration
    pub agent: AgentConfig,
}

impl Config {
    /// Creates a Config from environment variables
    pub fn from_env() -> Self {
        // Load .env file if present
        let _ = dotenv::dotenv();

        Self {
            browser_profile: BrowserProfile {
                headless: std::env::var("BROWSER_USE_HEADLESS")
                    .ok()
                    .and_then(|v| v.parse().ok()),
                user_data_dir: std::env::var("BROWSER_USE_USER_DATA_DIR")
                    .ok()
                    .map(PathBuf::from),
                allowed_domains: std::env::var("BROWSER_USE_ALLOWED_DOMAINS")
                    .ok()
                    .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
                downloads_path: std::env::var("BROWSER_USE_DOWNLOADS_PATH")
                    .ok()
                    .map(PathBuf::from),
                proxy: None, // TODO: Parse from env vars
            },
            llm: LlmConfig {
                api_key: std::env::var("LLM_API_KEY").ok(),
                model: std::env::var("LLM_MODEL").ok(),
                temperature: std::env::var("LLM_TEMPERATURE")
                    .ok()
                    .and_then(|v| v.parse().ok()),
                max_tokens: std::env::var("LLM_MAX_TOKENS")
                    .ok()
                    .and_then(|v| v.parse().ok()),
            },
            agent: AgentConfig {
                max_steps: std::env::var("BROWSER_USE_MAX_STEPS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .or(Some(100)),
                use_vision: std::env::var("BROWSER_USE_VISION")
                    .ok()
                    .and_then(|v| v.parse().ok()),
                system_prompt: None,
            },
        }
    }

    /// Loads configuration from a file
    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
    ) -> anyhow::Result<Self> {
        if !path.as_ref().exists() {
            warn!("Config file not found, using defaults");
            return Ok(Self::from_env());
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}
