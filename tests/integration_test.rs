//! Comprehensive integration tests for browsing

use async_trait::async_trait;
use browsing::agent::views::ActionResult;
use browsing::browser::BrowserProfile;
use browsing::error::Result as BrowserUseResult;
use browsing::llm::base::{ChatInvokeCompletion, ChatInvokeUsage, ChatMessage, ChatModel};
use browsing::tools::service::Tools;
use browsing::tools::views::{ActionHandler, ActionContext, ActionModel, ActionParams};
use browsing::utils::extract_urls;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_tools_creation() {
    let tools = Tools::new(vec![]);
    // Tools should be created successfully
    assert!(!tools.registry.registry.actions.is_empty());
}

#[tokio::test]
async fn test_action_model_creation() {
    let params_json = json!({
        "query": "test"
    });
    let params: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_value(params_json).unwrap();

    let action = ActionModel {
        action_type: "search".to_string(),
        params,
    };

    assert_eq!(action.action_type, "search");
    assert!(action.params.get("query").is_some());
}

#[test]
fn test_url_extraction() {
    let text = "Visit https://example.com and http://test.org";
    let urls = extract_urls(text);
    assert!(urls.len() >= 2);
    assert!(urls.iter().any(|u| u.contains("example.com")));
    assert!(urls.iter().any(|u| u.contains("test.org")));
}

#[test]
fn test_url_extraction_complex() {
    let text = r#"
        Check out https://github.com/user/repo/issues/123
        Also visit http://example.com/path?query=value#fragment
        And www.example.com
    "#;
    let urls = extract_urls(text);
    assert!(!urls.is_empty());
}

#[test]
fn test_action_model_serialization() {
    let params_json = json!({
        "url": "https://example.com",
        "new_tab": false
    });
    let params: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_value(params_json).unwrap();

    let action = ActionModel {
        action_type: "navigate".to_string(),
        params,
    };

    // Test that we can serialize and deserialize
    let json_str = serde_json::to_string(&action).unwrap();
    let deserialized: ActionModel = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.action_type, "navigate");
    assert_eq!(
        deserialized.params.get("url").and_then(|v| v.as_str()),
        Some("https://example.com")
    );
}

#[test]
fn test_action_model_all_actions() {
    let actions = vec![
        ("search", json!({"query": "test"})),
        ("navigate", json!({"url": "https://example.com"})),
        ("click", json!({"index": 1})),
        ("input", json!({"index": 1, "text": "test"})),
        ("scroll", json!({"down": true, "pages": 1.0})),
        ("wait", json!({"seconds": 5})),
        ("send_keys", json!({"keys": "Enter"})),
        ("evaluate", json!({"expression": "1+1"})),
        ("find_text", json!({"text": "search"})),
        ("dropdown_options", json!({"index": 1})),
        ("select_dropdown", json!({"index": 1, "text": "option"})),
        ("upload_file", json!({"index": 1, "path": "/tmp/test.txt"})),
        ("extract", json!({"query": "extract data"})),
        ("done", json!({"text": "completed", "success": true})),
    ];

    for (action_type, params_json) in actions {
        let params: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(params_json).unwrap();

        let action = ActionModel {
            action_type: action_type.to_string(),
            params,
        };
        assert_eq!(action.action_type, action_type);
    }
}

// ============================================================================
// Mock LLM for Testing
// ============================================================================

/// Mock LLM implementation for testing
pub struct MockLLM {
    responses: Arc<Mutex<Vec<String>>>,
    current_index: Arc<Mutex<usize>>,
    model_name: String,
    provider_name: String,
}

