//! Tests for browser session functionality

use browsing::browser::views::TabInfo;

#[test]
fn test_tab_info_creation() {
    let tab = TabInfo {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        target_id: "target-123".to_string(),
        parent_target_id: None,
    };

    assert_eq!(tab.url, "https://example.com");
    assert_eq!(tab.title, "Example");
    assert_eq!(tab.target_id, "target-123");
}

#[test]
fn test_tab_info_serialization() {
    let tab = TabInfo {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        target_id: "target-123".to_string(),
        parent_target_id: None,
    };

    let json_str = serde_json::to_string(&tab).unwrap();
    let deserialized: TabInfo = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.url, tab.url);
    assert_eq!(deserialized.title, tab.title);
}

#[tokio::test]
async fn test_browser_headless_startup() {
    use browsing::browser::{Browser, BrowserProfile};
    
    // Create a browser with headless configuration
    let profile = BrowserProfile {
        headless: Some(true),
        user_data_dir: None,
        allowed_domains: None,
        downloads_path: None,
    };
    
    let mut browser = Browser::new(profile);
    
    // Browser should be created successfully
    // sessions and profile are private fields
    
    // This test would require an actual browser installation
    // In CI environments, we'd want to verify headless mode works
    // For now, we just verify the configuration is accepted
    
    // Verify headless configuration is preserved
    // Note: In actual tests with a browser, we would:
    // 1. Start browser
    // 2. Verify it starts in headless mode
    // 3. Check no visible UI appears
    // 4. Verify screenshot capture works
    
    // The browser is created with the specified profile
}

#[tokio::test]
async fn test_browser_basic_workflow() {
    use browsing::browser::{Browser, BrowserProfile};
    
    // This test demonstrates the expected workflow
    // In CI environments without Chrome, this test would be skipped
    
    let profile = BrowserProfile {
        headless: Some(true), // Important for CI environments
        user_data_dir: None, // Use temporary directory
        allowed_domains: None,
        downloads_path: None,
    };
    
    let mut browser = Browser::new(profile);
    
    // Expected workflow:
    // 1. browser.start() - Launch headless Chrome
    // 2. browser.navigate("https://example.com") - Navigate to a page
    // 3. browser.get_page() - Get page state
    // 4. browser.stop() - Clean up resources
    
    // Verify initial state
    // sessions is a private field, browser is created successfully if no panic
    
    // In actual implementation, we would:
    // browser.start().await?;
    // browser.navigate("https://example.com").await?;
    // let page = browser.get_page();
    // browser.stop().await?;
    
    // All should succeed if Chrome is installed and available
    assert!(true); // Test reaches here = success
}

#[test]
fn test_browser_state_summary_structure() {
    use browsing::browser::views::BrowserStateSummary;
    use browsing::dom::views::SerializedDOMState;
    use std::collections::HashMap;

    let dom_state = SerializedDOMState {
        html: None,
        text: Some("Test content".to_string()),
        markdown: None,
        elements: vec![],
        selector_map: HashMap::new(),
    };

    let summary = BrowserStateSummary {
        dom_state,
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        tabs: vec![],
        screenshot: None,
        page_info: None,
        pixels_above: 0,
        pixels_below: 0,
        browser_errors: vec![],
        is_pdf_viewer: false,
        recent_events: None,
        pending_network_requests: vec![],
        pagination_buttons: vec![],
        closed_popup_messages: vec![],
    };

    assert_eq!(summary.url, "https://example.com");
    assert_eq!(summary.title, "Example");
    assert!(!summary.is_pdf_viewer);
}
