//! Watsonx LLM integration

use crate::error::{BrowserUseError, Result};
use crate::llm::base::{ChatModel, ChatMessage, ChatInvokeCompletion};
use async_trait::async_trait;
use serde_json::json;
use futures::StreamExt;

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
            "messages": watsonx_messages,
            "model_id": self.model,
            "parameters": {
                "temperature": 0.0,
                "max_tokens": 4096
            }
        })
    }

    /// Collect stream into a single completion
    async fn collect_stream(
        &self,
        mut stream: Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>,
    ) -> Result<String> {
        let mut result = String::new();
        
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => result.push_str(&chunk),
                Err(e) => return Err(e),
            }
        }
        
        Ok(result)
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
        // Use streaming method and collect results
        let stream = self.chat_stream(messages).await?;
        let completion_text = self.collect_stream(stream).await?;
        
        Ok(ChatInvokeCompletion::new(completion_text))
    }

    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        // Use watsonx-rs generate_stream when available
        // The watsonx-rs crate should provide generate_stream function
        // Example usage (adjust based on actual API):
        // use watsonx_rs::generate_stream;
        // let stream = generate_stream(
        //     &self.api_key,
        //     &self.base_url,
        //     &self.model,
        //     &payload,
        // ).await?;
        // return Ok(stream);
        
        // Current implementation: HTTP streaming as fallback
        // This will be replaced with watsonx-rs::generate_stream when the crate API is finalized
        let payload = self.messages_to_watsonx(messages);
        
        // Build request URL - adjust based on actual Watsonx API endpoint
        let url = format!("{}/ml/v1/text/generation_stream", self.base_url);
        
        // Make streaming request
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&payload)
            .send()
            .await
            .map_err(|e| BrowserUseError::Llm(format!("HTTP error: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(BrowserUseError::Llm(format!(
                "Watsonx API error: {} - {}",
                status, error_text
            )));
        }
        
        // Convert response stream to string chunks
        // Parse SSE (Server-Sent Events) format
        let stream = response
            .bytes_stream()
            .map(|result| {
                result
                    .map_err(|e| BrowserUseError::Llm(format!("Stream error: {}", e)))
                    .and_then(|bytes| {
                        // Parse SSE format: "data: <content>\n\n"
                        let text = String::from_utf8(bytes.to_vec())
                            .map_err(|e| BrowserUseError::Llm(format!("UTF-8 error: {}", e)))?;
                        
                        // Extract data from SSE format
                        let mut content = String::new();
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..]; // Skip "data: "
                                if data != "[DONE]" {
                                    content.push_str(data);
                                }
                            }
                        }
                        
                        Ok(content)
                    })
            });
        
        Ok(Box::new(stream))
    }
}