impl MockLLM {
    /// Creates a new MockLLM with predefined responses
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            current_index: Arc::new(Mutex::new(0)),
            model_name: "mock-llm".to_string(),
            provider_name: "mock".to_string(),
        }
    }

    /// Creates a MockLLM that returns a "done" action
    pub fn with_done_action() -> Self {
        Self::new(vec![
            json!({
                "thinking": "Task completed",
                "evaluation_previous_goal": "Successfully completed the task",
                "memory": "Task completed",
                "next_goal": "Task completed",
                "action": [{
                    "done": {
                        "text": "Task completed successfully",
                        "success": true
                    }
                }]
            })
            .to_string(),
        ])
    }

    /// Creates a MockLLM that returns a navigate action
    pub fn with_navigate_action(url: &str) -> Self {
        Self::new(vec![
            json!({
                "thinking": "Navigating to page",
                "action": [{
                    "navigate": {
                        "url": url,
                        "new_tab": false
                    }
                }]
            })
            .to_string(),
        ])
    }
}

#[async_trait]
impl ChatModel for MockLLM {
    fn model(&self) -> &str {
        &self.model_name
    }

    fn provider(&self) -> &str {
        &self.provider_name
    }

    async fn chat(
        &self,
        _messages: &[ChatMessage],
    ) -> BrowserUseResult<ChatInvokeCompletion<String>> {
        let mut index = self.current_index.lock().unwrap();
        let responses = self.responses.lock().unwrap();

        let response = if *index < responses.len() {
            responses[*index].clone()
        } else {
            // Default done action if out of responses
            json!({
                "thinking": "No more actions",
                "action": [{
                    "done": {
                        "text": "No more actions available",
                        "success": true
                    }
                }]
            })
            .to_string()
        };

        *index += 1;

        Ok(ChatInvokeCompletion {
            completion: response,
            thinking: None,
            redacted_thinking: None,
            usage: Some(ChatInvokeUsage {
                prompt_tokens: 100,
                prompt_cached_tokens: None,
                prompt_cache_creation_tokens: None,
                prompt_image_tokens: None,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            stop_reason: None,
        })
    }

    async fn chat_stream(
        &self,
        _messages: &[ChatMessage],
    ) -> BrowserUseResult<Box<dyn futures_util::stream::Stream<Item = BrowserUseResult<String>> + Send + Unpin>>
    {
        let response = self.chat(_messages).await?;
        let completion = response.completion;
        let stream = futures_util::stream::iter(vec![Ok(completion)]);
        Ok(Box::new(stream))
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_tools_action_registration() {
    let tools = Tools::new(vec![]);

    // Verify all default actions are registered
    let default_actions = vec![
        "search",
        "navigate",
        "click",
        "input",
        "done",
        "switch",
        "close",
        "scroll",
        "wait",
        "send_keys",
        "evaluate",
        "find_text",
        "dropdown_options",
        "select_dropdown",
        "upload_file",
        "extract",
    ];

    for action_name in default_actions {
        assert!(
            tools.registry.registry.actions.contains_key(action_name),
            "Action '{action_name}' should be registered"
        );
    }
}

#[tokio::test]
async fn test_tools_custom_action_registration() {
    struct TestActionHandler;

    #[async_trait::async_trait]
    impl ActionHandler for TestActionHandler {
        async fn execute(
            &self,
            _params: &ActionParams,
            _context: &mut ActionContext<'_>,
        ) -> BrowserUseResult<ActionResult> {
            Ok(ActionResult {
                extracted_content: Some("Custom action executed".to_string()),
                ..Default::default()
            })
        }
    }

    let mut tools = Tools::new(vec![]);
    tools.register_custom_action(
        "custom_test".to_string(),
        "Test custom action".to_string(),
        None,
        TestActionHandler,
    );

    assert!(tools.registry.registry.actions.contains_key("custom_test"));
    assert!(tools.registry.has_custom_handler("custom_test"));
}

#[tokio::test]
async fn test_tools_action_validation() {
    // Test valid action
    let _tools = Tools::new(vec![]);

    let params: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "text": "Test",
        "success": true
    }))
    .unwrap();
    let valid_action = ActionModel {
        action_type: "done".to_string(),
        params,
    };

    // Test invalid action type
    let invalid_action = ActionModel {
        action_type: "nonexistent_action".to_string(),
        params: HashMap::new(),
    };

    // Actions should be parseable even if invalid
    assert_eq!(valid_action.action_type, "done");
    assert_eq!(invalid_action.action_type, "nonexistent_action");
}

#[tokio::test]
async fn test_mock_llm_basic() {
    let llm = MockLLM::with_done_action();

    let messages = vec![
        ChatMessage::system("You are a helpful assistant".to_string()),
        ChatMessage::user("Complete the task".to_string()),
    ];

    let response = llm.chat(&messages).await.unwrap();
    assert!(!response.completion.is_empty());
    assert!(response.usage.is_some());

    let usage = response.usage.unwrap();
    assert_eq!(usage.total_tokens, 150);
}

#[tokio::test]
async fn test_mock_llm_multiple_responses() {
    let responses = vec![
        json!({"action": [{"navigate": {"url": "https://example.com"}}]}).to_string(),
        json!({"action": [{"done": {"text": "Done", "success": true}}]}).to_string(),
    ];

    let llm = MockLLM::new(responses);

    let messages = vec![ChatMessage::user("Test".to_string())];

    // First call
    let response1 = llm.chat(&messages).await.unwrap();
    assert!(response1.completion.contains("navigate"));

    // Second call
    let response2 = llm.chat(&messages).await.unwrap();
    assert!(response2.completion.contains("done"));
}

#[test]
fn test_browser_profile_creation() {
    let profile = BrowserProfile::default();
    assert_eq!(profile.headless, None);
}

#[test]
fn test_action_result_completion_detection() {
    let done_result = ActionResult {
        is_done: Some(true),
        success: Some(true),
        ..Default::default()
    };

    let not_done_result = ActionResult {
        is_done: Some(false),
        ..Default::default()
    };

    assert_eq!(done_result.is_done, Some(true));
    assert_eq!(not_done_result.is_done, Some(false));
}

#[test]
fn test_action_result_error_handling() {
    let error_result = ActionResult {
        error: Some("Test error".to_string()),
        success: Some(false),
        ..Default::default()
    };

    assert!(error_result.error.is_some());
    assert_eq!(error_result.success, Some(false));
}

#[tokio::test]
async fn test_tools_exclude_actions() {
    let tools = Tools::new(vec!["search".to_string(), "navigate".to_string()]);

    // Excluded actions should not be registered
    assert!(!tools.registry.registry.actions.contains_key("search"));
    assert!(!tools.registry.registry.actions.contains_key("navigate"));

    // Other actions should still be registered
    assert!(tools.registry.registry.actions.contains_key("done"));
    assert!(tools.registry.registry.actions.contains_key("click"));
}

#[test]
fn test_url_extraction_edge_cases() {
    // Test empty string
    let urls = extract_urls("");
    assert!(urls.is_empty());

    // Test text without URLs
    let urls = extract_urls("This is just plain text");
    assert!(urls.is_empty());

    // Test multiple URLs
    let text = "Visit https://example.com and https://test.org and http://another.com";
    let urls = extract_urls(text);
    assert!(urls.len() >= 3);
}

#[test]
fn test_action_model_parameter_types() {
    // Test string parameter
    let params: HashMap<String, serde_json::Value> =
        serde_json::from_value(json!({"query": "test"})).unwrap();
    let action = ActionModel {
        action_type: "search".to_string(),
        params,
    };
    assert_eq!(
        action.params.get("query").and_then(|v| v.as_str()),
        Some("test")
    );

    // Test number parameter
    let params: HashMap<String, serde_json::Value> =
        serde_json::from_value(json!({"index": 5})).unwrap();
    let action = ActionModel {
        action_type: "click".to_string(),
        params,
    };
    assert_eq!(action.params.get("index").and_then(|v| v.as_u64()), Some(5));

    // Test boolean parameter
    let params: HashMap<String, serde_json::Value> =
        serde_json::from_value(json!({"down": true})).unwrap();
    let action = ActionModel {
        action_type: "scroll".to_string(),
        params,
    };
    assert_eq!(
        action.params.get("down").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn test_action_model_nested_parameters() {
    let params: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "url": "https://example.com",
        "new_tab": false,
        "wait_for": "load"
    }))
    .unwrap();
    let action = ActionModel {
        action_type: "navigate".to_string(),
        params,
    };

