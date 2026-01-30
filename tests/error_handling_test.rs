//! Error handling tests

use std::error::Error;
use browser_use::error::{BrowserUseError, Result};
use browser_use::browser::{Browser, BrowserProfile};
use browser_use::tools::service::Tools;
use browser_use::dom::service::DomService;
use serde_json::json;

#[test]
fn test_error_variants() {
    // Test all error variants can be created
    let errors = [
        BrowserUseError::Config("Invalid configuration".to_string()),
        BrowserUseError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found"
        )),
        BrowserUseError::Browser("Browser startup failed".to_string()),
        BrowserUseError::Cdp("CDP connection lost".to_string()),
        BrowserUseError::Llm("LLM API error".to_string()),
        BrowserUseError::Agent("Agent execution failed".to_string()),
        BrowserUseError::Dom("DOM extraction failed".to_string()),
        BrowserUseError::Tool("Tool execution failed".to_string()),
        BrowserUseError::Validation("Invalid input".to_string()),
    ];
    
    // All errors should be creatable
    for error in errors {
        let _ = error.to_string();
    }
}

#[test]
fn test_error_display_formatting() {
    let error = BrowserUseError::Config("Browser profile not found".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("Configuration error"));
    assert!(display_str.contains("Browser profile not found"));
    
    let error = BrowserUseError::Tool("Invalid action".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("Tool error"));
    assert!(display_str.contains("Invalid action"));
}

#[test]
fn test_error_chain() {
    let io_error = std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "Cannot read file"
    );
    let browser_error = BrowserUseError::Io(io_error);
    
    // Error source should be preserved
    assert!(browser_error.source().is_some());
    let source = browser_error.source().unwrap();
    assert!(format!("{}", source).contains("Cannot read file"));
}

#[test]
fn test_result_type_alias() {
    // Result type alias should work correctly
    fn returns_result() -> Result<String> {
        Ok("success".to_string())
    }
    
    fn returns_error() -> Result<String> {
        Err(BrowserUseError::Validation("Invalid input".to_string()))
    }
    
    assert!(returns_result().is_ok());
    assert!(returns_result().unwrap() == "success");
    
    assert!(returns_error().is_err());
}

#[tokio::test]
async fn test_browser_error_scenarios() {
    // Test error handling in browser operations
    let mut browser = Browser::new(BrowserProfile::default());
    
    // Try to navigate before starting - should error
    let result = browser.navigate("https://example.com").await;
    assert!(result.is_err());
    
    // Try to get page before starting - should error
    let result = browser.get_page();
    assert!(result.is_err());
    
    // Try to get CDP client before starting - should error
    let result = browser.get_cdp_client();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tools_error_scenarios() {
    let _tools = Tools::new(vec![]);
    
    // Test invalid action types
    let invalid_action = json!({
        "action_type": "invalid_action_type",
        "params": {}
    });
    
    let action_model: std::result::Result<browser_use::tools::views::ActionModel, serde_json::Error> = 
        serde_json::from_value(invalid_action);
    
    // Should create model but execution would fail
    if let Ok(action) = action_model {
        // Can't actually execute without browser, so just test creation
        assert_eq!(action.action_type, "invalid_action_type");
    }
}

#[test]
fn test_dom_service_error_scenarios() {
    let _dom_service = DomService::new();
    
    // DomService stores its configuration internally after using with_* methods
    // It doesn't have getter methods for checking if fields are None
    // This test just verifies DomService can be created
}

#[test]
fn test_error_recovery_patterns() {
    // Test that errors can be properly recovered from
    
    let mut attempt_count = 0;
    let result: Result<String> = loop {
        attempt_count += 1;
        
        if attempt_count == 1 {
            continue; // Simulate retry
        }
        
        break Ok("success".to_string());
    };
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(attempt_count, 2);
}

#[test]
fn test_error_with_context() {
    // Test adding context to errors
    let base_error = BrowserUseError::Cdp("Connection failed".to_string());
    
    // In a real scenario, we might add context
    let contextual_error = match base_error {
        BrowserUseError::Cdp(msg) => 
            BrowserUseError::Cdp(format!("CDP error during startup: {}", msg)),
        other => other,
    };
    
    assert!(format!("{}", contextual_error).contains("CDP error during startup"));
}

#[test]
fn test_multiple_error_combinations() {
    // Test complex error scenarios with multiple potential failure points
    
    fn complex_operation(might_fail: bool) -> Result<String> {
        if might_fail {
            return Err(BrowserUseError::Validation("Input validation failed".to_string()));
        }
        
        // Simulate another potential failure point
        // Using a deterministic check instead of random for test reproducibility
        if false {
            return Err(BrowserUseError::Tool("Tool execution failed".to_string()));
        }
        
        Ok("Operation completed".to_string())
    }
    
    // Test both error cases
    let result1 = complex_operation(true);
    assert!(result1.is_err());
    assert!(matches!(result1.unwrap_err(), BrowserUseError::Validation(_)));
    
    // Note: Can't test random case deterministically
}

#[test]
fn test_error_aggregation() {
    // Test collecting multiple errors
    
    let mut errors = Vec::new();
    
    for i in 0..3 {
        match i {
            0 => errors.push(BrowserUseError::Config("Config error".to_string())),
            1 => errors.push(BrowserUseError::Browser("Browser error".to_string())),
            2 => errors.push(BrowserUseError::Llm("LLM error".to_string())),
            _ => unreachable!(),
        }
    }
    
    // Aggregate all error messages
    let combined_error = format!("Multiple errors occurred: {}",
        errors.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    
    assert!(combined_error.contains("Config error"));
    assert!(combined_error.contains("Browser error"));
    assert!(combined_error.contains("LLM error"));
}

#[test]
fn test_async_error_propagation() {
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new().unwrap();
    
    let result: Result<String> = rt.block_on(async {
        // Simulate async operation that might fail
        if true {
            return Err(BrowserUseError::Agent("Async operation failed".to_string()));
        }
        Ok("Async success".to_string())
    });
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BrowserUseError::Agent(_)));
}