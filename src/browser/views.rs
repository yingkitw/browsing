//! Browser view types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents information about a browser tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    /// URL of the tab
    pub url: String,
    /// Title of the tab
    pub title: String,
    /// Target ID of the tab
    #[serde(alias = "tab_id")]
    pub target_id: String,
    /// Parent target ID if this is a nested tab
    #[serde(alias = "parent_tab_id")]
    pub parent_target_id: Option<String>,
}

/// Comprehensive page size and scroll information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    /// Width of the viewport
    pub viewport_width: u32,
    /// Height of the viewport
    pub viewport_height: u32,
    /// Total width of the page
    pub page_width: u32,
    /// Total height of the page
    pub page_height: u32,
    /// Horizontal scroll position
    pub scroll_x: i32,
    /// Vertical scroll position
    pub scroll_y: i32,
    /// Pixels above current viewport
    pub pixels_above: u32,
    /// Pixels below current viewport
    pub pixels_below: u32,
    /// Pixels to the left of current viewport
    pub pixels_left: u32,
    /// Pixels to the right of current viewport
    pub pixels_right: u32,
}

/// Information about a pending network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// URL of the request
    pub url: String,
    /// HTTP method used
    pub method: String,
    /// Duration of loading in milliseconds
    pub loading_duration_ms: f64,
    /// Type of resource being requested
    pub resource_type: Option<String>,
}

/// Information about a pagination button detected on the page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationButton {
    /// Type of button ('next', 'prev', 'first', 'last', 'page_number')
    pub button_type: String,
    /// Backend node ID of the button
    pub backend_node_id: u32,
    /// Text content of the button
    pub text: String,
    /// CSS selector for the button
    pub selector: String,
    /// Whether the button is disabled
    pub is_disabled: bool,
}

/// The summary of the browser's current state designed for an LLM to process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateSummary {
    /// Serialized DOM state
    pub dom_state: crate::dom::views::SerializedDOMState,
    /// Current URL
    pub url: String,
    /// Page title
    pub title: String,
    /// List of open tabs
    pub tabs: Vec<TabInfo>,
    /// Base64 encoded screenshot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,
    /// Page information
    pub page_info: Option<PageInfo>,
    /// Pixels above current viewport
    pub pixels_above: u32,
    /// Pixels below current viewport
    pub pixels_below: u32,
    /// List of browser errors
    pub browser_errors: Vec<String>,
    /// Whether viewing a PDF
    pub is_pdf_viewer: bool,
    /// Recent browser events
    pub recent_events: Option<String>,
    /// List of pending network requests
    pub pending_network_requests: Vec<NetworkRequest>,
    /// List of pagination buttons
    pub pagination_buttons: Vec<PaginationButton>,
    /// List of closed popup messages
    pub closed_popup_messages: Vec<String>,
}

/// The summary of the browser's state at a past point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateHistory {
    /// URL at the time
    pub url: String,
    /// Title at the time
    pub title: String,
    /// Tabs at the time
    pub tabs: Vec<TabInfo>,
    /// Elements that were interacted with
    pub interacted_element: Vec<Option<crate::dom::views::DOMInteractedElement>>,
    /// Path to screenshot file
    pub screenshot_path: Option<String>,
}

impl BrowserStateHistory {
    /// Gets the screenshot as base64 string
    pub fn get_screenshot(&self) -> Option<String> {
        if let Some(ref path) = self.screenshot_path {
            if let Ok(data) = std::fs::read(path) {
                use base64::{engine::general_purpose, Engine as _};
                return Some(general_purpose::STANDARD.encode(&data));
            }
        }
        None
    }

    /// Converts the state history to a dictionary
    pub fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut data = HashMap::new();
        data.insert(
            "tabs".to_string(),
            serde_json::to_value(&self.tabs).unwrap(),
        );
        data.insert(
            "screenshot_path".to_string(),
            serde_json::to_value(&self.screenshot_path).unwrap(),
        );
        data.insert(
            "interacted_element".to_string(),
            serde_json::to_value(&self.interacted_element).unwrap(),
        );
        data.insert("url".to_string(), serde_json::to_value(&self.url).unwrap());
        data.insert(
            "title".to_string(),
            serde_json::to_value(&self.title).unwrap(),
        );
        data
    }
}
