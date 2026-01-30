//! Tests for LLM integration

use browsing::llm::base::{ChatInvokeCompletion, ChatInvokeUsage, ChatMessage};

#[test]
fn test_chat_message_creation() {
    let msg = ChatMessage::user("Hello".to_string());
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Hello");
}

#[test]
fn test_chat_message_system() {
    let msg = ChatMessage::system("You are a helpful assistant".to_string());
    assert_eq!(msg.role, "system");
    assert_eq!(msg.content, "You are a helpful assistant");
}

#[test]
fn test_chat_message_assistant() {
    let msg = ChatMessage::assistant("Hi there!".to_string());
    assert_eq!(msg.role, "assistant");
    assert_eq!(msg.content, "Hi there!");
}

#[test]
fn test_chat_message_new() {
    let msg = ChatMessage::new("custom".to_string(), "Custom message".to_string());
    assert_eq!(msg.role, "custom");
    assert_eq!(msg.content, "Custom message");
}

#[test]
fn test_chat_invoke_completion() {
    let completion = ChatInvokeCompletion::new("Response text".to_string());
    assert_eq!(completion.completion, "Response text");
    assert!(completion.thinking.is_none());
    assert!(completion.usage.is_none());
}

#[test]
fn test_chat_invoke_completion_with_usage() {
    let usage = ChatInvokeUsage {
        prompt_tokens: 100,
        prompt_cached_tokens: None,
        prompt_cache_creation_tokens: None,
        prompt_image_tokens: None,
        completion_tokens: 50,
        total_tokens: 150,
    };

    let completion = ChatInvokeCompletion::new("Response".to_string()).with_usage(usage);

    assert!(completion.usage.is_some());
    assert_eq!(completion.usage.as_ref().unwrap().total_tokens, 150);
}

#[test]
fn test_chat_invoke_usage() {
    let usage = ChatInvokeUsage {
        prompt_tokens: 200,
        prompt_cached_tokens: Some(50),
        prompt_cache_creation_tokens: None,
        prompt_image_tokens: Some(10),
        completion_tokens: 100,
        total_tokens: 300,
    };

    assert_eq!(usage.prompt_tokens, 200);
    assert_eq!(usage.completion_tokens, 100);
    assert_eq!(usage.total_tokens, 300);
    assert_eq!(usage.prompt_cached_tokens, Some(50));
}
