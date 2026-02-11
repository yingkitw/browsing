//! DOM view types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Default attributes to include in DOM serialization
pub const DEFAULT_INCLUDE_ATTRIBUTES: &[&str] = &[
    "title",
    "type",
    "checked",
    "id",
    "name",
    "role",
    "value",
    "placeholder",
    "data-date-format",
    "alt",
    "aria-label",
    "aria-expanded",
    "data-state",
    "aria-checked",
    "aria-valuemin",
    "aria-valuemax",
    "aria-valuenow",
    "aria-placeholder",
    "pattern",
    "min",
    "max",
    "minlength",
    "maxlength",
    "step",
    "accept",
    "multiple",
    "inputmode",
    "autocomplete",
    "data-mask",
    "data-inputmask",
    "data-datepicker",
    "format",
    "expected_format",
    "contenteditable",
    "pseudo",
    "selected",
    "expanded",
    "pressed",
    "disabled",
    "invalid",
    "valuemin",
    "valuemax",
    "valuenow",
    "keyshortcuts",
    "haspopup",
    "multiselectable",
    "required",
    "valuetext",
    "level",
    "busy",
    "live",
    "ax_name",
];

/// DOM element that was interacted with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMInteractedElement {
    /// Index of the element
    pub index: u32,
    /// Backend node ID of the element
    pub backend_node_id: Option<u32>,
    /// HTML tag of the element
    pub tag: String,
    /// Text content of the element
    pub text: Option<String>,
    /// Attributes of the element
    pub attributes: HashMap<String, String>,
    /// CSS selector of the element
    pub selector: Option<String>,
}

impl DOMInteractedElement {
    /// Converts the element to a dictionary
    pub fn to_dict(&self) -> HashMap<String, serde_json::Value> {
        serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap()
    }
}

/// Serialized DOM state for LLM processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedDOMState {
    /// HTML representation of the DOM
    pub html: Option<String>,
    /// Text content of the DOM
    pub text: Option<String>,
    /// Markdown representation of the DOM
    pub markdown: Option<String>,
    /// List of DOM elements
    pub elements: Vec<DOMElement>,
    /// Selector map for DOM elements
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
    /// Index of the element
    pub index: u32,
    /// HTML tag of the element
    pub tag: String,
    /// Text content of the element
    pub text: Option<String>,
    /// Attributes of the element
    pub attributes: HashMap<String, String>,
    /// Child elements
    pub children: Vec<DOMElement>,
}

/// Selector map for DOM elements
pub type DOMSelectorMap = HashMap<u32, DOMInteractedElement>;

/// DOM node types based on the DOM specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    /// Element node
    ElementNode = 1,
    /// Attribute node
    AttributeNode = 2,
    /// Text node
    TextNode = 3,
    /// CDATA section node
    CdataSectionNode = 4,
    /// Entity reference node
    EntityReferenceNode = 5,
    /// Entity node
    EntityNode = 6,
    /// Processing instruction node
    ProcessingInstructionNode = 7,
    /// Comment node
    CommentNode = 8,
    /// Document node
    DocumentNode = 9,
    /// Document type node
    DocumentTypeNode = 10,
    /// Document fragment node
    DocumentFragmentNode = 11,
    /// Notation node
    NotationNode = 12,
}

/// DOM rectangle for bounding boxes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DOMRect {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Width of the rectangle
    pub width: f64,
    /// Height of the rectangle
    pub height: f64,
}

impl DOMRect {
    /// Creates a new DOM rectangle
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Enhanced accessibility property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAXProperty {
    /// Name of the property
    pub name: String,
    /// Value of the property
    pub value: Option<serde_json::Value>,
}

/// Enhanced accessibility node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAXNode {
    /// Accessibility node ID
    pub ax_node_id: String,
    /// Whether the node is ignored
    pub ignored: bool,
    /// Role of the node
    pub role: Option<String>,
    /// Name of the node
    pub name: Option<String>,
    /// Description of the node
    pub description: Option<String>,
    /// Properties of the node
    pub properties: Option<Vec<EnhancedAXProperty>>,
    /// IDs of child nodes
    pub child_ids: Option<Vec<String>>,
}

/// Enhanced snapshot node with layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSnapshotNode {
    /// Whether the node is clickable
    pub is_clickable: Option<bool>,
    /// Cursor style of the node
    pub cursor_style: Option<String>,
    /// Bounding rectangle
    pub bounds: Option<DOMRect>,
    /// Client rectangle
    pub client_rects: Option<DOMRect>,
    /// Scroll rectangle
    pub scroll_rects: Option<DOMRect>,
    /// Computed CSS styles
    pub computed_styles: Option<HashMap<String, String>>,
    /// Paint order
    pub paint_order: Option<i32>,
    /// Stacking contexts
    pub stacking_contexts: Option<i32>,
}

/// Enhanced DOM tree node combining DOM, AX, and Snapshot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDOMTreeNode {
    /// Node ID
    pub node_id: u64,
    /// Backend node ID
    pub backend_node_id: u64,
    /// Type of the node
    pub node_type: NodeType,
    /// Name of the node
    pub node_name: String,
    /// Value of the node
    pub node_value: String,
    /// Attributes of the node
    pub attributes: HashMap<String, String>,
    /// Whether the node is scrollable
    pub is_scrollable: Option<bool>,
    /// Whether the node is visible
    pub is_visible: Option<bool>,
    /// Absolute position of the node
    pub absolute_position: Option<DOMRect>,

    // Frame information
    /// Target ID
    pub target_id: String,
    /// Frame ID
    pub frame_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Content document
    pub content_document: Option<Box<EnhancedDOMTreeNode>>,

    // Shadow DOM
    /// Shadow root type
    pub shadow_root_type: Option<String>,
    /// Shadow roots
    pub shadow_roots: Option<Vec<EnhancedDOMTreeNode>>,

    // Navigation
    /// Parent node
    pub parent_node: Option<Box<EnhancedDOMTreeNode>>,
    /// Child nodes
    pub children_nodes: Option<Vec<EnhancedDOMTreeNode>>,

    // AX node data
    /// Accessibility node data
    pub ax_node: Option<EnhancedAXNode>,

    // Snapshot node data
    /// Snapshot node data
    pub snapshot_node: Option<EnhancedSnapshotNode>,

    // UUID for tracking
    /// UUID for tracking
    pub uuid: String,
}

impl EnhancedDOMTreeNode {
    /// Creates a new enhanced DOM tree node
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

    /// Returns the tag name of the node
    pub fn tag_name(&self) -> String {
        self.node_name.to_lowercase()
    }
}
