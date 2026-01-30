//! Tools service for action registry

use crate::agent::views::ActionResult;
use crate::error::{BrowsingError, Result};
use crate::llm::base::ChatMessage;
use crate::traits::BrowserClient;
use crate::tools::handlers::{AdvancedHandler, ContentHandler, InteractionHandler, NavigationHandler, TabsHandler, Handler};
use crate::tools::registry::Registry;
use crate::tools::views::{ActionContext, ActionModel, ActionParams};
use tracing::info;

/// Tools registry for agent actions
pub struct Tools {
    /// Action registry
    pub registry: Registry,
    /// Whether to display files in done text
    pub display_files_in_done_text: bool,
}

impl Tools {
    /// Creates a new tools registry
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

    /// Executes an action
    pub async fn act(
        &self,
        action: ActionModel,
        browser_session: &mut dyn BrowserClient,
        selector_map: Option<
            &std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>,
        >,
    ) -> Result<ActionResult> {
        self.act_with_llm(action, browser_session, selector_map, None)
            .await
    }

    /// Executes an action with LLM
    pub async fn act_with_llm(
        &self,
        action: ActionModel,
        browser_session: &mut dyn BrowserClient,
        selector_map: Option<
            &std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>,
        >,
        llm: Option<&dyn crate::llm::base::ChatModel>,
    ) -> Result<ActionResult> {
        let action_type = action.action_type.as_str();

        // Check if this is a custom action with a handler
        if let Some(handler) = self.registry.get_handler(action_type) {
            let params = ActionParams::new(&action.params).with_action_type(action.action_type.clone());
            let mut context = ActionContext {
                browser: browser_session,
                selector_map,
            };
            return handler.execute(&params, &mut context).await;
        }

        // Use new handler-based dispatch for built-in actions
        let params = ActionParams::new(&action.params).with_action_type(action.action_type.clone());
        let mut context = ActionContext {
            browser: browser_session,
            selector_map,
        };

        match action_type {
            // Navigation actions
            "search" | "navigate" | "go_back" => {
                NavigationHandler.handle(&params, &mut context).await
            }
            // Interaction actions
            "click" | "input" | "send_keys" => {
                InteractionHandler.handle(&params, &mut context).await
            }
            // Tab actions
            "switch" | "close" => {
                TabsHandler.handle(&params, &mut context).await
            }
            // Content actions
            "scroll" | "find_text" | "dropdown_options" | "select_dropdown" => {
                ContentHandler.handle(&params, &mut context).await
            }
            // Advanced actions
            "done" | "evaluate" | "upload_file" | "wait" => {
                AdvancedHandler.handle(&params, &mut context).await
            }
            // Extract action (requires LLM)
            "extract" => self.handle_extract(action, browser_session, llm).await,
            _ => Err(BrowsingError::Tool(format!(
                "Unknown action type: {action_type}"
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
        self.registry
            .register_custom_action(name, description, domains, handler);
    }

    /// Handle extract action (requires LLM)
    async fn handle_extract(
        &self,
        action: ActionModel,
        browser_session: &mut dyn BrowserClient,
        llm: Option<&dyn crate::llm::base::ChatModel>,
    ) -> Result<ActionResult> {
        let query = action
            .params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BrowsingError::Tool("Missing 'query' parameter".to_string()))?;

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

        // Get current URL
        let current_url = browser_session
            .get_current_url()
            .await
            .unwrap_or_else(|_| "unknown".to_string());

        // Try to get markdown content using evaluate
        let page = browser_session.get_page()?;
        let content_script = r#"
            (function() {
                // Get page text content
                const body = document.body || document.documentElement;
                return body.innerText || body.textContent || '';
            })()
        "#;

        let content = page
            .evaluate(content_script)
            .await
            .unwrap_or_else(|_| "".to_string());

        let content_str = content.as_str();

        // Apply start_from_char
        let final_content = if start_from_char > 0 && start_from_char < content_str.len() {
            &content_str[start_from_char..]
        } else {
            content_str
        };

        // Truncate if too long (max 100k chars)
        let max_chars = 100_000;
        let truncated = final_content.len() > max_chars;
        let final_content = if truncated {
            &final_content[..max_chars]
        } else {
            final_content
        };

        // If LLM is available, use it to extract structured data
        if let Some(llm) = llm {
            let system_prompt = "You are a data extraction assistant. Extract the requested information from the provided content and return it in a structured format. Be concise and accurate.";
            
            let user_prompt = format!(
                "Extract the following information from this content:\n\nQuery: {}\n\nContent:\n{}",
                query, final_content
            );

            let messages = vec![
                ChatMessage::system(system_prompt.to_string()),
                ChatMessage::user(user_prompt),
            ];

            match llm.chat(&messages).await {
                Ok(response) => {
                    let extracted_content = format!(
                        "<url>\n{}\n</url>\n<query>\n{}\n</query>\n<result>\n{}\n</result>",
                        current_url, query, response.completion
                    );

                    let memory = if extracted_content.len() < 1000 {
                        extracted_content.clone()
                    } else {
                        format!(
                            "Query: {}\nContent extracted ({} chars)",
                            query,
                            extracted_content.len()
                        )
                    };

                    info!("📄 Extracted content for query: {}", query);
                    Ok(ActionResult {
                        extracted_content: Some(extracted_content),
                        long_term_memory: Some(memory),
                        ..Default::default()
                    })
                }
                Err(e) => {
                    Err(BrowsingError::Tool(format!(
                        "LLM extraction failed: {e}"
                    )))
                }
            }
        } else {
            // No LLM available - return raw content with a note
            let extracted_content = format!(
                "<url>\n{}\n</url>\n<query>\n{}\n</query>\n<result>\nNo LLM available for extraction. Raw content:\n{}\n</result>",
                current_url,
                query,
                if truncated {
                    format!(
                        "{}... (truncated)",
                        &final_content[..1000.min(final_content.len())]
                    )
                } else {
                    final_content.to_string()
                }
            );

            info!("📄 Extracted raw content for query: {} (no LLM)", query);
            Ok(ActionResult {
                extracted_content: Some(extracted_content),
                long_term_memory: Some(format!(
                    "Extracted content for query: {query} (no LLM available)"
                )),
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
