//! Tools service for action registry

use crate::error::{BrowserUseError, Result};
use crate::tools::registry::Registry;
use crate::tools::views::ActionModel;
use crate::agent::views::ActionResult;
use crate::browser::Browser;
use tracing::info;

/// Tools registry for agent actions
pub struct Tools {
    pub registry: Registry,
    pub display_files_in_done_text: bool,
}

impl Tools {
    pub fn new(exclude_actions: Vec<String>) -> Self {
        let mut registry = Registry::new(exclude_actions);
        
        // Register default actions
        Self::register_default_actions(&mut registry);
        
        Self {
            registry,
            display_files_in_done_text: true,
        }
    }

    fn register_default_actions(registry: &mut Registry) {
        // Register basic navigation actions
        registry.register_action(
            "search".to_string(),
            "Search the web using a search engine".to_string(),
            None,
        );
        
        registry.register_action(
            "navigate".to_string(),
            "Navigate to a URL".to_string(),
            None,
        );
        
        registry.register_action(
            "click".to_string(),
            "Click an element by index".to_string(),
            None,
        );
        
        registry.register_action(
            "input".to_string(),
            "Input text into a field".to_string(),
            None,
        );
        
        registry.register_action(
            "done".to_string(),
            "Mark the task as complete".to_string(),
            None,
        );
        
        // TODO: Register more default actions
    }

    pub async fn act(
        &self,
        action: ActionModel,
        browser_session: &mut crate::browser::Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        match action.action_type.as_str() {
            "search" => self.handle_search(action, browser_session).await,
            "navigate" => self.handle_navigate(action, browser_session).await,
            "click" => self.handle_click(action, browser_session, selector_map).await,
            "input" => self.handle_input(action, browser_session, selector_map).await,
            "done" => self.handle_done(action).await,
            _ => Err(BrowserUseError::Tool(format!(
                "Unknown action type: {}",
                action.action_type
            ))),
        }
    }

    async fn handle_search(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        // Parse search params
        let query = action
            .params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'query' parameter".to_string()))?;
        let engine = action
            .params
            .get("engine")
            .and_then(|v| v.as_str())
            .unwrap_or("duckduckgo");

        // Build search URL
        let encoded_query = urlencoding::encode(query);
        let search_url = match engine.to_lowercase().as_str() {
            "duckduckgo" => format!("https://duckduckgo.com/?q={}", encoded_query),
            "google" => format!("https://www.google.com/search?q={}&udm=14", encoded_query),
            "bing" => format!("https://www.bing.com/search?q={}", encoded_query),
            _ => {
                return Err(BrowserUseError::Tool(format!(
                    "Unsupported search engine: {}. Options: duckduckgo, google, bing",
                    engine
                )));
            }
        };

        // Navigate to search URL
        browser_session.navigate(&search_url).await?;
        let memory = format!("Searched {} for '{}'", engine, query);
        info!("🔍 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_navigate(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let url = action
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'url' parameter".to_string()))?;
        let new_tab = action
            .params
            .get("new_tab")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        browser_session.navigate(url).await?;
        let memory = if new_tab {
            format!("Opened new tab with URL {}", url)
        } else {
            format!("Navigated to {}", url)
        };
        info!("🔗 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_click(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        let index = action
            .params
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'index' parameter".to_string()))?
            as u32;

        // Get backend_node_id from selector map
        let backend_node_id = if let Some(selector_map) = selector_map {
            if let Some(element) = selector_map.get(&index) {
                element.backend_node_id.unwrap_or(index)
            } else {
                // Fallback to index if element not found in selector map
                index
            }
        } else {
            // No selector map available, use index as fallback
            index
        };
        
        // Get page actor
        let page = browser_session.get_page()?;
        
        // Click element by backend_node_id
        let element = page.get_element(backend_node_id).await;
        element.click(crate::actor::mouse::MouseButton::Left, 1, None).await?;
        
        let memory = format!("Clicked element {} (backend_node_id: {})", index, backend_node_id);
        info!("🖱️ {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_input(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        let index = action
            .params
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'index' parameter".to_string()))?
            as u32;
        let text = action
            .params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'text' parameter".to_string()))?;

        // Get backend_node_id from selector map
        let backend_node_id = if let Some(selector_map) = selector_map {
            if let Some(element) = selector_map.get(&index) {
                element.backend_node_id.unwrap_or(index)
            } else {
                // Fallback to index if element not found in selector map
                index
            }
        } else {
            // No selector map available, use index as fallback
            index
        };

        // Get page actor
        let page = browser_session.get_page()?;
        
        // Input text into element by backend_node_id
        let element = page.get_element(backend_node_id).await;
        element.fill(text).await?;
        
        let memory = format!("Input text into element {} (backend_node_id: {})", index, backend_node_id);
        info!("⌨️ {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_done(&self, action: ActionModel) -> Result<ActionResult> {
        let text = action
            .params
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("Task completed");

        info!("✅ {}", text);
        Ok(ActionResult {
            extracted_content: Some(text.to_string()),
            is_done: Some(true),
            success: Some(true),
            ..Default::default()
        })
    }
}

impl Default for Tools {
    fn default() -> Self {
        Self::new(vec![])
    }
}

