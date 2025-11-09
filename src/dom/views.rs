//! DOM view types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Default attributes to include in DOM serialization
pub const DEFAULT_INCLUDE_ATTRIBUTES: &[&str] = &[
    "title", "type", "checked", "id", "name", "role", "value", "placeholder",
    "data-date-format", "alt", "aria-label", "aria-expanded", "data-state",
    "aria-checked", "aria-valuemin", "aria-valuemax", "aria-valuenow",
    "aria-placeholder", "pattern", "min", "max", "minlength", "maxlength",
    "step", "accept", "multiple", "inputmode", "autocomplete", "data-mask",
    "data-inputmask", "data-datepicker", "format", "expected_format",
    "contenteditable", "pseudo", "selected", "expanded", "pressed",
    "disabled", "invalid", "valuemin", "valuemax", "valuenow", "keyshortcuts",
    "haspopup", "multiselectable", "required", "valuetext", "level", "busy",
    "live", "ax_name",
];

/// DOM element that was interacted with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMInteractedElement {
    pub index: u32,
    pub backend_node_id: Option<u32>,
    pub tag: String,
    pub text: Option<String>,
    pub attributes: HashMap<String, String>,
    pub selector: Option<String>,
}

impl DOMInteractedElement {
    pub fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        let mut data = HashMap::new();
        data.insert("index".to_string(), serde_json::to_value(self.index).unwrap());
        data.insert("tag".to_string(), serde_json::to_value(&self.tag).unwrap());
        if let Some(ref text) = self.text {
            data.insert("text".to_string(), serde_json::to_value(text).unwrap());
        }
        data.insert(
            "attributes".to_string(),
            serde_json::to_value(&self.attributes).unwrap(),
        );
        if let Some(ref selector) = self.selector {
            data.insert("selector".to_string(), serde_json::to_value(selector).unwrap());
        }
        data
    }
}

/// Serialized DOM state for LLM processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedDOMState {
    pub html: Option<String>,
    pub text: Option<String>,
    pub markdown: Option<String>,
    pub elements: Vec<DOMElement>,
    pub selector_map: HashMap<u32, DOMInteractedElement>,
}

impl SerializedDOMState {
    /// Get LLM representation of the DOM state
    pub fn llm_representation(&self, _include_attributes: Option<&[&str]>) -> Option<String> {
        // Prefer markdown, then text, then HTML
        if let Some(ref markdown) = self.markdown {
            return Some(markdown.clone());
        }
        if let Some(ref text) = self.text {
            return Some(text.clone());
        }
        if let Some(ref html) = self.html {
            return Some(html.clone());
        }
        None
    }
}

/// DOM element representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMElement {
    pub index: u32,
    pub tag: String,
    pub text: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<DOMElement>,
}

/// Selector map for DOM elements
pub type DOMSelectorMap = HashMap<u32, DOMInteractedElement>;

/// DOM node types based on the DOM specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    ElementNode = 1,
    AttributeNode = 2,
    TextNode = 3,
    CdataSectionNode = 4,
    EntityReferenceNode = 5,
    EntityNode = 6,
    ProcessingInstructionNode = 7,
    CommentNode = 8,
    DocumentNode = 9,
    DocumentTypeNode = 10,
    DocumentFragmentNode = 11,
    NotationNode = 12,
}

/// DOM rectangle for bounding boxes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DOMRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl DOMRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }
}

/// Enhanced accessibility property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAXProperty {
    pub name: String,
    pub value: Option<serde_json::Value>,
}

/// Enhanced accessibility node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAXNode {
    pub ax_node_id: String,
    pub ignored: bool,
    pub role: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub properties: Option<Vec<EnhancedAXProperty>>,
    pub child_ids: Option<Vec<String>>,
}

/// Enhanced snapshot node with layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSnapshotNode {
    pub is_clickable: Option<bool>,
    pub cursor_style: Option<String>,
    pub bounds: Option<DOMRect>,
    pub client_rects: Option<DOMRect>,
    pub scroll_rects: Option<DOMRect>,
    pub computed_styles: Option<HashMap<String, String>>,
    pub paint_order: Option<i32>,
    pub stacking_contexts: Option<i32>,
}

/// Enhanced DOM tree node combining DOM, AX, and Snapshot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDOMTreeNode {
    pub node_id: u64,
    pub backend_node_id: u64,
    pub node_type: NodeType,
    pub node_name: String,
    pub node_value: String,
    pub attributes: HashMap<String, String>,
    pub is_scrollable: Option<bool>,
    pub is_visible: Option<bool>,
    pub absolute_position: Option<DOMRect>,
    
    // Frame information
    pub target_id: String,
    pub frame_id: Option<String>,
    pub session_id: Option<String>,
    pub content_document: Option<Box<EnhancedDOMTreeNode>>,
    
    // Shadow DOM
    pub shadow_root_type: Option<String>,
    pub shadow_roots: Option<Vec<EnhancedDOMTreeNode>>,
    
    // Navigation
    pub parent_node: Option<Box<EnhancedDOMTreeNode>>,
    pub children_nodes: Option<Vec<EnhancedDOMTreeNode>>,
    
    // AX node data
    pub ax_node: Option<EnhancedAXNode>,
    
    // Snapshot node data
    pub snapshot_node: Option<EnhancedSnapshotNode>,
    
    // UUID for tracking
    pub uuid: String,
}

impl EnhancedDOMTreeNode {
    pub fn new(
        node_id: u64,
        backend_node_id: u64,
        node_type: NodeType,
        node_name: String,
        node_value: String,
        target_id: String,
    ) -> Self {
        Self {
            node_id,
            backend_node_id,
            node_type,
            node_name,
            node_value,
            attributes: HashMap::new(),
            is_scrollable: None,
            is_visible: None,
            absolute_position: None,
            target_id,
            frame_id: None,
            session_id: None,
            content_document: None,
            shadow_root_type: None,
            shadow_roots: None,
            parent_node: None,
            children_nodes: None,
            ax_node: None,
            snapshot_node: None,
            uuid: Uuid::now_v7().to_string(),
        }
    }

    pub fn tag_name(&self) -> String {
        self.node_name.to_lowercase()
    }
}
