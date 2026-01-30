//! Configuration management for browser-use-rs

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Configuration for browser profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProfileConfig {
    /// Whether to run browser in headless mode
    pub headless: Option<bool>,
    /// Path to user data directory
    pub user_data_dir: Option<PathBuf>,
    /// List of allowed domains
    pub allowed_domains: Option<Vec<String>>,
    /// Path to downloads directory
    pub downloads_path: Option<PathBuf>,
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy server URL
    pub server: String,
    /// Bypass list for proxy
    pub bypass: Option<String>,
    /// Username for proxy authentication
    pub username: Option<String>,
    /// Password for proxy authentication
    pub password: Option<String>,
}

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

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Browser profile configuration
    pub browser_profile: BrowserProfileConfig,
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
            browser_profile: BrowserProfileConfig {
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
                api_key: std::env::var("WATSONX_API_KEY")
                    .or_else(|_| std::env::var("IBM_WATSONX_API_KEY"))
                    .ok(),
                model: std::env::var("WATSONX_MODEL")
                    .ok()
                    .or_else(|| Some("ibm/granite-4-h-small".to_string())),
                temperature: std::env::var("WATSONX_TEMPERATURE")
                    .ok()
                    .and_then(|v| v.parse().ok()),
                max_tokens: std::env::var("WATSONX_MAX_TOKENS")
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
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        if !path.as_ref().exists() {
            warn!("Config file not found, using defaults");
            return Ok(Self::from_env());
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}
