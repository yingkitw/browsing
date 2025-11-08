//! Error types for browser-use-rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserUseError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("Browser error: {0}")]
    Browser(String),

    #[error("CDP error: {0}")]
    Cdp(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("DOM error: {0}")]
    Dom(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, BrowserUseError>;

