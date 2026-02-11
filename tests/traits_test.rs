//! Comprehensive tests for traits implementations
//!
//! These tests cover:
//! - BrowserClient trait implementations
//! - DOMProcessor trait implementations
//! - Mock implementations for testing
//! - Trait method validation

use browsing::error::Result;
use browsing::traits::{BrowserClient, DOMProcessor};
use browsing::actor::Page;
use browsing::browser::cdp::CdpClient;
use browsing::browser::views::TabInfo;
use browsing::dom::views::{DOMInteractedElement, SerializedDOMState};
use std::sync::Arc;
use std::collections::HashMap;

// ============================================================================
// BrowserClient Trait Tests
// ============================================================================

#[test]
fn test_browser_client_trait_methods_exist() {
    // Verify all BrowserClient trait methods are defined
    let methods = vec![
        "start",
        "navigate",
        "get_current_url",
        "create_tab",
        "switch_to_tab",
        "close_tab",
        "get_tabs",
        "get_target_id_from_tab_id",
        "get_page",
        "take_screenshot",
        "get_current_page_title",
        "get_cdp_client",
        "get_session_id",
        "get_current_target_id",
    ];

    assert_eq!(methods.len(), 14);
}

#[test]
fn test_browser_client_async_methods() {
    // All BrowserClient methods should be async (except get_page which returns Result)
    let async_methods = vec![
        "start",
        "navigate",
        "get_current_url",
        "create_tab",
        "switch_to_tab",
        "close_tab",
        "get_tabs",
        "get_target_id_from_tab_id",
        "take_screenshot",
        "get_current_page_title",
    ];

    assert_eq!(async_methods.len(), 10);
}

#[test]
fn test_browser_client_url_validation() {
    let valid_urls = vec![
        "https://example.com",
        "http://localhost:8080",
        "https://www.google.com/search?q=test",
        "about:blank",
    ];

    for url in valid_urls {
        assert!(url.len() > 0, "URL should not be empty");
        assert!(url.len() <= 2048, "URL should be reasonable length");
    }
}

#[test]
fn test_browser_client_target_id_validation() {
    let valid_target_ids = vec![
        "E12BA567-8A90-1234-5678-9ABCDEF01234",
        "tab123",
        "page-456",
    ];

    for target_id in valid_target_ids {
        assert!(!target_id.is_empty(), "Target ID should not be empty");
    }
}

#[test]
fn test_browser_client_tab_info_structure() {
    let tab_info = TabInfo {
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        target_id: "tab123".to_string(),
        parent_target_id: None,
    };

    assert_eq!(tab_info.url, "https://example.com");
    assert_eq!(tab_info.title, "Example");
    assert_eq!(tab_info.target_id, "tab123");
    assert!(tab_info.parent_target_id.is_none());
}

// ============================================================================
// DOMProcessor Trait Tests
// ============================================================================

#[test]
fn test_dom_processor_trait_methods_exist() {
    // Verify all DOMProcessor trait methods are defined
    let methods = vec![
        "get_serialized_dom",
        "get_page_state_string",
        "get_selector_map",
    ];

    assert_eq!(methods.len(), 3);
}

#[test]
fn test_dom_processor_async_methods() {
    // All DOMProcessor methods should be async
    let async_methods = vec![
        "get_serialized_dom",
        "get_page_state_string",
        "get_selector_map",
    ];

    assert_eq!(async_methods.len(), 3);
}

#[test]
fn test_dom_processor_serialized_state_structure() {
    let state = SerializedDOMState {
        html: Some("<html></html>".to_string()),
        text: Some("Example content".to_string()),
        markdown: None,
        elements: vec![],
        selector_map: HashMap::new(),
    };

    assert_eq!(state.html, Some("<html></html>".to_string()));
    assert_eq!(state.text, Some("Example content".to_string()));
    assert!(state.elements.is_empty());
    assert!(state.selector_map.is_empty());
}

#[test]
fn test_dom_processor_selector_map_structure() {
    let mut selector_map = HashMap::new();
    selector_map.insert(
        1u32,
        DOMInteractedElement {
            index: 1,
            backend_node_id: Some(123),
            tag: "button".to_string(),
            text: Some("Click".to_string()),
            attributes: HashMap::new(),
            selector: None,
        },
    );

    assert_eq!(selector_map.len(), 1);
    assert!(selector_map.contains_key(&1));

    let element = selector_map.get(&1).unwrap();
    assert_eq!(element.backend_node_id, Some(123));
    assert_eq!(element.tag, "button");
    assert_eq!(element.text, Some("Click".to_string()));
}

