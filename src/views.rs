//! View types and data structures

use serde::{Deserialize, Serialize};

// Placeholder for view types - will be populated during migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateSummary {
    pub url: String,
    pub title: String,
}

