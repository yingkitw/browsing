//! Browser profile configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Browser profile configuration
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
}
