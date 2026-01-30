//! View types and data structures

use serde::{Deserialize, Serialize};

// Placeholder for view types - will be populated during migration
/// Browser state summary placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateSummary {
    /// Current URL
    pub url: String,
    /// Current title
    pub title: String,
}
