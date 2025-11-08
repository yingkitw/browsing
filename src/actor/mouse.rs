//! Mouse operations for browser automation

use crate::error::{BrowserUseError, Result};
use crate::browser::cdp::CdpClient;
use serde_json::json;
use std::sync::Arc;

/// Mouse button types
#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub fn to_cdp_string(&self) -> &'static str {
        match self {
            MouseButton::Left => "left",
            MouseButton::Right => "right",
            MouseButton::Middle => "middle",
        }
    }
}

/// Mouse operations for a target
pub struct Mouse {
    client: Arc<CdpClient>,
    session_id: String,
}

impl Mouse {
    pub fn new(client: Arc<CdpClient>, session_id: String) -> Self {
        Self { client, session_id }
    }

    /// Click at the specified coordinates
    pub async fn click(
        &self,
        x: f64,
        y: f64,
        button: MouseButton,
        click_count: u32,
    ) -> Result<()> {
        // Mouse press
        let press_params = json!({
            "type": "mousePressed",
            "x": x,
            "y": y,
            "button": button.to_cdp_string(),
            "clickCount": click_count,
        });
        self.client
            .send_command("Input.dispatchMouseEvent", press_params)
            .await?;

        // Small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Mouse release
        let release_params = json!({
            "type": "mouseReleased",
            "x": x,
            "y": y,
            "button": button.to_cdp_string(),
            "clickCount": click_count,
        });
        self.client
            .send_command("Input.dispatchMouseEvent", release_params)
            .await?;

        Ok(())
    }

    /// Press mouse button down
    pub async fn down(&self, button: MouseButton, click_count: u32) -> Result<()> {
        let params = json!({
            "type": "mousePressed",
            "x": 0,
            "y": 0,
            "button": button.to_cdp_string(),
            "clickCount": click_count,
        });
        self.client
            .send_command("Input.dispatchMouseEvent", params)
            .await?;
        Ok(())
    }

    /// Release mouse button
    pub async fn up(&self, button: MouseButton, click_count: u32) -> Result<()> {
        let params = json!({
            "type": "mouseReleased",
            "x": 0,
            "y": 0,
            "button": button.to_cdp_string(),
            "clickCount": click_count,
        });
        self.client
            .send_command("Input.dispatchMouseEvent", params)
            .await?;
        Ok(())
    }

    /// Move mouse to the specified coordinates
    pub async fn r#move(&self, x: f64, y: f64) -> Result<()> {
        let params = json!({
            "type": "mouseMoved",
            "x": x,
            "y": y,
        });
        self.client
            .send_command("Input.dispatchMouseEvent", params)
            .await?;
        Ok(())
    }

    /// Scroll the page
    pub async fn scroll(
        &self,
        x: f64,
        y: f64,
        delta_x: Option<f64>,
        delta_y: Option<f64>,
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

        let scroll_x = if x > 0.0 { x } else { viewport_width / 2.0 };
        let scroll_y = if y > 0.0 { y } else { viewport_height / 2.0 };

        let delta_x = delta_x.unwrap_or(0.0);
        let delta_y = delta_y.unwrap_or(0.0);

        // Try mouse wheel event
        let params = json!({
            "type": "mouseWheel",
            "x": scroll_x,
            "y": scroll_y,
            "deltaX": delta_x,
            "deltaY": delta_y,
        });

        match self
            .client
            .send_command("Input.dispatchMouseEvent", params)
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback to JavaScript scroll
                let scroll_js = format!("window.scrollBy({}, {})", delta_x, delta_y);
                let eval_params = json!({
                    "expression": scroll_js,
                    "returnByValue": true,
                });
                self.client
                    .send_command("Runtime.evaluate", eval_params)
                    .await?;
                Ok(())
            }
        }
    }
}

