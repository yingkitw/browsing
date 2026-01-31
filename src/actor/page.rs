//! Page operations for browser automation

use crate::actor::{Element, Mouse, get_key_info};
use crate::browser::cdp::CdpClient;
use crate::error::{BrowsingError, Result};
use serde_json::json;
use std::sync::Arc;

/// Page operations (tab or iframe)
pub struct Page {
    client: Arc<CdpClient>,
    session_id: String,
    mouse: Option<Mouse>,
}

impl Page {
    /// Creates a new Page instance with the given CDP client and session ID
    pub fn new(client: Arc<CdpClient>, session_id: String) -> Self {
        Self {
            client,
            session_id,
            mouse: None,
        }
    }

    /// Get the mouse interface for this page
    pub async fn mouse(&mut self) -> &mut Mouse {
        if self.mouse.is_none() {
            self.mouse = Some(Mouse::new(
                Arc::clone(&self.client),
                self.session_id.clone(),
            ));
        }
        self.mouse.as_mut().unwrap()
    }

    /// Reload the page
    pub async fn reload(&self) -> Result<()> {
        self.client.send_command("Page.reload", json!({})).await?;
        Ok(())
    }

    /// Navigate to URL
    pub async fn goto(&self, url: &str) -> Result<()> {
        let params = json!({
            "url": url
        });
        self.client.send_command("Page.navigate", params).await?;
        Ok(())
    }

    /// Go back in browser history
    pub async fn go_back(&self) -> Result<()> {
        // Get navigation history to find the previous entry
        let history = self
            .client
            .send_command("Page.getNavigationHistory", json!({}))
            .await?;

        let current_index = history
            .get("currentIndex")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let entries = history
            .get("entries")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BrowsingError::Browser("No history entries found".to_string()))?;

        // Go back if not at the beginning
        if current_index > 0 {
            let previous_entry = entries
                .get(current_index - 1)
                .ok_or_else(|| BrowsingError::Browser("No previous history entry".to_string()))?;

            let entry_id = previous_entry
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| BrowsingError::Browser("No entryId in history entry".to_string()))?;

