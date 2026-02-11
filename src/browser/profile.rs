//! Browser profile configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// Browser profile configuration (streamlined, single source of truth)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrowserProfile {
    /// Whether to run browser in headless mode
    pub headless: Option<bool>,
    /// Path to user data directory
    pub user_data_dir: Option<PathBuf>,
    /// List of allowed domains
    pub allowed_domains: Option<Vec<String>>,
    /// Path to downloads directory
    pub downloads_path: Option<PathBuf>,
    /// Proxy configuration (for enterprise use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfig>,
}

impl BrowserProfile {
    /// Create a new BrowserProfile with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set headless mode
    pub fn with_headless(mut self, headless: bool) -> Self {
        self.headless = Some(headless);
        self
    }

    /// Set user data directory
    pub fn with_user_data_dir(mut self, dir: PathBuf) -> Self {
        self.user_data_dir = Some(dir);
        self
    }

    /// Set proxy configuration
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }
}
