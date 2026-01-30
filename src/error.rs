//! Error types for browser-use-rs

use thiserror::Error;

/// Error types for browser-use-rs
#[derive(Error, Debug)]
pub enum BrowserUseError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// URL parse error
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    /// Browser error
    #[error("Browser error: {0}")]
    Browser(String),

    /// Chrome DevTools Protocol error
    #[error("CDP error: {0}")]
    Cdp(String),

    /// LLM error
    #[error("LLM error: {0}")]
    Llm(String),

    /// Agent error
    #[error("Agent error: {0}")]
    Agent(String),

    /// DOM error
    #[error("DOM error: {0}")]
    Dom(String),

    /// Tool error
    #[error("Tool error: {0}")]
    Tool(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type alias for browser-use-rs
pub type Result<T> = std::result::Result<T, BrowserUseError>;

#[cfg(test)]
mod tests {
    use super::BrowserUseError;

    #[test]
    fn test_browser_error() {
        let err = BrowserUseError::Browser("Test error".to_string());
        assert!(err.to_string().contains("Test error"));
    }

    #[test]
    fn test_dom_error() {
        let err = BrowserUseError::Dom("DOM error".to_string());
        assert!(err.to_string().contains("DOM error"));
    }

    #[test]
    fn test_tool_error() {
        let err = BrowserUseError::Tool("Tool error".to_string());
        assert!(err.to_string().contains("Tool error"));
    }

    #[test]
    fn test_llm_error() {
        let err = BrowserUseError::Llm("LLM error".to_string());
        assert!(err.to_string().contains("LLM error"));
    }

    #[test]
    fn test_config_error() {
        let err = BrowserUseError::Config("Config error".to_string());
        assert!(err.to_string().contains("Config error"));
    }

    #[test]
    fn test_error_display() {
        let errors = vec![
            BrowserUseError::Browser("browser".to_string()),
            BrowserUseError::Dom("dom".to_string()),
            BrowserUseError::Tool("tool".to_string()),
            BrowserUseError::Llm("llm".to_string()),
            BrowserUseError::Config("config".to_string()),
        ];

        for err in errors {
            let display = format!("{err}");
            assert!(!display.is_empty());
        }
    }
}
