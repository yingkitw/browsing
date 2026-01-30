//! Browser session management using CDP

use crate::browser::cdp::{CdpClient, CdpSession};
use crate::browser::profile::BrowserProfile;
use crate::error::{BrowserUseError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Browser session for managing CDP connections
pub struct Browser {
    profile: BrowserProfile,
    cdp_client: Option<CdpClient>,
    cdp_url: Option<String>,
    sessions: HashMap<String, CdpSession>,
    current_target_id: Option<String>,
    launcher: Option<crate::browser::launcher::BrowserLauncher>,
}

impl Browser {
    /// Create a new Browser session with given profile
    pub fn new(profile: BrowserProfile) -> Self {
        Self {
            profile,
            cdp_client: None,
            cdp_url: None,
            sessions: HashMap::new(),
            current_target_id: None,
            launcher: None,
        }
    }

    /// Set CDP URL to connect to existing browser instead of launching new one
    pub fn with_cdp_url(mut self, cdp_url: String) -> Self {
        self.cdp_url = Some(cdp_url);
        self
    }

    /// Start the browser session (launches browser or connects to existing)
    pub async fn start(&mut self) -> Result<()> {
        // If cdp_url is provided, connect to existing browser
        if let Some(ref cdp_url) = self.cdp_url {
            let mut client = CdpClient::new(cdp_url.clone());
            client.start().await?;
            let client_arc = Arc::new(client);

            // Get available targets
            let targets = client_arc
                .send_command("Target.getTargets", serde_json::json!({}))
                .await?;

            if let Some(target_infos) = targets["targetInfos"].as_array() {
                if let Some(first_target) = target_infos.first() {
                    if let Some(target_id) = first_target["targetId"].as_str() {
                        let session = CdpSession::for_target(
                            Arc::clone(&client_arc),
                            target_id.to_string(),
                            None,
                        )
                        .await?;
                        self.current_target_id = Some(target_id.to_string());
                        self.sessions.insert(target_id.to_string(), session);
                    }
                }
            }

            // Store client reference (we'll need to handle this differently)
            // For now, we'll keep it in sessions
        } else {
            // Launch browser locally
            use crate::browser::launcher::BrowserLauncher;

            let mut launcher = BrowserLauncher::new(self.profile.clone());
            let cdp_url = launcher.launch().await?;

            // Store launcher for cleanup
            self.launcher = Some(launcher);

            // Connect to the launched browser
            self.cdp_url = Some(cdp_url.clone());

            // Now connect via CDP
            let mut client = CdpClient::new(cdp_url);
            client.start().await?;
            let client_arc = Arc::new(client);

            // Get available targets
            let targets = client_arc
                .send_command("Target.getTargets", serde_json::json!({}))
                .await?;

            if let Some(target_infos) = targets["targetInfos"].as_array() {
                if let Some(first_target) = target_infos.first() {
                    if let Some(target_id) = first_target["targetId"].as_str() {
                        let session = CdpSession::for_target(
                            Arc::clone(&client_arc),
                            target_id.to_string(),
                            None,
                        )
                        .await?;
                        self.current_target_id = Some(target_id.to_string());
                        self.sessions.insert(target_id.to_string(), session);
                    }
                }
            }
        }

        Ok(())
    }

    /// Navigate to the specified URL
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        if let Some(ref target_id) = self.current_target_id {
            if let Some(session) = self.sessions.get(target_id) {
                let params = serde_json::json!({
                    "url": url
                });
                session.client.send_command("Page.navigate", params).await?;
                return Ok(());
            }
        }
        Err(BrowserUseError::Browser("No active session".to_string()))
    }

    /// Get the current page URL
    pub async fn get_current_url(&self) -> Result<String> {
        if let Some(ref target_id) = self.current_target_id {
            if let Some(session) = self.sessions.get(target_id) {
                return Ok(session.url.clone());
            }
        }
        Err(BrowserUseError::Browser("No active session".to_string()))
    }

    /// Stop the browser session and clean up resources
    pub async fn stop(&mut self) -> Result<()> {
        // Stop launcher if present
        if let Some(ref mut launcher) = self.launcher {
            launcher.stop().await?;
        }

        // Close all sessions
        self.sessions.clear();
        self.cdp_client = None;
        self.launcher = None;
        Ok(())
    }

    /// Get the CDP client for the current session
    pub fn get_cdp_client(&self) -> Result<std::sync::Arc<crate::browser::cdp::CdpClient>> {
        if let Some(ref target_id) = self.current_target_id {
            if let Some(session) = self.sessions.get(target_id) {
                return Ok(Arc::clone(&session.client));
            }
        }
        Err(BrowserUseError::Browser("No active session".to_string()))
    }

    /// Get the session ID for the current target
    pub fn get_session_id(&self) -> Result<String> {
        if let Some(ref target_id) = self.current_target_id {
            if let Some(session) = self.sessions.get(target_id) {
                return Ok(session.session_id.clone());
            }
        }
        Err(BrowserUseError::Browser("No active session".to_string()))
    }

    /// Get a Page actor for the current session
    pub fn get_page(&self) -> Result<crate::actor::Page> {
        let client = self.get_cdp_client()?;
        let session_id = self.get_session_id()?;
        Ok(crate::actor::Page::new(client, session_id))
    }

    /// Get the current target ID
    pub fn get_current_target_id(&self) -> Result<String> {
        self.current_target_id
            .clone()
            .ok_or_else(|| BrowserUseError::Browser("No current target ID".to_string()))
    }

    /// Take a screenshot of the current page
    pub async fn take_screenshot(
        &self,
        path: Option<&str>,
        full_page: bool,
        format: Option<&str>,
        quality: Option<u32>,
    ) -> Result<Vec<u8>> {
        use base64::Engine;
        use base64::engine::general_purpose;

        let page = self.get_page()?;

        // Use Page's screenshot method with full_page option (returns base64 string)
        let data_b64 = page
            .screenshot_with_options(format, quality, full_page, None)
            .await?;

        // Decode base64
        let screenshot_data = general_purpose::STANDARD
            .decode(&data_b64)
            .map_err(|e| BrowserUseError::Browser(format!("Failed to decode screenshot: {e}")))?;

        // Save to file if path provided
        if let Some(file_path) = path {
            tokio::fs::write(file_path, &screenshot_data)
                .await
                .map_err(|e| {
                    BrowserUseError::Browser(format!("Failed to save screenshot: {e}"))
                })?;
        }

        Ok(screenshot_data)
    }

    /// Get all open tabs
    pub async fn get_tabs(&self) -> Result<Vec<crate::browser::views::TabInfo>> {
        let client = self.get_cdp_client()?;

        // Get all targets
        let targets = client
            .send_command("Target.getTargets", serde_json::json!({}))
            .await?;

        let target_infos = targets
            .get("targetInfos")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BrowserUseError::Browser("No targetInfos in response".to_string()))?;

        let mut tabs = Vec::new();

        for target_info in target_infos {
            let target_type = target_info
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Only include page/tab targets, not iframes or workers
            if target_type != "page" && target_type != "tab" {
                continue;
            }

            let target_id = target_info
                .get("targetId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let url = target_info
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Try to get title from target info
            let title = target_info
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            tabs.push(crate::browser::views::TabInfo {
                url,
                title: if title.is_empty() {
                    "Unknown".to_string()
                } else {
                    title
                },
                target_id,
                parent_target_id: None,
            });
        }

        Ok(tabs)
    }

    /// Create a new tab
    pub async fn create_new_tab(&mut self, url: Option<&str>) -> Result<String> {
        let client = self.get_cdp_client()?;

        let target_url = url.unwrap_or("about:blank");

        let params = serde_json::json!({
            "url": target_url
        });

        let result = client.send_command("Target.createTarget", params).await?;

        let target_id = result
            .get("targetId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                BrowserUseError::Browser("No targetId in createTarget response".to_string())
            })?
            .to_string();

        // Wait a bit for the target to be ready
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create session for the new target
        let session = CdpSession::for_target(client, target_id.clone(), None).await?;

        // Add to sessions map
        self.sessions.insert(target_id.clone(), session);

        Ok(target_id)
    }

    /// Switch to a different tab by target ID
    pub async fn switch_to_tab(&mut self, target_id: &str) -> Result<()> {
        let client = self.get_cdp_client()?;

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
            return Err(BrowserUseError::Browser(format!(
                "Target {target_id} not found"
            )));
        }

        // Create or get session for this target
        let session = CdpSession::for_target(client, target_id.to_string(), None).await?;

        // Update current target
        self.current_target_id = Some(target_id.to_string());
        self.sessions.insert(target_id.to_string(), session);

        Ok(())
    }

    /// Close a tab by target ID
    pub async fn close_tab(&mut self, target_id: &str) -> Result<()> {
        let client = self.get_cdp_client()?;

        let params = serde_json::json!({
            "targetId": target_id
        });

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
            // Get remaining tabs
            let tabs = self.get_tabs().await?;
            if let Some(first_tab) = tabs.first() {
                self.current_target_id = Some(first_tab.target_id.clone());
            } else {
                self.current_target_id = None;
            }
        }

        Ok(())
    }

    /// Get target ID from short tab ID (last 4 characters)
    pub async fn get_target_id_from_tab_id(&self, tab_id: &str) -> Result<String> {
        let tabs = self.get_tabs().await?;

        // Try to find target ID ending with tab_id
        for tab in tabs {
            if tab.target_id.ends_with(tab_id) {
                return Ok(tab.target_id);
            }
        }

        Err(BrowserUseError::Browser(format!(
            "No target ID found ending with {tab_id}"
        )))
    }

    /// Get the current page title
    pub async fn get_current_page_title(&self) -> Result<String> {
        let client = self.get_cdp_client()?;
        let target_id = self.get_current_target_id()?;

        // Get target info
        let params = serde_json::json!({
            "targetId": target_id
        });

        let target_info = client.send_command("Target.getTargetInfo", params).await?;

        let title = target_info
            .get("targetInfo")
            .and_then(|v| v.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(title)
    }

    /// Get browser state summary for LLM consumption
    pub async fn get_browser_state_summary(
        &self,
        include_screenshot: bool,
        dom_service: Option<&crate::dom::DomService>,
    ) -> Result<crate::browser::views::BrowserStateSummary> {
        // Get current URL and title
        let url = self
            .get_current_url()
            .await
            .unwrap_or_else(|_| "about:blank".to_string());
        let title = self
            .get_current_page_title()
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

        // Get tabs
        let tabs = self.get_tabs().await.unwrap_or_default();

        // Get DOM state
        let dom_state = if let Some(dom_service) = dom_service {
            let (state, _, _) = dom_service.get_serialized_dom_tree(None).await?;
            state
        } else {
            // Fallback to empty DOM state
            crate::dom::views::SerializedDOMState {
                html: None,
                text: Some("No DOM state available".to_string()),
                markdown: None,
                elements: vec![],
                selector_map: std::collections::HashMap::new(),
            }
        };

        // Get screenshot if requested
        let screenshot = if include_screenshot {
            match self.take_screenshot(None, false, Some("png"), None).await {
                Ok(data) => {
                    use base64::{Engine as _, engine::general_purpose};
                    Some(general_purpose::STANDARD.encode(&data))
                }
                Err(_) => None,
            }
        } else {
            None
        };

        // Check if PDF viewer
        let is_pdf_viewer = url.ends_with(".pdf") || title.contains("PDF");

        Ok(crate::browser::views::BrowserStateSummary {
            dom_state,
            url,
            title,
            tabs,
            screenshot,
            page_info: None, // Can be populated with Page.getLayoutMetrics if needed
            pixels_above: 0,
            pixels_below: 0,
            browser_errors: vec![],
            is_pdf_viewer,
            recent_events: None,
            pending_network_requests: vec![],
            pagination_buttons: vec![],
            closed_popup_messages: vec![],
        })
    }
}
