//! Watsonx LLM integration

use crate::error::{BrowserUseError, Result};
use crate::llm::base::{ChatModel, ChatMessage, ChatInvokeCompletion};
use async_trait::async_trait;
use serde_json::json;

/// Watsonx chat model implementation
pub struct WatsonxChat {
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

impl WatsonxChat {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "ibm/granite-4-h-small".to_string()),
            client: reqwest::Client::new(),
            base_url: "https://us-south.ml.cloud.ibm.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Convert messages to watsonx format
    fn messages_to_watsonx(&self, messages: &[ChatMessage]) -> serde_json::Value {
        let watsonx_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role,
                    "content": msg.content
                })
            })
            .collect();
        
        json!({
            "messages": watsonx_messages
        })
    }
}

#[async_trait]
impl ChatModel for WatsonxChat {
    fn model(&self) -> &str {
        &self.model
    }

    fn provider(&self) -> &str {
        "watsonx"
    }

    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatInvokeCompletion<String>> {
        // Use watsonx-rs for actual API calls
        // For now, implement a basic HTTP call structure
        let payload = self.messages_to_watsonx(messages);
        
        // TODO: Use watsonx-rs crate's generate_stream function
        // For now, return an error indicating it needs watsonx-rs integration
        Err(BrowserUseError::Llm(
            "Watsonx integration requires watsonx-rs crate implementation".to_string(),
        ))
    }

    async fn chat_stream(
        &self,
        _messages: &[ChatMessage],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        // TODO: Implement streaming using watsonx-rs
        // This should use watsonx-rs's generate_stream function
        // For now, return an empty stream
        use futures::stream;
        Ok(Box::new(stream::empty()))
    }
}
