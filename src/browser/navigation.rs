//! Navigation management for browser sessions
//!
//! This module handles navigation operations like going forward/back in history.

use crate::actor::Page;
use crate::error::{BrowsingError, Result};
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

    /// Go back in browser history
    pub async fn go_back(&self, page: &Page) -> Result<()> {
        page.go_back().await?;
        info!("Navigated back in browser history");
        Ok(())
    }

    /// Go forward in browser history
    pub async fn go_forward(&self, page: &Page) -> Result<()> {
        page.go_forward().await?;
        info!("Navigated forward in browser history");
        Ok(())
    }
}

impl Default for NavigationManager {
    fn default() -> Self {
        Self::new()
    }
}
