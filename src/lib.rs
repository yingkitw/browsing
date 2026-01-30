//! Browsing: Autonomous web browsing for AI agents
//!
//! This library provides tools for AI agents to automate web browsing tasks.

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

pub use error::{BrowsingError, Result};

// Re-export main types
pub use actor::{Element, Mouse, Page};
pub use agent::Agent;
pub use browser::Browser;
pub use config::Config;
pub use llm::{ChatInvokeCompletion, ChatInvokeUsage, ChatMessage, ChatModel};
pub use traits::{BrowserClient, DOMProcessor};

/// Initialize the library (sets up logging, etc.)
pub fn init() {
    logging::setup_logging();
}