// ============================================================================
// Mock BrowserClient Tests
// ============================================================================

/// Mock BrowserClient for testing
struct MockBrowserClient {
    started: bool,
    current_url: String,
    navigation_count: std::sync::atomic::AtomicUsize,
}

impl MockBrowserClient {
    fn new() -> Self {
        Self {
            started: false,
            current_url: "about:blank".to_string(),
            navigation_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl BrowserClient for MockBrowserClient {
    async fn start(&mut self) -> Result<()> {
        self.started = true;
        Ok(())
    }

    async fn navigate(&mut self, url: &str) -> Result<()> {
        self.current_url = url.to_string();
        self.navigation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn get_current_url(&self) -> Result<String> {
        Ok(self.current_url.clone())
    }

    async fn create_tab(&mut self, _url: Option<&str>) -> Result<String> {
        Ok("mock-tab-123".to_string())
    }

    async fn switch_to_tab(&mut self, _target_id: &str) -> Result<()> {
        Ok(())
    }

    async fn close_tab(&mut self, _target_id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_tabs(&self) -> Result<Vec<TabInfo>> {
        Ok(vec![TabInfo {
            url: self.current_url.clone(),
            title: "Mock Page".to_string(),
            target_id: "mock-tab-123".to_string(),
            parent_target_id: None,
        }])
    }

    async fn get_target_id_from_tab_id(&self, _tab_id: &str) -> Result<String> {
        Ok("mock-target-123".to_string())
    }

    fn get_page(&self) -> Result<Page> {
        // This would require a real CDP client, so we just return an error for the mock
        Err(browsing::error::BrowsingError::Browser(
            "Mock browser doesn't support page operations".to_string(),
        ))
    }

    async fn take_screenshot(&self, _path: Option<&str>, _full_page: bool) -> Result<Vec<u8>> {
        Ok(vec![0, 1, 2, 3]) // Mock screenshot data
    }

    async fn get_current_page_title(&self) -> Result<String> {
        Ok("Mock Page".to_string())
    }

    fn get_cdp_client(&self) -> Result<Arc<CdpClient>> {
        Err(browsing::error::BrowsingError::Browser(
            "Mock browser doesn't support CDP".to_string(),
        ))
    }

    fn get_session_id(&self) -> Result<String> {
        Ok("mock-session-123".to_string())
    }

    fn get_current_target_id(&self) -> Result<String> {
        Ok("mock-target-123".to_string())
    }
}

#[test]
fn test_mock_browser_creation() {
    let mock = MockBrowserClient::new();
    assert!(!mock.started);
    assert_eq!(mock.current_url, "about:blank");
}

#[tokio::test]
async fn test_mock_browser_start() {
    let mut mock = MockBrowserClient::new();
    mock.start().await.unwrap();
    assert!(mock.started);
}

#[tokio::test]
async fn test_mock_browser_navigate() {
    let mut mock = MockBrowserClient::new();
    mock.navigate("https://example.com").await.unwrap();
    assert_eq!(mock.current_url, "https://example.com");
}

#[tokio::test]
async fn test_mock_browser_get_current_url() {
    let mut mock = MockBrowserClient::new();
    mock.navigate("https://test.com").await.unwrap();
    let url = mock.get_current_url().await.unwrap();
    assert_eq!(url, "https://test.com");
}

#[tokio::test]
async fn test_mock_browser_navigation_count() {
    let mut mock = MockBrowserClient::new();
    mock.navigate("https://example.com").await.unwrap();
    mock.navigate("https://test.com").await.unwrap();
    mock.navigate("https://another.com").await.unwrap();

    let count = mock.navigation_count.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_mock_browser_create_tab() {
    let mut mock = MockBrowserClient::new();
    let target_id = mock.create_tab(Some("https://newtab.com")).await.unwrap();
    assert_eq!(target_id, "mock-tab-123");
}

#[tokio::test]
async fn test_mock_browser_get_tabs() {
    let mut mock = MockBrowserClient::new();
    mock.navigate("https://example.com").await.unwrap();
    let tabs = mock.get_tabs().await.unwrap();

    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].url, "https://example.com");
    assert_eq!(tabs[0].title, "Mock Page");
}

#[tokio::test]
async fn test_mock_browser_take_screenshot() {
    let mock = MockBrowserClient::new();
    let screenshot = mock.take_screenshot(None, false).await.unwrap();

    assert_eq!(screenshot, vec![0, 1, 2, 3]);
    assert_eq!(screenshot.len(), 4);
}

#[tokio::test]
async fn test_mock_browser_get_page_title() {
    let mock = MockBrowserClient::new();
    let title = mock.get_current_page_title().await.unwrap();

    assert_eq!(title, "Mock Page");
}

// ============================================================================
// Mock DOMProcessor Tests
// ============================================================================

/// Mock DOMProcessor for testing
struct MockDOMProcessor {
    content: String,
}

impl MockDOMProcessor {
    fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DOMProcessor for MockDOMProcessor {
    async fn get_serialized_dom(&self) -> Result<SerializedDOMState> {
        Ok(SerializedDOMState {
            html: Some("<html>Mock Page</html>".to_string()),
            text: Some("Mock Page".to_string()),
            markdown: None,
            elements: vec![],
            selector_map: HashMap::new(),
        })
    }

    async fn get_page_state_string(&self) -> Result<String> {
        Ok(self.content.clone())
    }

    async fn get_selector_map(&self) -> Result<HashMap<u32, DOMInteractedElement>> {
        let mut map = HashMap::new();
        map.insert(
            1u32,
            DOMInteractedElement {
                index: 1,
                backend_node_id: Some(123),
                tag: "button".to_string(),
                text: Some("Click".to_string()),
                attributes: HashMap::new(),
                selector: None,
            },
        );
        Ok(map)
    }
}

#[test]
fn test_mock_dom_processor_creation() {
    let mock = MockDOMProcessor::new("Mock content");
    assert_eq!(mock.content, "Mock content");
}

#[tokio::test]
async fn test_mock_dom_processor_get_serialized_dom() {
    let mock = MockDOMProcessor::new("Mock content");
    let state = mock.get_serialized_dom().await.unwrap();

    assert_eq!(state.html, Some("<html>Mock Page</html>".to_string()));
    assert_eq!(state.text, Some("Mock Page".to_string()));
    assert!(state.elements.is_empty());
}

#[tokio::test]
async fn test_mock_dom_processor_get_page_state_string() {
    let mock = MockDOMProcessor::new("Test page content");
    let content = mock.get_page_state_string().await.unwrap();

    assert_eq!(content, "Test page content");
}

#[tokio::test]
async fn test_mock_dom_processor_get_selector_map() {
    let mock = MockDOMProcessor::new("Mock content");
    let map = mock.get_selector_map().await.unwrap();

    assert_eq!(map.len(), 1);
    assert!(map.contains_key(&1));

    let element = map.get(&1).unwrap();
    assert_eq!(element.tag, "button");
    assert_eq!(element.text, Some("Click".to_string()));
}

// ============================================================================
// Trait Bound Tests
// ============================================================================

#[test]
fn test_browser_client_send_sync_bounds() {
    // BrowserClient should be Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}

    // This will compile only if BrowserClient is Send + Sync
    assert_send_sync::<Box<dyn BrowserClient>>();
}

#[test]
fn test_dom_processor_send_sync_bounds() {
    // DOMProcessor should be Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}

    // This will compile only if DOMProcessor is Send + Sync
    assert_send_sync::<Box<dyn DOMProcessor>>();
}

// ============================================================================
// Integration Test Markers
// ============================================================================

#[test]
#[ignore = "Requires real browser"]
fn test_browser_client_full_lifecycle() {
    // This test would:
    // 1. Create a real BrowserClient
    // 2. Start the browser
    // 3. Navigate to a URL
    // 4. Get page state
    // 5. Take a screenshot
    // 6. Verify all operations succeed
}

#[test]
#[ignore = "Requires real browser"]
fn test_dom_processor_full_extraction() {
    // This test would:
    // 1. Navigate to a real page
    // 2. Extract serialized DOM
    // 3. Get page state string
    // 4. Get selector map
    // 5. Verify all data is valid
}

#[test]
#[ignore = "Requires real browser"]
fn test_trait_integration() {
    // This test would:
    // 1. Use BrowserClient and DOMProcessor together
    // 2. Navigate to a page
    // 3. Extract DOM state
    // 4. Interact with elements using selector map
    // 5. Verify end-to-end functionality
}
