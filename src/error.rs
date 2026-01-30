//! Error types for browsing

use thiserror::Error;

/// Error types for browsing
#[derive(Error, Debug)]
pub enum BrowsingError {
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

/// Result type alias for browsing
pub type Result<T> = std::result::Result<T, BrowsingError>;

#[cfg(test)]
mod tests {
    use super::BrowsingError;

    #[test]
    fn test_browser_error() {
        let err = BrowsingError::Browser("Test error".to_string());
        assert!(err.to_string().contains("Test error"));
    }

    #[test]
    fn test_dom_error() {
        let err = BrowsingError::Dom("DOM error".to_string());
        assert!(err.to_string().contains("DOM error"));
    }

    #[test]
    fn test_tool_error() {
        let err = BrowsingError::Tool("Tool error".to_string());
        assert!(err.to_string().contains("Tool error"));
    }

    #[test]
    fn test_llm_error() {
        let err = BrowsingError::Llm("LLM error".to_string());
        assert!(err.to_string().contains("LLM error"));
    }

    #[test]
    fn test_config_error() {
        let err = BrowsingError::Config("Config error".to_string());
        assert!(err.to_string().contains("Config error"));
    }

    #[test]
    fn test_error_display() {
        let errors = vec![
            BrowsingError::Browser("browser".to_string()),
            BrowsingError::Dom("dom".to_string()),
            BrowsingError::Tool("tool".to_string()),
            BrowsingError::Llm("llm".to_string()),
            BrowsingError::Config("config".to_string()),
        ];

        for err in errors {
            let display = format!("{err}");
            assert!(!display.is_empty());
        }
    }
}
