//! Browser view types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents information about a browser tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub url: String,
    pub title: String,
    #[serde(alias = "tab_id")]
    pub target_id: String,
    #[serde(alias = "parent_tab_id")]
    pub parent_target_id: Option<String>,
}

/// Comprehensive page size and scroll information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub page_width: u32,
    pub page_height: u32,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub pixels_above: u32,
    pub pixels_below: u32,
    pub pixels_left: u32,
    pub pixels_right: u32,
}

/// Information about a pending network request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub loading_duration_ms: f64,
    pub resource_type: Option<String>,
}

/// Information about a pagination button detected on the page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationButton {
    pub button_type: String, // 'next', 'prev', 'first', 'last', 'page_number'
    pub backend_node_id: u32,
    pub text: String,
    pub selector: String,
    pub is_disabled: bool,
}

/// The summary of the browser's current state designed for an LLM to process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateSummary {
    pub dom_state: crate::dom::views::SerializedDOMState,
    pub url: String,
    pub title: String,
    pub tabs: Vec<TabInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>, // base64 encoded
    pub page_info: Option<PageInfo>,
    pub pixels_above: u32,
    pub pixels_below: u32,
    pub browser_errors: Vec<String>,
    pub is_pdf_viewer: bool,
    pub recent_events: Option<String>,
    pub pending_network_requests: Vec<NetworkRequest>,
    pub pagination_buttons: Vec<PaginationButton>,
    pub closed_popup_messages: Vec<String>,
}

/// The summary of the browser's state at a past point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateHistory {
    pub url: String,
    pub title: String,
    pub tabs: Vec<TabInfo>,
    pub interacted_element: Vec<Option<crate::dom::views::DOMInteractedElement>>,
    pub screenshot_path: Option<String>,
}

impl BrowserStateHistory {
    pub fn get_screenshot(&self) -> Option<String> {
        if let Some(ref path) = self.screenshot_path {
            if let Ok(data) = std::fs::read(path) {
                use base64::{Engine as _, engine::general_purpose};
                return Some(general_purpose::STANDARD.encode(&data));
            }
        }
        None
    }

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