            let params = json!({
                "entryId": entry_id
            });
            self.client
                .send_command("Page.navigateToHistoryEntry", params)
                .await?;
        }

        Ok(())
    }

    /// Go forward in browser history
    pub async fn go_forward(&self) -> Result<()> {
        // Get navigation history to find the next entry
        let history = self
            .client
            .send_command("Page.getNavigationHistory", json!({}))
            .await?;

        let current_index = history
            .get("currentIndex")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let entries = history
            .get("entries")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BrowsingError::Browser("No history entries found".to_string()))?;

        // Go forward if not at the end
        if current_index + 1 < entries.len() {
            let next_entry = entries
                .get(current_index + 1)
                .ok_or_else(|| BrowsingError::Browser("No next history entry".to_string()))?;

            let entry_id = next_entry
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| BrowsingError::Browser("No entryId in history entry".to_string()))?;

            let params = json!({
                "entryId": entry_id
            });
            self.client
                .send_command("Page.navigateToHistoryEntry", params)
                .await?;
        }

        Ok(())
    }

    /// Get an element by its backend node ID
    pub async fn get_element(&self, backend_node_id: u32) -> Element {
        Element::new(
            Arc::clone(&self.client),
            self.session_id.clone(),
            backend_node_id,
        )
    }

    /// Get elements by CSS selector
    pub async fn get_elements_by_css_selector(&self, selector: &str) -> Result<Vec<Element>> {
        // First, get document node
        let document_result = self
            .client
            .send_command("DOM.getDocument", json!({ "depth": 0 }))
            .await?;

        let root_node_id = document_result
            .get("root")
            .and_then(|v| v.get("nodeId"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowsingError::Dom("No root node found".to_string()))?;

        // Query selector
        let query_params = json!({
            "nodeId": root_node_id,
            "selector": selector
        });
        let query_result = self
            .client
            .send_command("DOM.querySelectorAll", query_params)
            .await?;

        let node_ids = query_result
            .get("nodeIds")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BrowsingError::Dom("No nodeIds in query result".to_string()))?;

        let mut elements = Vec::new();
        for node_id_value in node_ids {
            if let Some(node_id) = node_id_value.as_u64() {
                // Get backend node ID
                let describe_params = json!({
                    "nodeId": node_id
                });
                if let Ok(describe_result) = self
                    .client
                    .send_command("DOM.describeNode", describe_params)
                    .await
                {
                    if let Some(backend_node_id) = describe_result
                        .get("node")
                        .and_then(|v| v.get("backendNodeId"))
                        .and_then(|v| v.as_u64())
                    {
                        elements.push(Element::new(
                            Arc::clone(&self.client),
                            self.session_id.clone(),
                            backend_node_id as u32,
                        ));
                    }
                }
            }
        }

        Ok(elements)
    }

    /// Execute JavaScript in the page
    pub async fn evaluate(&self, expression: &str) -> Result<String> {
        let params = json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true
        });
        let result = self.client.send_command("Runtime.evaluate", params).await?;

        if let Some(exception) = result.get("exceptionDetails") {
            return Err(BrowsingError::Dom(format!(
                "JavaScript evaluation failed: {exception}"
            )));
        }

        let value = result.get("result").and_then(|v| v.get("value"));

        match value {
            Some(serde_json::Value::String(s)) => Ok(s.clone()),
            Some(v) => Ok(serde_json::to_string(v)?),
            None => Ok(String::new()),
        }
    }

    /// Take a screenshot
    pub async fn screenshot(&self, format: Option<&str>, quality: Option<u32>) -> Result<String> {
        self.screenshot_with_options(format, quality, false, None)
            .await
    }

    /// Take a screenshot with additional options
    pub async fn screenshot_with_options(
        &self,
        format: Option<&str>,
        quality: Option<u32>,
        full_page: bool,
        clip: Option<(f64, f64, f64, f64)>,
    ) -> Result<String> {
        let format = format.unwrap_or("png");
        let mut params = json!({
            "format": format,
            "captureBeyondViewport": full_page
        });

        if format == "jpeg" {
            if let Some(q) = quality {
                params["quality"] = json!(q);
            }
        }

        if let Some((x, y, width, height)) = clip {
            params["clip"] = json!({
                "x": x,
                "y": y,
                "width": width,
                "height": height,
                "scale": 1.0
            });
        }

        let result = self
            .client
            .send_command_with_session("Page.captureScreenshot", params, Some(&self.session_id))
            .await?;

        let data = result
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowsingError::Browser("No screenshot data".to_string()))?;

        Ok(data.to_string())
    }

    /// Press a key on the page (supports key combinations like "Control+A")
    pub async fn press(&self, key: &str) -> Result<()> {
        // Handle key combinations like "Control+A"
        if key.contains('+') {
            let parts: Vec<&str> = key.split('+').collect();
            let modifiers = &parts[..parts.len() - 1];
            let main_key = parts.last().unwrap();

            // Calculate modifier bitmask
            let mut modifier_value = 0u32;
            let modifier_map: std::collections::HashMap<&str, u32> =
                [("Alt", 1), ("Control", 2), ("Meta", 4), ("Shift", 8)]
                    .iter()
                    .cloned()
                    .collect();

            for mod_str in modifiers {
                if let Some(&val) = modifier_map.get(mod_str) {
                    modifier_value |= val;
                }
            }

            // Press modifier keys
            for mod_str in modifiers {
                let (code, vk_code) = get_key_info(mod_str);
                let mut params = json!({
                    "type": "keyDown",
                    "key": mod_str,
                    "code": code
                });
                if let Some(vk) = vk_code {
                    params["windowsVirtualKeyCode"] = json!(vk);
                }
                self.client
                    .send_command("Input.dispatchKeyEvent", params)
                    .await?;
            }

            // Press main key with modifiers
            let (main_code, main_vk_code) = get_key_info(main_key);
            let mut main_down_params = json!({
                "type": "keyDown",
                "key": main_key,
                "code": main_code,
                "modifiers": modifier_value
            });
            if let Some(vk) = main_vk_code {
                main_down_params["windowsVirtualKeyCode"] = json!(vk);
            }
            self.client
                .send_command("Input.dispatchKeyEvent", main_down_params)
                .await?;

            let mut main_up_params = json!({
                "type": "keyUp",
                "key": main_key,
                "code": main_code,
                "modifiers": modifier_value
            });
            if let Some(vk) = main_vk_code {
                main_up_params["windowsVirtualKeyCode"] = json!(vk);
            }
            self.client
                .send_command("Input.dispatchKeyEvent", main_up_params)
                .await?;

            // Release modifier keys
            for mod_str in modifiers.iter().rev() {
                let (code, vk_code) = get_key_info(mod_str);
                let mut params = json!({
                    "type": "keyUp",
                    "key": mod_str,
                    "code": code
                });
                if let Some(vk) = vk_code {
                    params["windowsVirtualKeyCode"] = json!(vk);
                }
                self.client
                    .send_command("Input.dispatchKeyEvent", params)
                    .await?;
            }
        } else {
            // Simple key press
            let (code, vk_code) = get_key_info(key);
            let mut key_down_params = json!({
                "type": "keyDown",
                "key": key,
                "code": code
            });
            if let Some(vk) = vk_code {
                key_down_params["windowsVirtualKeyCode"] = json!(vk);
            }
            self.client
                .send_command("Input.dispatchKeyEvent", key_down_params)
                .await?;

            let mut key_up_params = json!({
                "type": "keyUp",
                "key": key,
                "code": code
            });
            if let Some(vk) = vk_code {
                key_up_params["windowsVirtualKeyCode"] = json!(vk);
            }
            self.client
                .send_command("Input.dispatchKeyEvent", key_up_params)
                .await?;
        }

        Ok(())
    }

    /// Set viewport size
    pub async fn set_viewport_size(&self, width: u32, height: u32) -> Result<()> {
        let params = json!({
            "width": width,
            "height": height,
            "deviceScaleFactor": 1.0,
            "mobile": false
        });
        self.client
            .send_command("Emulation.setDeviceMetricsOverride", params)
            .await?;
        Ok(())
    }
}
