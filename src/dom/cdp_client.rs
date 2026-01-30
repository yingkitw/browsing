//! CDP client wrapper for DOM operations
//!
//! This module provides a wrapper around CDP operations for DOM extraction.

use crate::browser::cdp::CdpClient;
use crate::error::{BrowsingError, Result};
use futures::future::try_join4;
use serde_json::Value;
use std::sync::Arc;

/// CDP client wrapper for DOM operations
pub struct DOMCDPClient {
    client: Arc<CdpClient>,
    session_id: Option<String>,
}

impl DOMCDPClient {
    /// Create a new DOM CDP client wrapper
    pub fn new(client: Arc<CdpClient>, session_id: Option<String>) -> Self {
        Self { client, session_id }
    }

    /// Get all trees (snapshot, DOM tree, AX tree, device pixel ratio) for a target
    pub async fn get_all_trees(
        &self,
        target_id: &str,
    ) -> Result<(Value, Value, Value, f64)> {
        let session_id = self.session_id.as_deref();

        // Required computed styles for snapshot
        let required_computed_styles = vec![
            "display", "visibility", "opacity", "overflow", "overflow-x",
            "overflow-y", "position", "z-index", "transform", "transform-origin",
        ];

        // Create snapshot request
        let snapshot_params = serde_json::json!({
            "computedStyles": required_computed_styles,
            "includePaintOrder": true,
            "includeDOMRects": true,
            "includeBlendedBackgroundColors": false,
            "includeTextColorOpacities": false,
        });

        // Create DOM tree request
        let dom_tree_params = serde_json::json!({
            "depth": -1,
            "pierce": true
        });

        // Create accessibility tree request
        let ax_tree_params = serde_json::json!({});

        // Execute all requests in parallel
        let snapshot_fut = self.client.send_command_with_session(
            "DOMSnapshot.captureSnapshot",
            snapshot_params,
            session_id,
        );
        let dom_tree_fut = self.client.send_command_with_session(
            "DOM.getDocument",
            dom_tree_params,
            session_id,
        );
        let ax_tree_fut = self.client.send_command_with_session(
            "Accessibility.getFullAXTree",
            ax_tree_params,
            session_id,
        );
        let viewport_fut = self.get_viewport_ratio(target_id);

        // Wait for all with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            try_join4(snapshot_fut, dom_tree_fut, ax_tree_fut, viewport_fut),
        )
        .await
        .map_err(|_| BrowsingError::Dom("Timeout waiting for CDP responses".to_string()))??;

        Ok(result)
    }

    /// Get viewport ratio (device pixel ratio)
    async fn get_viewport_ratio(&self, _target_id: &str) -> Result<f64> {
        let session_id = self.session_id.as_deref();

        // Get layout metrics
        let metrics = self.client.send_command_with_session(
            "Page.getLayoutMetrics",
            serde_json::json!({}),
            session_id,
        ).await?;

        // Extract device pixel ratio
        if let Some(visual_viewport) = metrics.get("visualViewport") {
            if let Some(css_visual_viewport) = metrics.get("cssVisualViewport") {
                let device_width = visual_viewport
                    .get("clientWidth")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1920.0);
                let css_width = css_visual_viewport
                    .get("clientWidth")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1920.0);

                if css_width > 0.0 {
                    return Ok(device_width / css_width);
                }
            }
        }

        // Fallback to default
        Ok(1.0)
    }

    /// Get the CDP client
    pub fn client(&self) -> &Arc<CdpClient> {
        &self.client
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}
