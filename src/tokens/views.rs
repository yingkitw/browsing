//! Token usage and cost tracking views

use serde::{Deserialize, Serialize};

/// Summary of token usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageSummary {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub cost: Option<f64>,
}

