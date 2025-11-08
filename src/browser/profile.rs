//! Browser profile configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProfile {
    pub headless: Option<bool>,
    pub user_data_dir: Option<PathBuf>,
    pub allowed_domains: Option<Vec<String>>,
    pub downloads_path: Option<PathBuf>,
}

impl Default for BrowserProfile {
    fn default() -> Self {
        Self {
            headless: None,
            user_data_dir: None,
            allowed_domains: None,
            downloads_path: None,
        }
    }
}

