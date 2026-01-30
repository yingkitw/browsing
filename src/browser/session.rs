//! Browser session management using CDP

use crate::browser::cdp::{CdpClient, CdpSession};
use crate::browser::navigation::NavigationManager;
use crate::browser::profile::BrowserProfile;
use crate::browser::screenshot::ScreenshotManager;
use crate::browser::tab_manager::TabManager;
use crate::error::{BrowsingError, Result};
use crate::traits::BrowserClient;
use async_trait::async_trait;
use std::sync::Arc;

/// Browser session for managing CDP connections
pub struct Browser {
    profile: BrowserProfile,
    cdp_client: Option<Arc<CdpClient>>,
    cdp_url: Option<String>,
    tab_manager: TabManager,
    navigation_manager: NavigationManager,
    screenshot_manager: ScreenshotManager,
    launcher: Option<crate::browser::launcher::BrowserLauncher>,
}

impl Browser {
    /// Create a new Browser session with given profile
    pub fn new(profile: BrowserProfile) -> Self {
        Self {
            profile,
            cdp_client: None,
            cdp_url: None,
            tab_manager: TabManager::new(),
            navigation_manager: NavigationManager::new(),
            screenshot_manager: ScreenshotManager::new(),
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
            self.cdp_client = Some(Arc::clone(&client_arc));

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
                        self.tab_manager.set_current_target_id(target_id.to_string());
                        self.tab_manager.insert_session(target_id.to_string(), session);
                    }
                }
            }
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
            self.cdp_client = Some(Arc::clone(&client_arc));

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
                        self.tab_manager.set_current_target_id(target_id.to_string());
                        self.tab_manager.insert_session(target_id.to_string(), session);
                    }
                }
            }
        }

        Ok(())
    }

    /// Navigate to the specified URL
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        let page = self.get_page()?;
        self.navigation_manager.navigate(&page, url).await
    }

    /// Get the current page URL
    pub async fn get_current_url(&self) -> Result<String> {
        if let Some(target_id) = self.tab_manager.current_target_id() {
            if let Some(session) = self.tab_manager.get_session(target_id) {
                return Ok(session.url.clone());
            }
        }
        Err(BrowsingError::Browser("No active session".to_string()))
    }

    /// Stop the browser session and clean up resources
    pub async fn stop(&mut self) -> Result<()> {
        // Stop launcher if present
        if let Some(ref mut launcher) = self.launcher {
            launcher.stop().await?;
        }

        // Clear managers
        self.tab_manager = TabManager::new();
        self.cdp_client = None;
        self.launcher = None;
        Ok(())
    }

    /// Get the CDP client for the current session
    pub fn get_cdp_client(&self) -> Result<std::sync::Arc<crate::browser::cdp::CdpClient>> {
        if let Some(target_id) = self.tab_manager.current_target_id() {
            if let Some(session) = self.tab_manager.get_session(target_id) {
                return Ok(Arc::clone(&session.client));
            }
        }
        Err(BrowsingError::Browser("No active session".to_string()))
    }

    /// Get the session ID for the current target
    pub fn get_session_id(&self) -> Result<String> {
        if let Some(target_id) = self.tab_manager.current_target_id() {
            if let Some(session) = self.tab_manager.get_session(target_id) {
                return Ok(session.session_id.clone());
            }
        }
        Err(BrowsingError::Browser("No active session".to_string()))
    }

    /// Get a Page actor for the current session
    pub fn get_page(&self) -> Result<crate::actor::Page> {
        let client = self.get_cdp_client()?;
        let session_id = self.get_session_id()?;
        Ok(crate::actor::Page::new(client, session_id))
    }

    /// Get the current target ID
    pub fn get_current_target_id(&self) -> Result<String> {
        self.tab_manager
            .current_target_id()
            .map(|id| id.to_string())
            .ok_or_else(|| BrowsingError::Browser("No current target ID".to_string()))
    }

    /// Take a screenshot of the current page
    pub async fn take_screenshot(
        &self,
        path: Option<&str>,
        full_page: bool,
        format: Option<&str>,
        quality: Option<u32>,
    ) -> Result<Vec<u8>> {
        let page = self.get_page()?;
        self.screenshot_manager
            .take_screenshot(&page, path, full_page, format, quality)
            .await
    }

    /// Get all open tabs
    pub async fn get_tabs(&self) -> Result<Vec<crate::browser::views::TabInfo>> {
        let client = self.get_cdp_client()?;
        self.tab_manager.get_tabs(&client).await
    }

    /// Create a new tab
    pub async fn create_new_tab(&mut self, url: Option<&str>) -> Result<String> {
        let client = self.get_cdp_client()?;
        self.tab_manager.create_tab(&client, url).await
    }

    /// Switch to a different tab by target ID
    pub async fn switch_to_tab(&mut self, target_id: &str) -> Result<()> {
        let client = self.get_cdp_client()?;
        self.tab_manager.switch_to_tab(&client, target_id).await
    }

    /// Close a tab by target ID
    pub async fn close_tab(&mut self, target_id: &str) -> Result<()> {
        let client = self.get_cdp_client()?;
        self.tab_manager.close_tab(&client, target_id).await
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

        Err(BrowsingError::Browser(format!(
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

// BrowserClient trait implementation with proper delegation to managers
#[async_trait]
impl BrowserClient for Browser {
    async fn start(&mut self) -> Result<()> {
        self.start().await
    }

    async fn navigate(&mut self, url: &str) -> Result<()> {
        let page = self.get_page()?;
        self.navigation_manager.navigate(&page, url).await
    }

    async fn go_back(&mut self) -> Result<()> {
        let page = self.get_page()?;
        self.navigation_manager.go_back(&page).await
    }

    async fn get_current_url(&self) -> Result<String> {
        self.get_current_url().await
    }

    async fn create_tab(&mut self, url: Option<&str>) -> Result<String> {
        let client = self.get_cdp_client()?;
        self.tab_manager.create_tab(&client, url).await
    }

    async fn switch_to_tab(&mut self, target_id: &str) -> Result<()> {
        let client = self.get_cdp_client()?;
        self.tab_manager.switch_to_tab(&client, target_id).await
    }

    async fn close_tab(&mut self, target_id: &str) -> Result<()> {
        let client = self.get_cdp_client()?;
        self.tab_manager.close_tab(&client, target_id).await
    }

    async fn get_tabs(&self) -> Result<Vec<crate::browser::views::TabInfo>> {
        let client = self.get_cdp_client()?;
        self.tab_manager.get_tabs(&client).await
    }

    async fn get_target_id_from_tab_id(&self, tab_id: &str) -> Result<String> {
        self.get_target_id_from_tab_id(tab_id).await
    }

    fn get_page(&self) -> Result<crate::actor::Page> {
        let client = self.get_cdp_client()?;
        let session_id = self.get_session_id()?;
        Ok(crate::actor::Page::new(client, session_id))
    }

    async fn take_screenshot(&self, path: Option<&str>, full_page: bool) -> Result<Vec<u8>> {
        let page = self.get_page()?;
        self.screenshot_manager.take_screenshot(&page, path, full_page, None, None).await
    }

    async fn get_current_page_title(&self) -> Result<String> {
        self.get_current_page_title().await
    }

    fn get_cdp_client(&self) -> Result<Arc<CdpClient>> {
        self.get_cdp_client()
    }

    fn get_session_id(&self) -> Result<String> {
        self.get_session_id()
    }

    fn get_current_target_id(&self) -> Result<String> {
        self.get_current_target_id()
    }
}
