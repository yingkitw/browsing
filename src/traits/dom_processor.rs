//! DOM processor trait for DOM abstraction
//!
//! This trait defines the interface for DOM processing operations,
//! enabling different DOM processing implementations.

use crate::dom::views::{DOMInteractedElement, SerializedDOMState};
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for DOM processing operations
///
/// This trait provides a unified interface for extracting and processing
/// DOM information from web pages.
#[async_trait]
pub trait DOMProcessor: Send + Sync {
    /// Get serialized DOM state
    async fn get_serialized_dom(&self) -> Result<SerializedDOMState>;

    /// Get page state as string for LLM consumption
    async fn get_page_state_string(&self) -> Result<String>;

    /// Get selector map (index -> element mapping)
    async fn get_selector_map(&self) -> Result<HashMap<u32, DOMInteractedElement>>;
}
