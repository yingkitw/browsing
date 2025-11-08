//! LLM integration modules

pub mod watsonx;
pub mod base;

pub use base::{ChatModel, ChatMessage, ChatInvokeCompletion, ChatInvokeUsage};
pub use watsonx::WatsonxChat;

