//! Tests for agent service functionality

use browsing::agent::views::{ActionResult, AgentHistoryList, AgentState};
use browsing::tokens::views::UsageSummary;

#[test]
fn test_action_result_creation() {
    let result = ActionResult {
        is_done: Some(false),
        success: Some(true),
        judgement: None,
        error: None,
        attachments: None,
        images: None,
        long_term_memory: Some("Memory".to_string()),
        extracted_content: Some("Test content".to_string()),
        include_extracted_content_only_once: false,
        metadata: None,
    };

    assert_eq!(result.extracted_content, Some("Test content".to_string()));
    assert_eq!(result.is_done, Some(false));
    assert!(!result.include_extracted_content_only_once);
}

#[test]
fn test_action_result_default() {
    let result = ActionResult::default();
    assert!(result.extracted_content.is_none());
    assert_eq!(result.is_done, Some(false)); // Default is Some(false)
    assert!(!result.include_extracted_content_only_once);
}

#[test]
fn test_usage_summary() {
    let usage = UsageSummary {
        prompt_tokens: Some(100),
        completion_tokens: Some(50),
        total_tokens: Some(150),
        cost: None,
    };

    assert_eq!(usage.prompt_tokens, Some(100));
    assert_eq!(usage.total_tokens, Some(150));
}

#[test]
fn test_agent_history_list_creation() {
    let history = AgentHistoryList {
        history: vec![],
        usage: None,
    };

    assert!(history.history.is_empty());
    assert!(history.usage.is_none());
}

#[test]
fn test_agent_state_default() {
    let state = AgentState::default();

    assert!(!state.agent_id.is_empty());
    assert_eq!(state.n_steps, 1);
    assert_eq!(state.consecutive_failures, 0);
    assert!(!state.paused);
    assert!(!state.stopped);
}

#[test]
fn test_action_result_serialization() {
    let result = ActionResult {
        is_done: Some(true),
        success: Some(true),
        judgement: None,
        error: None,
        attachments: None,
        images: None,
        long_term_memory: Some("Memory".to_string()),
        extracted_content: Some("Test".to_string()),
        include_extracted_content_only_once: false,
        metadata: None,
    };

    let json_str = serde_json::to_string(&result).unwrap();
    let deserialized: ActionResult = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.extracted_content, result.extracted_content);
    assert_eq!(deserialized.is_done, result.is_done);
}
