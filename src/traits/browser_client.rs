//! Browser client trait for browser abstraction
//!
//! This trait defines the interface for browser operations, enabling
//! mock implementations for testing and alternative browser backends.

use crate::actor::Page;
use crate::browser::cdp::CdpClient;
use crate::browser::views::{SessionInfo, TabInfo};
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Trait for browser client operations
///
/// This trait provides a unified interface for browser automation,
/// abstracting over concrete browser implementations.
#[async_trait]
pub trait BrowserClient: Send + Sync {
    /// Start the browser session
    async fn start(&mut self) -> Result<()>;

    /// Navigate to the specified URL
    async fn navigate(&mut self, url: &str) -> Result<()>;

    /// Get the current page URL
    async fn get_current_url(&self) -> Result<String>;

    /// Create a new tab with optional URL
    async fn create_tab(&mut self, url: Option<&str>) -> Result<String>;

    /// Switch to a different tab by target ID
    async fn switch_to_tab(&mut self, target_id: &str) -> Result<()>;

    /// Close a tab by target ID
    async fn close_tab(&mut self, target_id: &str) -> Result<()>;

    /// Get all open tabs
    async fn get_tabs(&self) -> Result<Vec<TabInfo>>;

    /// Get target ID from short tab ID (last 4 characters)
    async fn get_target_id_from_tab_id(&self, tab_id: &str) -> Result<String>;

    /// Get a Page actor for the current session
    fn get_page(&self) -> Result<Page>;

    /// Take a screenshot of the current page
    async fn take_screenshot(
        &self,
        path: Option<&str>,
        full_page: bool,
    ) -> Result<Vec<u8>>;

    /// Get streamlined session information (URL, title, target ID, session ID)
    async fn get_session_info(&self) -> Result<SessionInfo> {
        Ok(SessionInfo {
            url: self.get_current_url().await?,
            title: self.get_current_page_title().await?,
            target_id: self.get_current_target_id()?,
            session_id: self.get_session_id()?,
        })
    }

    /// Get the current page title
    #[deprecated(since = "0.1.2", note = "Use get_session_info() instead")]
    async fn get_current_page_title(&self) -> Result<String>;

    /// Get the CDP client for the current session
    fn get_cdp_client(&self) -> Result<Arc<CdpClient>>;

    /// Get the session ID for the current target
    #[deprecated(since = "0.1.2", note = "Use get_session_info() instead")]
    fn get_session_id(&self) -> Result<String>;

    /// Get the current target ID
    #[deprecated(since = "0.1.2", note = "Use get_session_info() instead")]
    fn get_current_target_id(&self) -> Result<String>;
}
