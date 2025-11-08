//! Base traits for LLM chat models

use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Chat message for LLM communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: String, content: String) -> Self {
        Self { role, content }
    }

    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }

    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
        }
    }
}

/// Chat model trait for LLM integration
#[async_trait]
pub trait ChatModel: Send + Sync {
    /// Get the model name
    fn model(&self) -> &str;
    
    /// Get the provider name
    fn provider(&self) -> &str;
    
    /// Get the model name (alias for model)
    fn name(&self) -> &str {
        self.model()
    }

    /// Chat with the model (non-streaming)
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatInvokeCompletion<String>>;
    
    /// Chat with the model (streaming)
    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>>;
}

/// Usage information for a chat model invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInvokeUsage {
    pub prompt_tokens: u32,
    pub prompt_cached_tokens: Option<u32>,
    pub prompt_cache_creation_tokens: Option<u32>,
    pub prompt_image_tokens: Option<u32>,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Response from a chat model invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInvokeCompletion<T> {
    pub completion: T,
    pub thinking: Option<String>,
    pub redacted_thinking: Option<String>,
    pub usage: Option<ChatInvokeUsage>,
    pub stop_reason: Option<String>,
}

impl<T> ChatInvokeCompletion<T> {
    pub fn new(completion: T) -> Self {
        Self {
            completion,
            thinking: None,
            redacted_thinking: None,
            usage: None,
            stop_reason: None,
        }
    }

    pub fn with_usage(mut self, usage: ChatInvokeUsage) -> Self {
        self.usage = Some(usage);
        self
    }
}
