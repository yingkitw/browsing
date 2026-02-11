//! Navigation management for browser sessions
//!
//! This module handles navigation to URLs.

use crate::actor::Page;
use crate::error::Result;
use tracing::info;

/// Manager for browser navigation operations
pub struct NavigationManager;

impl NavigationManager {
    /// Create a new navigation manager
    pub fn new() -> Self {
        Self
    }

    /// Navigate to the specified URL
    pub async fn navigate(&self, page: &Page, url: &str) -> Result<()> {
        page.goto(url).await?;
        info!("Navigated to: {}", url);
        Ok(())
    }
}

impl Default for NavigationManager {
    fn default() -> Self {
        Self::new()
    }
}
