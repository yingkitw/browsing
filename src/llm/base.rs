//! Base traits for LLM chat models

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Chat message for LLM communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender (user, assistant, system)
    pub role: String,
    /// Content of the message
    pub content: String,
}

impl ChatMessage {
    /// Creates a new chat message
    pub fn new(role: String, content: String) -> Self {
        Self { role, content }
    }

    /// Creates a user message
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }

    /// Creates an assistant message
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }

    /// Creates a system message
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
    ) -> Result<Box<dyn futures_util::stream::Stream<Item = Result<String>> + Send + Unpin>>;
}

/// Usage information for a chat model invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInvokeUsage {
    /// Number of prompt tokens used
    pub prompt_tokens: u32,
    /// Number of cached prompt tokens
    pub prompt_cached_tokens: Option<u32>,
    /// Number of tokens used to create prompt cache
    pub prompt_cache_creation_tokens: Option<u32>,
    /// Number of tokens used for images
    pub prompt_image_tokens: Option<u32>,
    /// Number of completion tokens used
    pub completion_tokens: u32,
    /// Total number of tokens used
    pub total_tokens: u32,
}

/// Response from a chat model invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInvokeCompletion<T> {
    /// The completion content
    pub completion: T,
    /// Thinking process of the model
    pub thinking: Option<String>,
    /// Redacted thinking content
    pub redacted_thinking: Option<String>,
    /// Token usage information
    pub usage: Option<ChatInvokeUsage>,
    /// Reason for stopping
    pub stop_reason: Option<String>,
}

impl<T> ChatInvokeCompletion<T> {
    /// Creates a new completion
    pub fn new(completion: T) -> Self {
        Self {
            completion,
            thinking: None,
            redacted_thinking: None,
            usage: None,
            stop_reason: None,
        }
    }

    /// Sets the usage information
    pub fn with_usage(mut self, usage: ChatInvokeUsage) -> Self {
        self.usage = Some(usage);
        self
    }
}
