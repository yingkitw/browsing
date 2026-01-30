//! Advanced action handlers

use super::Handler;
use crate::agent::views::ActionResult;
use crate::error::{BrowsingError, Result};
use crate::tools::views::{ActionContext, ActionParams};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tracing::info;

pub struct AdvancedHandler;

#[async_trait]
impl Handler for AdvancedHandler {
    async fn handle(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        match params.get_action_type().unwrap_or("unknown") {
            "done" => self.done(params).await,
            "evaluate" => self.evaluate(params, context).await,
            "upload_file" => self.upload_file(params, context).await,
            "wait" => self.wait(params).await,
            _ => Err(BrowsingError::Tool("Unknown advanced action".into())),
        }
    }
}

impl AdvancedHandler {
    async fn done(&self, params: &ActionParams<'_>) -> Result<ActionResult> {
        let text = params.get_required_str("text").unwrap_or("Task completed");
        info!("✅ {}", text);
        Ok(ActionResult {
            extracted_content: Some(text.to_string()),
            is_done: Some(true),
            success: Some(true),
            ..Default::default()
        })
    }

    async fn evaluate(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let expression = params.get_required_str("expression")?;

        let dangerous_patterns = [
            "document.cookie", "localStorage.", "sessionStorage.", "window.location",
            "fetch(", "XMLHttpRequest", "eval(", "Function(", "setTimeout(", "setInterval(",
            "<script", "javascript:", "data:",
        ];

        for pattern in dangerous_patterns {
            if expression.to_lowercase().contains(pattern) {
                return Err(BrowsingError::Tool(format!("Potentially dangerous JavaScript detected: {}", pattern)));
            }
        }

        let page = context.browser.get_page()?;
        let result = page.evaluate(expression).await?;

        let memory = format!("Evaluated JavaScript: {}", expression);
        info!("💻 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(result),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn upload_file(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let index = params.get_required_u32("index")?;
        let path = params.get_required_str("path")?;

        if path.contains("..") || path.contains("~") {
            return Err(BrowsingError::Tool("Invalid file path: path traversal not allowed".into()));
        }

        let absolute_path = Path::new(path).canonicalize()
            .map_err(|_| BrowsingError::Tool("Invalid file path: cannot resolve to absolute path".into()))?;

        if !absolute_path.exists() {
            return Err(BrowsingError::Tool(format!("File {} does not exist", path)));
        }

        if !absolute_path.is_file() {
            return Err(BrowsingError::Tool(format!("Path {} is not a file", path)));
        }

        let path_str = absolute_path.to_str()
            .ok_or_else(|| BrowsingError::Tool("Invalid file path: non-UTF8 characters".into()))?;

        let element = context.selector_map.and_then(|map| map.get(&index))
            .ok_or_else(|| BrowsingError::Tool(format!("Element index {} not found", index)))?;

        let client = context.browser.get_cdp_client()?;
        let backend_node_id = element.backend_node_id.ok_or_else(|| {
            BrowsingError::Tool(format!("Element index {} has no backend_node_id", index))
        })?;

        let node_id = {
            let result = client.send_command("DOM.pushNodesByBackendIdsToFrontend", json!({
                "backendNodeIds": [backend_node_id]
            })).await?;
            let node_ids = result.get("nodeIds").and_then(|v| v.as_array())
                .ok_or_else(|| BrowsingError::Dom("No nodeIds in response".into()))?;
            node_ids.first().and_then(|v| v.as_u64())
                .ok_or_else(|| BrowsingError::Dom("Invalid nodeId".into()))? as u32
        };

        let session_id = context.browser.get_session_id()?;
        client.send_command_with_session("DOM.setFileInputFiles", json!({
            "nodeId": node_id,
            "files": [path_str]
        }), Some(&session_id)).await
            .map_err(|e| BrowsingError::Tool(format!("Failed to upload file: {}", e)))?;

        let memory = format!("Uploaded file {} to element {}", path_str, index);
        info!("📁 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(format!("Successfully uploaded file to index {}", index)),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn wait(&self, params: &ActionParams<'_>) -> Result<ActionResult> {
        let seconds = params.get_optional_u64("seconds").unwrap_or(3);
        let actual_seconds = seconds.min(30);

        tokio::time::sleep(tokio::time::Duration::from_secs(actual_seconds)).await;

        let memory = format!("Waited for {} seconds", seconds);
        info!("🕒 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }
}
