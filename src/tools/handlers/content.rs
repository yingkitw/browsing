//! Content action handlers

use super::Handler;
use crate::agent::views::ActionResult;
use crate::error::{BrowserUseError, Result};
use crate::tools::views::{ActionContext, ActionParams};
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

pub struct ContentHandler;

#[async_trait]
impl Handler for ContentHandler {
    async fn handle(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        match params.get_action_type().unwrap_or("unknown") {
            "scroll" => self.scroll(params, context).await,
            "find_text" => self.find_text(params, context).await,
            "dropdown_options" => self.dropdown_options(params, context).await,
            "select_dropdown" => self.select_dropdown(params, context).await,
            _ => Err(BrowserUseError::Tool("Unknown content action".into())),
        }
    }
}

impl ContentHandler {
    async fn scroll(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let down = params.get_optional_bool("down");
        let pages = params.get_optional_f64("pages").unwrap_or(1.0);

        let mut page = context.browser.get_page()?;
        let mouse = page.mouse().await;
        let viewport_height = 1000.0;
        let delta_y = if down { pages * viewport_height } else { -pages * viewport_height };

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

    async fn find_text(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let text = params.get_required_str("text")?;
        let page = context.browser.get_page()?;

        let script = format!(
            r#"(function() {{
                const searchText = {};
                const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null, false);
                let node;
                while (node = walker.nextNode()) {{
                    if (node.textContent && node.textContent.includes(searchText)) {{
                        const range = document.createRange();
                        range.selectNodeContents(node);
                        const rect = range.getBoundingClientRect();
                        window.scrollTo({{ top: window.scrollY + rect.top - window.innerHeight / 2, behavior: 'smooth' }});
                        return true;
                    }}
                }}
                return false;
            }})()"#,
            json!(text)
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

    async fn dropdown_options(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let index = params.get_required_u32("index")?;
        let element = context.selector_map.and_then(|map| map.get(&index))
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} not found", index)))?;

        let page = context.browser.get_page()?;
        let backend_node_id = element.backend_node_id.ok_or_else(|| {
            BrowserUseError::Tool(format!("Element index {} has no backend_node_id", index))
        })?;

        let script = format!(
            r#"(function() {{
                const nodeId = {};
                const node = document.querySelector(`[data-backend-node-id="${{nodeId}}"]`) ||
                             Array.from(document.querySelectorAll('select')).find(el => {{
                                 const rect = el.getBoundingClientRect();
                                 return rect.width > 0 && rect.height > 0;
                             }}) || document.querySelector('select');
                if (!node && document.querySelector('select')) {{
                    const select = document.querySelector('select');
                    const options = Array.from(select.options).map(opt => ({{ value: opt.value, text: opt.text, selected: opt.selected }}));
                    return JSON.stringify(options);
                }}
                if (node && node.tagName === 'SELECT') {{
                    const options = Array.from(node.options).map(opt => ({{ value: opt.value, text: opt.text, selected: opt.selected }}));
                    return JSON.stringify(options);
                }}
                return JSON.stringify([]);
            }})()"#,
            backend_node_id
        );

        let result = page.evaluate(&script).await?;
        let options: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap_or_default();

        let options_text = options.iter().enumerate()
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

    async fn select_dropdown(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult> {
        let index = params.get_required_u32("index")?;
        let text = params.get_required_str("text")?;

        let element = context.selector_map.and_then(|map| map.get(&index))
            .ok_or_else(|| BrowserUseError::Tool(format!("Element index {} not found", index)))?;

        let page = context.browser.get_page()?;
        let backend_node_id = element.backend_node_id.ok_or_else(|| {
            BrowserUseError::Tool(format!("Element index {} has no backend_node_id", index))
        })?;

        let script = format!(
            r#"(function() {{
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
                const option = options.find(opt => opt.text.trim() === searchText || opt.value === searchText || opt.text.includes(searchText));
                if (!option) {{
                    return {{ success: false, error: `Option "${{searchText}}" not found` }};
                }}
                node.value = option.value;
                node.dispatchEvent(new Event('change', {{ bubbles: true }}));
                node.dispatchEvent(new Event('input', {{ bubbles: true }}));
                return {{ success: true, message: `Selected option: ${{option.text}} (value: ${{option.value}})` }};
            }})()"#,
            backend_node_id,
            json!(text)
        );

        let result = page.evaluate(&script).await?;
        let result_obj: serde_json::Value = serde_json::from_str(&result).unwrap_or(serde_json::json!({}));

        if result_obj.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            let message = result_obj.get("message").and_then(|v| v.as_str()).unwrap_or("Selected option");
            let memory = format!("Selected dropdown option '{}' at index {}", text, index);
            info!("✅ {}", memory);
            Ok(ActionResult {
                extracted_content: Some(message.to_string()),
                long_term_memory: Some(memory),
                ..Default::default()
            })
        } else {
            let error = result_obj.get("error").and_then(|v| v.as_str()).unwrap_or("Failed to select dropdown option");
            Err(BrowserUseError::Tool(error.to_string()))
        }
    }
}
