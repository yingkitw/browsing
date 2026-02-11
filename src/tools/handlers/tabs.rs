//! Tab management action handlers

use super::Handler;
use crate::agent::views::ActionResult;
use crate::error::{BrowsingError, Result};
use crate::tools::views::{ActionContext, ActionParams};
use async_trait::async_trait;
use tracing::info;

/// Handler for tab management actions
/// Handles switch, close, and create operations
pub struct TabsHandler;

#[async_trait]
impl Handler for TabsHandler {
    async fn handle(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        match params.get_action_type().unwrap_or("unknown") {
            "switch" => self.switch_tab(params, context).await,
            "close" => self.close_tab(params, context).await,
            _ => Err(BrowsingError::Tool("Unknown tabs action".into())),
        }
    }
}

impl TabsHandler {
    async fn switch_tab(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let tab_id = params.get_required_str("tab_id")?;
        let target_id = self.get_target_id_from_tab_id(context, tab_id).await?;
        context.browser.switch_to_tab(&target_id).await?;

        let current_url = context.browser.get_current_url().await.unwrap_or_default();
        let memory = format!("Switched to tab #{} (URL: {})", tab_id, current_url);
        info!("🔄 {}", memory);
        Ok(ActionResult::success_with_memory(memory))
    }

    async fn close_tab(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let tab_id = params.get_required_str("tab_id")?;
        let target_id = self.get_target_id_from_tab_id(context, tab_id).await?;
        context.browser.close_tab(&target_id).await?;

        let current_url = context.browser.get_current_url().await.unwrap_or_default();
        let memory = format!("Closed tab #{}, now on {}", tab_id, current_url);
        info!("❌ {}", memory);
        Ok(ActionResult::success_with_memory(memory))
    }

    async fn get_target_id_from_tab_id(&self, context: &mut ActionContext<'_>, tab_id: &str) -> Result<String> {
        let tabs = context.browser.get_tabs().await?;
        for tab in tabs {
            if tab.target_id.ends_with(tab_id) {
                return Ok(tab.target_id);
            }
        }
        Err(BrowsingError::Browser(format!("No target ID found ending with {}", tab_id)))
    }
}