    assert_eq!(
        action.params.get("url").and_then(|v| v.as_str()),
        Some("https://example.com")
    );
    assert_eq!(
        action.params.get("new_tab").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        action.params.get("wait_for").and_then(|v| v.as_str()),
        Some("load")
    );
}

#[tokio::test]
async fn test_tools_registry_action_count() {
    let tools = Tools::new(vec![]);
    let action_count = tools.registry.registry.actions.len();

    // Should have at least the core actions registered
    assert!(
        action_count >= 10,
        "Should have at least 10 default actions"
    );
}

#[test]
fn test_chat_message_creation() {
    let system_msg = ChatMessage::system("System message".to_string());
    assert_eq!(system_msg.role, "system");

    let user_msg = ChatMessage::user("User message".to_string());
    assert_eq!(user_msg.role, "user");

    let assistant_msg = ChatMessage::assistant("Assistant message".to_string());
    assert_eq!(assistant_msg.role, "assistant");
}

#[test]
fn test_chat_invoke_usage_tracking() {
    let usage = ChatInvokeUsage {
        prompt_tokens: 1000,
        prompt_cached_tokens: Some(200),
        prompt_cache_creation_tokens: None,
        prompt_image_tokens: None,
        completion_tokens: 500,
        total_tokens: 1500,
    };

    assert_eq!(usage.prompt_tokens, 1000);
    assert_eq!(usage.completion_tokens, 500);
    assert_eq!(usage.total_tokens, 1500);
}

