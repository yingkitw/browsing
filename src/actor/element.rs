//! Element operations for browser automation

use crate::error::{BrowserUseError, Result};
use crate::browser::cdp::CdpClient;
use crate::actor::mouse::MouseButton;
use serde_json::json;
use std::sync::Arc;

/// Element operations using BackendNodeId
pub struct Element {
    client: Arc<CdpClient>,
    session_id: String,
    backend_node_id: u32,
}

impl Element {
    pub fn new(client: Arc<CdpClient>, session_id: String, backend_node_id: u32) -> Self {
        Self {
            client,
            session_id,
            backend_node_id,
        }
    }

    /// Get DOM node ID from backend node ID
    async fn get_node_id(&self) -> Result<u32> {
        let params = json!({
            "backendNodeIds": [self.backend_node_id]
        });
        let result = self
            .client
            .send_command("DOM.pushNodesByBackendIdsToFrontend", params)
            .await?;
        let node_ids = result
            .get("nodeIds")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BrowserUseError::Dom("No nodeIds in response".to_string()))?;
        let node_id = node_ids
            .first()
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowserUseError::Dom("Invalid nodeId".to_string()))?;
        Ok(node_id as u32)
    }

    /// Click the element
    pub async fn click(
        &self,
        button: MouseButton,
        click_count: u32,
        modifiers: Option<Vec<String>>,
    ) -> Result<()> {
        // Get viewport dimensions
        let layout_metrics = self
            .client
            .send_command("Page.getLayoutMetrics", json!({}))
            .await?;

        let viewport_width = layout_metrics
            .get("layoutViewport")
            .and_then(|v| v.get("clientWidth"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1920.0);
        let viewport_height = layout_metrics
            .get("layoutViewport")
            .and_then(|v| v.get("clientHeight"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1080.0);

        // Try to get element geometry
        let mut center_x = viewport_width / 2.0;
        let mut center_y = viewport_height / 2.0;

        // Try DOM.getContentQuads first
        let quads_result = self
            .client
            .send_command(
                "DOM.getContentQuads",
                json!({ "backendNodeId": self.backend_node_id }),
            )
            .await;

        if let Ok(quads_result) = quads_result {
            if let Some(quads) = quads_result.get("quads").and_then(|v| v.as_array()) {
                if let Some(first_quad) = quads.first().and_then(|v| v.as_array()) {
                    if first_quad.len() >= 8 {
                        // Calculate center of quad
                        let x_coords: Vec<f64> = first_quad
                            .iter()
                            .step_by(2)
                            .filter_map(|v| v.as_f64())
                            .collect();
                        let y_coords: Vec<f64> = first_quad
                            .iter()
                            .skip(1)
                            .step_by(2)
                            .filter_map(|v| v.as_f64())
                            .collect();

                        if !x_coords.is_empty() && !y_coords.is_empty() {
                            center_x = x_coords.iter().sum::<f64>() / x_coords.len() as f64;
                            center_y = y_coords.iter().sum::<f64>() / y_coords.len() as f64;
                        }
                    }
                }
            }
        }

        // Ensure coordinates are within viewport
        center_x = center_x.max(0.0).min(viewport_width - 1.0);
        center_y = center_y.max(0.0).min(viewport_height - 1.0);

        // Scroll element into view
        let _ = self
            .client
            .send_command(
                "DOM.scrollIntoViewIfNeeded",
                json!({ "backendNodeId": self.backend_node_id }),
            )
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Calculate modifier bitmask
        let mut modifier_value = 0u32;
        if let Some(modifiers) = modifiers {
            for mod_str in modifiers {
                match mod_str.as_str() {
                    "Alt" => modifier_value |= 1,
                    "Control" => modifier_value |= 2,
                    "Meta" => modifier_value |= 4,
                    "Shift" => modifier_value |= 8,
                    _ => {}
                }
            }
        }

        // Move mouse to element
        let move_params = json!({
            "type": "mouseMoved",
            "x": center_x,
            "y": center_y,
        });
        let _ = self
            .client
            .send_command("Input.dispatchMouseEvent", move_params)
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Mouse down
        let press_params = json!({
            "type": "mousePressed",
            "x": center_x,
            "y": center_y,
            "button": button.to_cdp_string(),
            "clickCount": click_count,
            "modifiers": modifier_value,
        });
        let _ = self
            .client
            .send_command("Input.dispatchMouseEvent", press_params)
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;

        // Mouse up
        let release_params = json!({
            "type": "mouseReleased",
            "x": center_x,
            "y": center_y,
            "button": button.to_cdp_string(),
            "clickCount": click_count,
            "modifiers": modifier_value,
        });
        self.client
            .send_command("Input.dispatchMouseEvent", release_params)
            .await?;

        Ok(())
    }

    /// Fill the element with text (clears first, then types)
    pub async fn fill(&self, text: &str) -> Result<()> {
        // Focus the element
        let node_id = self.get_node_id().await?;
        let focus_params = json!({ "nodeId": node_id });
        let _ = self
            .client
            .send_command("DOM.focus", focus_params)
            .await;

        // Clear and set value using JavaScript
        let script = format!(
            r#"
            (() => {{
                const node = arguments[0];
                node.value = '';
                node.focus();
                node.value = {};
                node.dispatchEvent(new Event('input', {{ bubbles: true }}));
                node.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return node.value;
            }})
            "#,
            serde_json::to_string(text)?
        );

        let eval_params = json!({
            "expression": script,
            "returnByValue": true,
        });
        self.client
            .send_command("Runtime.evaluate", eval_params)
            .await?;

        Ok(())
    }

    /// Get element text content
    pub async fn text(&self) -> Result<String> {
        let node_id = self.get_node_id().await?;
        let script = format!(
            r#"
            (() => {{
                const node = arguments[0];
                return node.textContent || node.innerText || '';
            }})
            "#
        );

        let eval_params = json!({
            "expression": script,
            "returnByValue": true,
        });
        let result = self
            .client
            .send_command("Runtime.evaluate", eval_params)
            .await?;

        let text = result
            .get("result")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(text)
    }

    /// Get element bounding box
    pub async fn get_bounding_box(&self) -> Result<Option<(f64, f64, f64, f64)>> {
        // Try DOM.getContentQuads first
        let quads_result = self
            .client
            .send_command(
                "DOM.getContentQuads",
                json!({ "backendNodeId": self.backend_node_id }),
            )
            .await;

        if let Ok(quads_result) = quads_result {
            if let Some(quads) = quads_result.get("quads").and_then(|v| v.as_array()) {
                if let Some(first_quad) = quads.first().and_then(|v| v.as_array()) {
                    if first_quad.len() >= 8 {
                        // Calculate bounding box from quad
                        let x_coords: Vec<f64> = first_quad
                            .iter()
                            .step_by(2)
                            .filter_map(|v| v.as_f64())
                            .collect();
                        let y_coords: Vec<f64> = first_quad
                            .iter()
                            .skip(1)
                            .step_by(2)
                            .filter_map(|v| v.as_f64())
                            .collect();

                        if !x_coords.is_empty() && !y_coords.is_empty() {
                            let min_x = x_coords.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                            let max_x = x_coords.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                            let min_y = y_coords.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                            let max_y = y_coords.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                            
                            let width = max_x - min_x;
                            let height = max_y - min_y;
                            
                            return Ok(Some((min_x, min_y, width, height)));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Take a screenshot of this element
    pub async fn screenshot(&self, format: Option<&str>, quality: Option<u32>) -> Result<String> {
        // Get element's bounding box
        let (x, y, width, height) = self
            .get_bounding_box()
            .await?
            .ok_or_else(|| BrowserUseError::Browser("Element is not visible or has no bounding box".to_string()))?;

        let format = format.unwrap_or("png");
        let mut params = json!({
            "format": format,
            "clip": {
                "x": x,
                "y": y,
                "width": width,
                "height": height,
                "scale": 1.0
            }
        });

        if format == "jpeg" {
            if let Some(q) = quality {
                params["quality"] = json!(q);
            }
        }

        let result = self
            .client
            .send_command_with_session("Page.captureScreenshot", params, Some(&self.session_id))
            .await?;

        let data = result
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Browser("No screenshot data".to_string()))?;

        Ok(data.to_string())
    }
}

