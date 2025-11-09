//! Browser session management using CDP

use crate::error::{BrowserUseError, Result};
use crate::browser::profile::BrowserProfile;
use crate::browser::cdp::{CdpClient, CdpSession};
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

    pub fn with_cdp_url(mut self, cdp_url: String) -> Self {
        self.cdp_url = Some(cdp_url);
        self
    }

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
                        ).await?;
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
                        ).await?;
                        self.current_target_id = Some(target_id.to_string());
                        self.sessions.insert(target_id.to_string(), session);
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        if let Some(ref target_id) = self.current_target_id {
            if let Some(session) = self.sessions.get(target_id) {
                let params = serde_json::json!({
                    "url": url
                });
                session
                    .client
                    .send_command("Page.navigate", params)
                    .await?;
                return Ok(());
            }
        }
        Err(BrowserUseError::Browser("No active session".to_string()))
    }

    pub async fn get_current_url(&self) -> Result<String> {
        if let Some(ref target_id) = self.current_target_id {
            if let Some(session) = self.sessions.get(target_id) {
                return Ok(session.url.clone());
            }
        }
        Err(BrowserUseError::Browser("No active session".to_string()))
    }

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
}

