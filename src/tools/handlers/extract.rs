//! Extract action handler (LLM-based content extraction)

use crate::agent::views::ActionResult;
use crate::error::{BrowsingError, Result};
use crate::llm::base::ChatMessage;
use crate::tools::views::ActionModel;
use crate::traits::BrowserClient;
use tracing::info;

/// Execute extract action: get page content and optionally use LLM to extract structured data.
pub async fn handle_extract(
    action: ActionModel,
    browser_session: &mut dyn BrowserClient,
    llm: Option<&dyn crate::llm::base::ChatModel>,
) -> Result<ActionResult> {
    let query = action
        .params
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BrowsingError::Tool("Missing 'query' parameter".to_string()))?;

    let start_from_char = action
        .params
        .get("start_from_char")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let current_url = browser_session
        .get_current_url()
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let page = browser_session.get_page()?;
    let content_script = r#"
        (function() {
            const body = document.body || document.documentElement;
            return body.innerText || body.textContent || '';
        })()
    "#;

    let content = page.evaluate(content_script).await.unwrap_or_default();
    let content_str = content.as_str();

    let final_content = if start_from_char > 0 && start_from_char < content_str.len() {
        &content_str[start_from_char..]
    } else {
        content_str
    };

    let max_chars = 100_000;
    let truncated = final_content.len() > max_chars;
    let final_content = if truncated {
        &final_content[..max_chars]
    } else {
        final_content
    };

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
            Err(e) => Err(BrowsingError::Tool(format!("LLM extraction failed: {e}"))),
        }
    } else {
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
