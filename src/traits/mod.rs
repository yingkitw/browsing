//! Core trait abstractions for browsing-rs
//!
//! This module defines the key traits that enable polymorphism and testability
//! throughout the browsing codebase.

mod browser_client;
mod dom_processor;

pub use browser_client::BrowserClient;
pub use dom_processor::DOMProcessor;
