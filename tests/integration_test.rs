//! Integration tests for browser-use

use browser_use::tools::service::Tools;
use browser_use::tools::views::ActionModel;
use browser_use::utils::extract_urls;
use serde_json::json;

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
        ("go_back", json!({})),
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

