//! Tab management for browser sessions
//!
//! This module handles tab creation, switching, and closing operations.

use crate::browser::cdp::{CdpClient, CdpSession};
use crate::error::{BrowsingError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Manager for browser tab operations
pub struct TabManager {
    sessions: HashMap<String, CdpSession>,
    current_target_id: Option<String>,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            current_target_id: None,
        }
    }

    /// Get all open tabs
    pub async fn get_tabs(&self, client: &Arc<CdpClient>) -> Result<Vec<crate::browser::views::TabInfo>> {
        let targets = client
            .send_command("Target.getTargets", serde_json::json!({}))
            .await?;

        let target_infos = targets
            .get("targetInfos")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BrowsingError::Browser("No targetInfos in response".to_string()))?;

        let mut tabs = Vec::new();

        for target_info in target_infos {
            let target_type = target_info.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if target_type != "page" && target_type != "tab" {
                continue;
            }

            tabs.push(crate::browser::views::TabInfo {
                url: target_info.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                title: target_info.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                target_id: target_info.get("targetId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                parent_target_id: None,
            });
        }

        Ok(tabs)
    }

    /// Create a new tab
    pub async fn create_tab(
        &mut self,
        client: &Arc<CdpClient>,
        url: Option<&str>,
    ) -> Result<String> {
        let target_url = url.unwrap_or("about:blank");
        let params = serde_json::json!({ "url": target_url });

        let result = client.send_command("Target.createTarget", params).await?;

        let target_id = result
            .get("targetId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                BrowsingError::Browser("No targetId in createTarget response".to_string())
            })?
            .to_string();

        // Wait for target to be ready
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create session for the new target
        let session = CdpSession::for_target(client.clone(), target_id.clone(), None).await?;

        // Add to sessions map
        self.sessions.insert(target_id.clone(), session);

        info!("Created new tab with target_id: {}", target_id);
        Ok(target_id)
    }

    /// Switch to a different tab by target ID
    pub async fn switch_to_tab(
        &mut self,
        client: &Arc<CdpClient>,
        target_id: &str,
    ) -> Result<()> {
        // Verify target exists
        let targets = client
            .send_command("Target.getTargets", serde_json::json!({}))
            .await?;

        let target_exists = targets
            .get("targetInfos")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().any(|t| {
                    t.get("targetId")
                        .and_then(|v| v.as_str())
                        .map(|id| id == target_id)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !target_exists {
            return Err(BrowsingError::Browser(format!("Target {} not found", target_id)));
        }

        // Create or get session for this target
        let session = CdpSession::for_target(client.clone(), target_id.to_string(), None).await?;

        // Update current target
        self.current_target_id = Some(target_id.to_string());
        self.sessions.insert(target_id.to_string(), session);

        info!("Switched to tab with target_id: {}", target_id);
        Ok(())
    }

    /// Close a tab by target ID
    pub async fn close_tab(&mut self, client: &Arc<CdpClient>, target_id: &str) -> Result<()> {
        let params = serde_json::json!({ "targetId": target_id });
        client.send_command("Target.closeTarget", params).await?;

        // Remove from sessions
        self.sessions.remove(target_id);

        // If this was the current target, switch to another one
        if self
            .current_target_id
            .as_ref()
            .map(|id| id == target_id)
            .unwrap_or(false)
        {
            self.current_target_id = None;
        }

        info!("Closed tab with target_id: {}", target_id);
        Ok(())
    }

    /// Get the current target ID
    pub fn current_target_id(&self) -> Option<&str> {
        self.current_target_id.as_deref()
    }

    /// Set the current target ID
    pub fn set_current_target_id(&mut self, target_id: String) {
        self.current_target_id = Some(target_id);
    }

    /// Get a session by target ID
    pub fn get_session(&self, target_id: &str) -> Option<&CdpSession> {
        self.sessions.get(target_id)
    }

    /// Insert a session
    pub fn insert_session(&mut self, target_id: String, session: CdpSession) {
        self.sessions.insert(target_id, session);
    }

    /// Get all sessions
    pub fn sessions(&self) -> &HashMap<String, CdpSession> {
        &self.sessions
    }

    /// Check if there's an active session
    pub fn has_active_session(&self) -> bool {
        self.current_target_id
            .as_ref()
            .is_some_and(|id| self.sessions.contains_key(id))
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}
