//! Screenshot management for browser sessions
//!
//! This module handles screenshot capture and saving operations.

use crate::error::{BrowsingError, Result};
use base64::{Engine as _, engine::general_purpose};
use tracing::info;

/// Manager for screenshot operations
pub struct ScreenshotManager;

impl ScreenshotManager {
    /// Create a new screenshot manager
    pub fn new() -> Self {
        Self
    }

    /// Take a screenshot of the current page
    pub async fn take_screenshot(
        &self,
        page: &crate::actor::Page,
        path: Option<&str>,
        full_page: bool,
        format: Option<&str>,
        quality: Option<u32>,
    ) -> Result<Vec<u8>> {
        let data_b64 = page
            .screenshot_with_options(format, quality, full_page, None)
            .await?;

        // Decode base64
        let screenshot_data = general_purpose::STANDARD
            .decode(&data_b64)
            .map_err(|e| BrowsingError::Browser(format!("Failed to decode screenshot: {}", e)))?;

        // Save to file if path provided
        if let Some(file_path) = path {
            tokio::fs::write(file_path, &screenshot_data)
                .await
                .map_err(|e| {
                    BrowsingError::Browser(format!("Failed to save screenshot: {}", e))
                })?;
            info!("Screenshot saved to: {}", file_path);
        }

        Ok(screenshot_data)
    }

    /// Take a screenshot and return as base64 string
    pub async fn take_screenshot_base64(
        &self,
        page: &crate::actor::Page,
        full_page: bool,
    ) -> Result<String> {
        let data_b64 = page.screenshot_with_options(None, None, full_page, None).await?;
        Ok(data_b64)
    }
}

impl Default for ScreenshotManager {
    fn default() -> Self {
        Self::new()
    }
}
