//! Tests for browser session functionality

use browser_use::browser::views::TabInfo;

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

#[test]
fn test_browser_state_summary_structure() {
    use browser_use::browser::views::BrowserStateSummary;
    use browser_use::dom::views::SerializedDOMState;
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

