//! Browser-Use: Make websites accessible for AI agents
//!
//! This is a Rust implementation of the browser-use library for autonomous web automation.

pub mod actor;
pub mod agent;
pub mod browser;
pub mod config;
pub mod dom;
pub mod error;
pub mod llm;
pub mod logging;
pub mod tokens;
pub mod tools;
pub mod traits;
pub mod utils;
pub mod views;

pub use error::{BrowserUseError, Result};

// Re-export main types
pub use actor::{Element, Mouse, Page};
pub use agent::Agent;
pub use browser::Browser;
pub use config::Config;
pub use llm::{ChatMessage, ChatModel, WatsonxChat};
pub use traits::{BrowserClient, DOMProcessor};

/// Initialize the library (sets up logging, etc.)
pub fn init() {
    logging::setup_logging();
}
