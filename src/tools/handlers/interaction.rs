//! Interaction action handlers

use super::Handler;
use crate::agent::views::ActionResult;
use crate::error::{BrowsingError, Result};
use crate::tools::views::{ActionContext, ActionParams};
use async_trait::async_trait;
use tracing::info;

/// Handler for user interaction actions
/// Handles click, input, and send_keys operations
pub struct InteractionHandler;

#[async_trait]
impl Handler for InteractionHandler {
    async fn handle(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        match params.get_action_type().unwrap_or("unknown") {
            "click" => self.click(params, context).await,
            "input" => self.input(params, context).await,
            "send_keys" => self.send_keys(params, context).await,
            _ => Err(BrowsingError::Tool("Unknown interaction action".into())),
        }
    }
}

impl InteractionHandler {
    async fn click(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let index = params.get_required_u32("index")?;
        let backend_node_id = params.backend_node_id_from_index(index, context.selector_map);

        let page = context.browser.get_page()?;
        let element = page.get_element(backend_node_id).await;
        element.click(crate::actor::mouse::MouseButton::Left, 1, None).await?;

        let memory = format!("Clicked element {} (backend_node_id: {})", index, backend_node_id);
        info!("🖱️ {}", memory);
        Ok(ActionResult::success_with_memory(memory))
    }

    async fn input(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let index = params.get_required_u32("index")?;
        let text = params.get_required_str("text")?;
        let backend_node_id = params.backend_node_id_from_index(index, context.selector_map);

        let page = context.browser.get_page()?;
        let element = page.get_element(backend_node_id).await;
        element.fill(text).await?;

        let memory = format!("Input text into element {} (backend_node_id: {})", index, backend_node_id);
        info!("⌨️ {}", memory);
        Ok(ActionResult::success_with_memory(memory))
    }

    async fn send_keys(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let keys = params.get_required_str("keys")?;
        let page = context.browser.get_page()?;

        for key in keys.split_whitespace() {
            page.press(key).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        let memory = format!("Sent keys: {}", keys);
        info!("⌨️ {}", memory);
        Ok(ActionResult::success_with_memory(memory))
    }
}
