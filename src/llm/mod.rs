//! LLM integration modules

pub mod base;
pub mod watsonx;

pub use base::{ChatInvokeCompletion, ChatInvokeUsage, ChatMessage, ChatModel};
pub use watsonx::WatsonxChat;
