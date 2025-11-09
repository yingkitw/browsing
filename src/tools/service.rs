//! Tools service for action registry

use crate::error::{BrowserUseError, Result};
use crate::tools::registry::Registry;
use crate::tools::views::ActionModel;
use crate::agent::views::ActionResult;
use crate::browser::Browser;
use tracing::info;
use serde_json::json;

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
        
        registry.register_action(
            "find_text".to_string(),
            "Scroll to specific text on page".to_string(),
            None,
        );
        
        registry.register_action(
            "dropdown_options".to_string(),
            "Get dropdown option values".to_string(),
            None,
        );
        
        registry.register_action(
            "select_dropdown".to_string(),
            "Select dropdown options".to_string(),
            None,
        );
        
        registry.register_action(
            "upload_file".to_string(),
            "Upload files to file inputs".to_string(),
            None,
        );
        
        registry.register_action(
            "extract".to_string(),
            "LLM extracts structured data from page markdown. Use when: on right page, know what to extract, haven't called before on same page+query.".to_string(),
            None,
        );
    }

    pub async fn act(
        &self,
        action: ActionModel,
        browser_session: &mut crate::browser::Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        self.act_with_llm(action, browser_session, selector_map, None).await
    }
    
    pub async fn act_with_llm(
        &self,
        action: ActionModel,
        browser_session: &mut crate::browser::Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
        llm: Option<&dyn crate::llm::base::ChatModel>,
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
            "find_text" => self.handle_find_text(action, browser_session).await,
            "dropdown_options" => self.handle_dropdown_options(action, browser_session, selector_map).await,
            "select_dropdown" => self.handle_select_dropdown(action, browser_session, selector_map).await,
            "upload_file" => self.handle_upload_file(action, browser_session, selector_map).await,
            "extract" => self.handle_extract(action, browser_session, llm).await,
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
        let _index = action
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

    async fn handle_find_text(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
    ) -> Result<ActionResult> {
        let text = action
            .params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'text' parameter".to_string()))?;
        
        let page = browser_session.get_page()?;
        
        // Use JavaScript to find and scroll to text
        let script = format!(
            r#"
            (function() {{
                const searchText = {};
                const walker = document.createTreeWalker(
                    document.body,
                    NodeFilter.SHOW_TEXT,
                    null,
                    false
                );
                
                let node;
                while (node = walker.nextNode()) {{
                    if (node.textContent && node.textContent.includes(searchText)) {{
                        const range = document.createRange();
                        range.selectNodeContents(node);
                        const rect = range.getBoundingClientRect();
                        window.scrollTo({{
                            top: window.scrollY + rect.top - window.innerHeight / 2,
                            behavior: 'smooth'
                        }});
                        return true;
                    }}
                }}
                return false;
            }})()
            "#,
            serde_json::to_string(text)?
        );
        
        let result = page.evaluate(&script).await?;
        let found = result.trim() == "true";
        
        if found {
            let memory = format!("Scrolled to text: {}", text);
            info!("🔍 {}", memory);
            Ok(ActionResult {
                extracted_content: Some(memory.clone()),
                long_term_memory: Some(memory),
                ..Default::default()
            })
        } else {
            let msg = format!("Text '{}' not found or not visible on page", text);
            info!("⚠️ {}", msg);
            Ok(ActionResult {
                extracted_content: Some(msg.clone()),
                long_term_memory: Some(format!("Tried scrolling to text '{}' but it was not found", text)),
                ..Default::default()
            })
        }
    }

    async fn handle_dropdown_options(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        let index = action
            .params
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'index' parameter".to_string()))? as u32;
        
        // Get element from selector map
        let element = selector_map
            .and_then(|map| map.get(&index))
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} not found", index)))?;
        
        let page = browser_session.get_page()?;
        
        // Get options using JavaScript
        let backend_node_id = element.backend_node_id
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} has no backend_node_id", index)))?;
        
        let script = format!(
            r#"
            (function() {{
                const nodeId = {};
                const node = document.querySelector(`[data-backend-node-id="${{nodeId}}"]`) ||
                             Array.from(document.querySelectorAll('select')).find(el => {{
                                 const rect = el.getBoundingClientRect();
                                 return rect.width > 0 && rect.height > 0;
                             }}) || document.querySelector('select');
                
                if (!node && document.querySelector('select')) {{
                    const select = document.querySelector('select');
                    const options = Array.from(select.options).map(opt => ({{
                        value: opt.value,
                        text: opt.text,
                        selected: opt.selected
                    }}));
                    return JSON.stringify(options);
                }}
                
                if (node && node.tagName === 'SELECT') {{
                    const options = Array.from(node.options).map(opt => ({{
                        value: opt.value,
                        text: opt.text,
                        selected: opt.selected
                    }}));
                    return JSON.stringify(options);
                }}
                
                return JSON.stringify([]);
            }})()
            "#,
            backend_node_id
        );
        
        let result = page.evaluate(&script).await?;
        let options: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap_or_default();
        
        let options_text = options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let value = opt.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let text = opt.get("text").and_then(|v| v.as_str()).unwrap_or("");
                format!("{}. {} (value: {})", i + 1, text, value)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let memory = format!("Dropdown options for index {}:\n{}", index, options_text);
        info!("📋 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(options_text),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_select_dropdown(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        let index = action
            .params
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'index' parameter".to_string()))? as u32;
        
        let text = action
            .params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'text' parameter".to_string()))?;
        
        // Get element from selector map
        let element = selector_map
            .and_then(|map| map.get(&index))
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} not found", index)))?;
        
        let page = browser_session.get_page()?;
        
        // Select option using JavaScript
        let backend_node_id = element.backend_node_id
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} has no backend_node_id", index)))?;
        
        let script = format!(
            r#"
            (function() {{
                const nodeId = {};
                const searchText = {};
                const node = document.querySelector(`[data-backend-node-id="${{nodeId}}"]`) ||
                             Array.from(document.querySelectorAll('select')).find(el => {{
                                 const rect = el.getBoundingClientRect();
                                 return rect.width > 0 && rect.height > 0;
                             }}) || document.querySelector('select');
                
                if (!node || node.tagName !== 'SELECT') {{
                    return {{ success: false, error: 'Element is not a select dropdown' }};
                }}
                
                const options = Array.from(node.options);
                const option = options.find(opt => 
                    opt.text.trim() === searchText || 
                    opt.value === searchText ||
                    opt.text.includes(searchText)
                );
                
                if (!option) {{
                    return {{ success: false, error: `Option "${{searchText}}" not found` }};
                }}
                
                node.value = option.value;
                node.dispatchEvent(new Event('change', {{ bubbles: true }}));
                node.dispatchEvent(new Event('input', {{ bubbles: true }}));
                
                return {{ success: true, message: `Selected option: ${{option.text}} (value: ${{option.value}})` }};
            }})()
            "#,
            backend_node_id,
            serde_json::to_string(text)?
        );
        
        let result = page.evaluate(&script).await?;
        let result_obj: serde_json::Value = serde_json::from_str(&result).unwrap_or(serde_json::json!({}));
        
        if result_obj.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            let message = result_obj
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Selected option");
            let memory = format!("Selected dropdown option '{}' at index {}", text, index);
            info!("✅ {}", memory);
            Ok(ActionResult {
                extracted_content: Some(message.to_string()),
                long_term_memory: Some(memory),
                ..Default::default()
            })
        } else {
            let error = result_obj
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Failed to select dropdown option");
            Err(BrowserUseError::Tool(error.to_string()))
        }
    }

    async fn handle_upload_file(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
        selector_map: Option<&std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> Result<ActionResult> {
        let index = action
            .params
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'index' parameter".to_string()))? as u32;
        
        let path = action
            .params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'path' parameter".to_string()))?;
        
        // Check if file exists
        if !std::path::Path::new(path).exists() {
            return Err(BrowserUseError::Tool(format!("File {} does not exist", path)));
        }
        
        // Get element from selector map
        let element = selector_map
            .and_then(|map| map.get(&index))
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} not found", index)))?;
        
        // Get CDP client for file upload
        let client = browser_session.get_cdp_client()?;
        
        // For file upload, we need to use DOM.setFileInputFiles
        // First, get the node ID
        let backend_node_id = element.backend_node_id
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} has no backend_node_id", index)))?;
        
        let node_id = {
            let params = json!({
                "backendNodeIds": [backend_node_id]
            });
            let result = client
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
            node_id as u32
        };
        
        // Use DOM.setFileInputFiles to upload the file
        // Note: This requires the file to be accessible via the browser's file system
        // For local browsers, we can use the file path directly
        let params = json!({
            "nodeId": node_id,
            "files": [path]
        });
        
        // Get current session ID
        let session_id = browser_session.get_session_id()?;
        
        client
            .send_command_with_session("DOM.setFileInputFiles", params, Some(&session_id))
            .await
            .map_err(|e| BrowserUseError::Tool(format!("Failed to upload file: {}", e)))?;
        
        let memory = format!("Uploaded file {} to element {}", path, index);
        info!("📁 {}", memory);
        Ok(ActionResult {
            extracted_content: Some(format!("Successfully uploaded file to index {}", index)),
            long_term_memory: Some(memory),
            ..Default::default()
        })
    }

    async fn handle_extract(
        &self,
        action: ActionModel,
        browser_session: &mut Browser,
        llm: Option<&dyn crate::llm::base::ChatModel>,
    ) -> Result<ActionResult> {
        let query = action
            .params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowserUseError::Tool("Missing 'query' parameter".to_string()))?;
        
        let _extract_links = action
            .params
            .get("extract_links")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let start_from_char = action
            .params
            .get("start_from_char")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        
        // Get DOM service - we need to access it through browser session
        // For now, we'll get the markdown from the serialized DOM state
        // This is a simplified version - in production, you'd want to pass dom_service
        
        // Get current URL
        let current_url = browser_session.get_current_url().await.unwrap_or_else(|_| "unknown".to_string());
        
        // Try to get markdown content using evaluate
        let page = browser_session.get_page()?;
        let content_script = r#"
            (function() {
                // Get page text content
                const body = document.body || document.documentElement;
                return body.innerText || body.textContent || '';
            })()
        "#;
        
        let content = page.evaluate(content_script).await.unwrap_or_else(|_| "Unable to extract content".to_string());
        
        // Apply start_from_char if specified
        let content = if start_from_char > 0 && start_from_char < content.len() {
            &content[start_from_char..]
        } else {
            &content
        };
        
        // Truncate if too long (MAX_CHAR_LIMIT = 30000)
        const MAX_CHAR_LIMIT: usize = 30000;
        let truncated = content.len() > MAX_CHAR_LIMIT;
        let final_content = if truncated {
            // Try to truncate at paragraph break
            if let Some(break_pos) = content[..MAX_CHAR_LIMIT].rfind("\n\n") {
                &content[..break_pos]
            } else if let Some(break_pos) = content[..MAX_CHAR_LIMIT].rfind('.') {
                &content[..=break_pos]
            } else {
                &content[..MAX_CHAR_LIMIT]
            }
        } else {
            content
        };
        
        // If LLM is available, use it to extract data
        if let Some(llm) = llm {
            let system_prompt = r#"You are an expert at extracting data from the markdown of a webpage.

<input>
You will be given a query and the text content of a webpage.
</input>

<instructions>
- You are tasked to extract information from the webpage that is relevant to the query.
- You should ONLY use the information available in the webpage to answer the query. Do not make up information or provide guess from your own knowledge.
- If the information relevant to the query is not available in the page, your response should mention that.
- If the query asks for all items, products, etc., make sure to directly list all of them.
</instructions>

<output>
- Your output should present ALL the information relevant to the query in a concise way.
- Do not answer in conversational format - directly output the relevant information or that the information is unavailable.
</output>"#;
            
            let prompt = format!(
                "<query>\n{}\n</query>\n\n<webpage_content>\n{}\n</webpage_content>",
                query,
                final_content
            );
            
            let messages = vec![
                crate::llm::base::ChatMessage::system(system_prompt.to_string()),
                crate::llm::base::ChatMessage::user(prompt),
            ];
            
            match llm.chat(&messages).await {
                Ok(response) => {
                    let extracted_content = format!(
                        "<url>\n{}\n</url>\n<query>\n{}\n</query>\n<result>\n{}\n</result>",
                        current_url,
                        query,
                        response.completion
                    );
                    
                    let memory = if extracted_content.len() < 1000 {
                        extracted_content.clone()
                    } else {
                        format!("Query: {}\nContent extracted ({} chars)", query, extracted_content.len())
                    };
                    
                    info!("📄 Extracted content for query: {}", query);
                    Ok(ActionResult {
                        extracted_content: Some(extracted_content),
                        long_term_memory: Some(memory),
                        ..Default::default()
                    })
                }
                Err(e) => {
                    // Fallback: return raw content
                    let _extracted_content = format!(
                        "<url>\n{}\n</url>\n<query>\n{}\n</query>\n<result>\n{}\n</result>",
                        current_url,
                        query,
                        "LLM extraction failed, returning raw content"
                    );
                    Err(BrowserUseError::Tool(format!("LLM extraction failed: {}", e)))
                }
            }
        } else {
            // No LLM available - return raw content with a note
            let extracted_content = format!(
                "<url>\n{}\n</url>\n<query>\n{}\n</query>\n<result>\nNo LLM available for extraction. Raw content:\n{}\n</result>",
                current_url,
                query,
                if truncated { format!("{}... (truncated)", &final_content[..1000.min(final_content.len())]) } else { final_content.to_string() }
            );
            
            info!("📄 Extracted raw content for query: {} (no LLM)", query);
            Ok(ActionResult {
                extracted_content: Some(extracted_content),
                long_term_memory: Some(format!("Extracted content for query: {} (no LLM available)", query)),
                ..Default::default()
            })
        }
    }
}

impl Default for Tools {
    fn default() -> Self {
        Self::new(vec![])
    }
}

