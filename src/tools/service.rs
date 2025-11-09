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
        
        registry.register_action(
            "switch".to_string(),
            "Switch to another open tab by tab_id".to_string(),
            None,
        );
        
        registry.register_action(
            "close".to_string(),
            "Close a tab by tab_id".to_string(),
            None,
        );
        
        registry.register_action(
            "scroll".to_string(),
            "Scroll the page up or down by pages".to_string(),
            None,
        );
        
        registry.register_action(
            "go_back".to_string(),
            "Go back in browser history".to_string(),
            None,
        );
        
        registry.register_action(
            "wait".to_string(),
            "Wait for specified seconds".to_string(),
            None,
        );
        
        registry.register_action(
            "send_keys".to_string(),
            "Send keyboard keys (Enter, Escape, Tab, etc.)".to_string(),
            None,
        );
        
        registry.register_action(
            "evaluate".to_string(),
            "Execute JavaScript code on the page".to_string(),
            None,
        );
    }

    pub async fn act(
        &self,
        action: ActionModel,
        browser_session: &mut crate::browser::Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        let action_type = action.action_type.as_str();
        
        // Check if this is a custom action with a handler
        if let Some(handler) = self.registry.get_handler(action_type) {
            return handler.execute(&action.params, browser_session).await;
        }
        
        // Otherwise, use built-in handlers
        match action_type {
            "search" => self.handle_search(action, browser_session).await,
            "navigate" => self.handle_navigate(action, browser_session).await,
            "click" => self.handle_click(action, browser_session, selector_map).await,
            "input" => self.handle_input(action, browser_session, selector_map).await,
            "done" => self.handle_done(action).await,
            "switch" => self.handle_switch_tab(action, browser_session).await,
            "close" => self.handle_close_tab(action, browser_session).await,
            "scroll" => self.handle_scroll(action, browser_session).await,
            "go_back" => self.handle_go_back(action, browser_session).await,
            "wait" => self.handle_wait(action).await,
            "send_keys" => self.handle_send_keys(action, browser_session).await,
            "evaluate" => self.handle_evaluate(action, browser_session).await,
            _ => Err(BrowserUseError::Tool(format!(
                "Unknown action type: {}",
                action_type
            ))),
        }
    }

    /// Register a custom action
    pub fn register_custom_action<H: crate::tools::views::ActionHandler + 'static>(
        &mut self,
        name: String,
        description: String,
        domains: Option<Vec<String>>,
        handler: H,
    ) {
        self.registry.register_custom_action(name, description, domains, handler);
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

        if new_tab {
            // Create new tab and navigate
            let target_id = browser_session.create_new_tab(Some(url)).await?;
            browser_session.switch_to_tab(&target_id).await?;
            let memory = format!("Opened new tab with URL {}", url);
            info!("🔗 {}", memory);
            Ok(ActionResult {
                extracted_content: Some(memory.clone()),
                long_term_memory: Some(memory),
                ..Default::default()
            })
        } else {
            // Navigate in current tab
            browser_session.navigate(url).await?;
            let memory = format!("Navigated to {}", url);
            info!("🔗 {}", memory);
            Ok(ActionResult {
                extracted_content: Some(memory.clone()),
                long_term_memory: Some(memory),
                ..Default::default()
            })
        }
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

    async fn handle_switch_tab(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let tab_id = action
            .params
            .get("tab_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'tab_id' parameter".to_string()))?;

        // Get full target ID from short tab ID
        let target_id = browser_session.get_target_id_from_tab_id(tab_id).await?;
        
        // Switch to the tab
        browser_session.switch_to_tab(&target_id).await?;
        
        // Get current URL after switch
        let current_url = browser_session.get_current_url().await.unwrap_or_default();
        
        let memory = format!("Switched to tab #{} (URL: {})", tab_id, current_url);
        info!("🔄 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_close_tab(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let tab_id = action
            .params
            .get("tab_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'tab_id' parameter".to_string()))?;

        // Get full target ID from short tab ID
        let target_id = browser_session.get_target_id_from_tab_id(tab_id).await?;
        
        // Close the tab
        browser_session.close_tab(&target_id).await?;
        
        // Get current URL after close (if any tabs remain)
        let current_url = browser_session.get_current_url().await.unwrap_or_default();
        
        let memory = format!("Closed tab #{}, now on {}", tab_id, current_url);
        info!("❌ {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_scroll(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let down = action
            .params
            .get("down")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let pages = action
            .params
            .get("pages")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let index = action
            .params
            .get("index")
            .and_then(|v| v.as_u64())
            .map(|i| i as u32);

        let mut page = browser_session.get_page()?;
        let mouse = page.mouse().await;
        
        // Get viewport height for accurate scrolling
        let viewport_height = 1000.0; // Default fallback
        let scroll_amount = pages * viewport_height;
        let delta_y = if down { scroll_amount } else { -scroll_amount };
        
        // Scroll the page or element
        // For element scrolling, we'd need to get element position - simplified for now
        mouse.scroll(0.0, 0.0, None, Some(delta_y)).await?;
        
        let direction = if down { "down" } else { "up" };
        let memory = format!("Scrolled {} {} pages", direction, pages);
        info!("📜 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_go_back(
        &self,
        _action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let page = browser_session.get_page()?;
        page.go_back().await?;
        
        let memory = "Navigated back".to_string();
        info!("🔙 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_wait(&self, action: ActionModel) -> Result<ActionResult> {
        let seconds = action
            .params
            .get("seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as u64;
        
        // Cap at 30 seconds
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

    async fn handle_send_keys(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let keys = action
            .params
            .get("keys")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'keys' parameter".to_string()))?;
        
        let page = browser_session.get_page()?;
        
        // Parse keys string (e.g., "Tab Tab Enter" or "ArrowDown ArrowDown")
        let key_sequence: Vec<&str> = keys.split_whitespace().collect();
        
        for key in key_sequence {
            page.press(key).await?;
            // Small delay between keys
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        
        let memory = format!("Sent keys: {}", keys);
        info!("⌨️ {}", memory);
        Ok(ActionResult {
            extracted_content: Some(memory.clone()),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_evaluate(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let expression = action
            .params
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'expression' parameter".to_string()))?;
        
        let page = browser_session.get_page()?;
        let result = page.evaluate(expression).await?;
        
        let memory = format!("Evaluated JavaScript: {}", expression);
        info!("💻 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(result),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }
}

impl Default for Tools {
    fn default() -> Self {
        Self::new(vec![])
    }
}

