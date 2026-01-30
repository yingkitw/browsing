//! Token usage and cost tracking views

use serde::{Deserialize, Serialize};

/// Summary of token usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageSummary {
    /// Number of prompt tokens used
    pub prompt_tokens: Option<u32>,
    /// Number of completion tokens used
    pub completion_tokens: Option<u32>,
    /// Total number of tokens used
    pub total_tokens: Option<u32>,
    /// Estimated cost
    pub cost: Option<f64>,
}
