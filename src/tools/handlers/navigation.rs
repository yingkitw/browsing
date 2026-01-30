//! Navigation action handlers
//!
//! Handlers for search, navigate, and go_back actions.

use super::Handler;
use crate::agent::views::ActionResult;
use crate::error::{BrowsingError, Result};
use crate::tools::views::{ActionContext, ActionParams};
use async_trait::async_trait;
use tracing::info;

/// Handler for navigation actions
pub struct NavigationHandler;

#[async_trait]
impl Handler for NavigationHandler {
    async fn handle(
        &self,
        params: &ActionParams<'_>,
        context: &mut ActionContext<'_>,
    ) -> Result<ActionResult> {
        let action_type = params.get_action_type().unwrap_or("unknown");

        match action_type {
            "search" => self.search(params, context).await,
            "navigate" => self.navigate(params, context).await,
            "go_back" => self.go_back(context).await,
            _ => Err(BrowsingError::Tool(format!(
                "Unknown navigation action: {action_type}"
            ))),
        }
    }
}

impl NavigationHandler {
    /// Search the web using a search engine
    async fn search(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let query = params.get_required_str("query")?;
        let engine = params
            .get_required_str("engine")
            .unwrap_or("duckduckgo");

        let encoded_query = urlencoding::encode(query);
        let search_url = match engine.to_lowercase().as_str() {
            "duckduckgo" => format!("https://duckduckgo.com/?q={encoded_query}"),
            "google" => format!("https://www.google.com/search?q={encoded_query}&udm=14"),
            "bing" => format!("https://www.bing.com/search?q={encoded_query}"),
            _ => {
                return Err(BrowsingError::Tool(format!(
                    "Unsupported search engine: {}. Options: duckduckgo, google, bing",
                    engine
                )))
            }
        };

        context.browser.navigate(&search_url).await?;
        let memory = format!("Searched {} for '{}'", engine, query);
        info!("🔍 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    /// Navigate to a URL
    async fn navigate(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let url = params.get_required_str("url")?;
        let new_tab = params.get_optional_bool("new_tab");

        if new_tab {
            let target_id = context.browser.create_tab(Some(url)).await?;
            context.browser.switch_to_tab(&target_id).await?;
            let memory = format!("Opened new tab with URL {}", url);
            info!("🔗 {}", memory);
            Ok(ActionResult {
                extracted_content: Some(memory.clone()),
                long_term_memory: Some(memory),
                ..Default::default()
            })
        } else {
            context.browser.navigate(url).await?;
            let memory = format!("Navigated to {}", url);
            info!("🔗 {}", memory);
            Ok(ActionResult {
                extracted_content: Some(memory.clone()),
                long_term_memory: Some(memory),
                ..Default::default()
            })
        }
    }

    /// Go back in browser history
    async fn go_back(&self, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        context.browser.go_back().await?;
        let memory = "Navigated back".to_string();
        info!("🔙 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }
}