#[test]
fn test_action_result_serialization_roundtrip() {
    let original = ActionResult {
        is_done: Some(true),
        success: Some(true),
        error: None,
        extracted_content: Some("Test content".to_string()),
        long_term_memory: Some("Memory".to_string()),
        ..Default::default()
    };

    let json_str = serde_json::to_string(&original).unwrap();
    let deserialized: ActionResult = serde_json::from_str(&json_str).unwrap();

    assert_eq!(original.is_done, deserialized.is_done);
    assert_eq!(original.extracted_content, deserialized.extracted_content);
    assert_eq!(original.long_term_memory, deserialized.long_term_memory);
}

#[tokio::test]
async fn test_tools_action_parameter_extraction() {
    // Test that action parameters can be correctly extracted
    let params: HashMap<String, serde_json::Value> = serde_json::from_value(json!({
        "index": 1,
        "text": "Hello World"
    }))
    .unwrap();
    let action = ActionModel {
        action_type: "input".to_string(),
        params,
    };

    assert_eq!(action.params.get("index").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(
        action.params.get("text").and_then(|v| v.as_str()),
        Some("Hello World")
    );
}

#[test]
fn test_comprehensive_action_model_coverage() {
    // Test all action types with their typical parameters
    let test_cases = vec![
        ("search", json!({"query": "test query"})),
        (
            "navigate",
            json!({"url": "https://example.com", "new_tab": false}),
        ),
        ("click", json!({"index": 0})),
        ("input", json!({"index": 1, "text": "input text"})),
        ("scroll", json!({"down": true, "pages": 2.0})),
        ("wait", json!({"seconds": 5})),
        ("send_keys", json!({"keys": "Enter Tab"})),
        ("evaluate", json!({"expression": "document.title"})),
        ("find_text", json!({"text": "search text"})),
        ("dropdown_options", json!({"index": 0})),
        ("select_dropdown", json!({"index": 0, "text": "option"})),
        ("upload_file", json!({"index": 0, "path": "/path/to/file"})),
        (
            "extract",
            json!({"query": "extract query", "extract_links": false}),
        ),
        ("done", json!({"text": "Task done", "success": true})),
    ];

    for (action_type, params_json) in test_cases {
        let params: HashMap<String, serde_json::Value> =
            serde_json::from_value(params_json).unwrap();

        let action = ActionModel {
            action_type: action_type.to_string(),
            params: params.clone(),
        };

        assert_eq!(action.action_type, action_type);
        assert_eq!(action.params.len(), params.len());
    }
}
