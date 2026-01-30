//! DOM serializer for LLM representation

use crate::dom::views::{
    DOMInteractedElement, EnhancedDOMTreeNode, NodeType, DEFAULT_INCLUDE_ATTRIBUTES,
};
use std::collections::HashMap;

/// Simplified node for serialization
#[derive(Debug, Clone)]
pub struct SimplifiedNode {
    /// Original enhanced DOM tree node
    pub original_node: EnhancedDOMTreeNode,
    /// Child nodes
    pub children: Vec<SimplifiedNode>,
    /// Whether this node should be displayed
    pub should_display: bool,
    /// Whether this node is interactive
    pub is_interactive: bool,
    /// Interactive index if applicable
    pub interactive_index: Option<u32>,
}

impl SimplifiedNode {
    /// Creates a new simplified node from an enhanced DOM tree node
    pub fn new(node: EnhancedDOMTreeNode) -> Self {
        Self {
            original_node: node,
            children: Vec::new(),
            should_display: true,
            is_interactive: false,
            interactive_index: None,
        }
    }
}

/// DOM tree serializer
pub struct DOMTreeSerializer {
    /// Root node of the DOM tree
    root_node: EnhancedDOMTreeNode,
    /// Counter for interactive elements
    interactive_counter: u32,
    /// Map of selectors
    selector_map: HashMap<u32, DOMInteractedElement>,
}

impl DOMTreeSerializer {
    /// Creates a new DOM tree serializer
    pub fn new(root_node: EnhancedDOMTreeNode) -> Self {
        Self {
            root_node,
            interactive_counter: 1,
            selector_map: HashMap::new(),
        }
    }

    /// Serialize accessible elements and build selector map
    pub fn serialize_accessible_elements(mut self) -> (SerializedDOMState, HashMap<String, f64>) {
        // Reset state
        self.interactive_counter = 1;
        self.selector_map.clear();

        // Create simplified tree
        let simplified_tree = self._create_simplified_tree(&self.root_node);

        // Assign interactive indices (need mutable reference)
        let mut simplified_tree_mut = simplified_tree;
        self._assign_interactive_indices(&mut simplified_tree_mut);
        let simplified_tree = simplified_tree_mut;

        // Serialize to string
        let serialized_string =
            Self::serialize_tree(&simplified_tree, DEFAULT_INCLUDE_ATTRIBUTES, 0);

        let serialized_state = SerializedDOMState {
            html: None,
            text: Some(serialized_string.clone()),
            markdown: Some(serialized_string),
            elements: vec![],
            selector_map: self.selector_map,
        };

        (serialized_state, HashMap::new())
    }

    /// Create simplified tree from enhanced DOM tree
    fn _create_simplified_tree(&self, node: &EnhancedDOMTreeNode) -> SimplifiedNode {
        let mut simplified = SimplifiedNode::new(node.clone());

        // Determine if node should be displayed
        simplified.should_display = self._should_display_node(node);

        // Process children
        if let Some(ref children) = node.children_nodes {
            for child in children {
                let child_simplified = self._create_simplified_tree(child);
                simplified.children.push(child_simplified);
            }
        }

        // Process shadow roots
        if let Some(ref shadow_roots) = node.shadow_roots {
            for shadow_root in shadow_roots {
                let shadow_simplified = self._create_simplified_tree(shadow_root);
                simplified.children.push(shadow_simplified);
            }
        }

        // Process content document (iframe)
        if let Some(ref content_doc) = node.content_document {
            let doc_simplified = self._create_simplified_tree(content_doc);
            simplified.children.push(doc_simplified);
        }

        simplified
    }

    /// Check if node should be displayed
    fn _should_display_node(&self, node: &EnhancedDOMTreeNode) -> bool {
        // Skip disabled elements
        if let Some(attrs) = node.attributes.get("disabled") {
            if attrs.as_str() == "true" || attrs.as_str() == "disabled" {
                return false;
            }
        }

        // Skip hidden elements
        if let Some(ref snapshot) = node.snapshot_node {
            if let Some(ref styles) = snapshot.computed_styles {
                if let Some(display) = styles.get("display") {
                    if display == "none" {
                        return false;
                    }
                }
                if let Some(visibility) = styles.get("visibility") {
                    if visibility == "hidden" {
                        return false;
                    }
                }
            }
        }

        // Skip script and style tags
        let tag = node.tag_name();
        if matches!(
            tag.as_str(),
            "script" | "style" | "head" | "meta" | "link" | "title"
        ) {
            return false;
        }

        true
    }

    /// Assign interactive indices to clickable elements
    fn _assign_interactive_indices(&mut self, simplified: &mut SimplifiedNode) {
        if !simplified.should_display {
            // Still process children
            for child in &mut simplified.children {
                self._assign_interactive_indices(child);
            }
            return;
        }

        let node = &simplified.original_node;

        // Check if element is interactive/clickable
        let is_clickable = node
            .snapshot_node
            .as_ref()
            .and_then(|s| s.is_clickable)
            .unwrap_or(false)
            || self._is_interactive_element(node);

        if is_clickable {
            let index = self.interactive_counter;
            self.interactive_counter += 1;

            simplified.is_interactive = true;
            simplified.interactive_index = Some(index);

            // Create interacted element
            let interacted = DOMInteractedElement {
                index,
                backend_node_id: Some(node.backend_node_id as u32),
                tag: node.tag_name(),
                text: self._get_element_text(node),
                attributes: node.attributes.clone(),
                selector: None, // TODO: Generate XPath selector
            };

            self.selector_map.insert(index, interacted);
        }

        // Process children
        for child in &mut simplified.children {
            self._assign_interactive_indices(child);
        }
    }

    /// Check if element is interactive
    fn _is_interactive_element(&self, node: &EnhancedDOMTreeNode) -> bool {
        let tag = node.tag_name();
        matches!(
            tag.as_str(),
            "a" | "button" | "input" | "select" | "textarea" | "label"
        ) || node
            .attributes
            .get("role")
            .map(|r| {
                matches!(
                    r.as_str(),
                    "button" | "link" | "menuitem" | "tab" | "option"
                )
            })
            .unwrap_or(false)
    }

    /// Get element text content
    fn _get_element_text(&self, node: &EnhancedDOMTreeNode) -> Option<String> {
        // Try aria-label first
        if let Some(label) = node.attributes.get("aria-label") {
            if !label.is_empty() {
                return Some(label.clone());
            }
        }

        // Try value attribute
        if let Some(value) = node.attributes.get("value") {
            if !value.is_empty() {
                return Some(value.clone());
            }
        }

        // Try placeholder
        if let Some(placeholder) = node.attributes.get("placeholder") {
            if !placeholder.is_empty() {
                return Some(placeholder.clone());
            }
        }

        // Extract text from children (simplified)
        if node.node_type == NodeType::TextNode && !node.node_value.trim().is_empty() {
            return Some(node.node_value.trim().to_string());
        }

        None
    }

    /// Serialize tree to string representation
    pub fn serialize_tree(
        node: &SimplifiedNode,
        include_attributes: &[&str],
        depth: usize,
    ) -> String {
        if !node.should_display {
            return Self::_serialize_children(node, include_attributes, depth);
        }

        let mut formatted_text = Vec::new();
        let depth_str = "\t".repeat(depth);
        let next_depth = depth + 1;

        match node.original_node.node_type {
            NodeType::ElementNode => {
                let tag = node.original_node.tag_name();
                let mut parts = vec![tag.clone()];

                // Add attributes
                let attrs_str =
                    Self::_build_attributes_string(&node.original_node, include_attributes);
                if !attrs_str.is_empty() {
                    parts.push(attrs_str);
                }

                // Add index if interactive
                if let Some(index) = node.interactive_index {
                    parts.push(format!("[{index}]"));
                }

                formatted_text.push(format!("{}{}", depth_str, parts.join(" ")));

                // Process children
                for child in &node.children {
                    let child_text = Self::serialize_tree(child, include_attributes, next_depth);
                    if !child_text.trim().is_empty() {
                        formatted_text.push(child_text);
                    }
                }
            }
            NodeType::TextNode => {
                let text = node.original_node.node_value.trim();
                if !text.is_empty() && text.len() > 1 {
                    formatted_text.push(format!("{depth_str}{text}"));
                }
            }
            _ => {
                // Process children for other node types
                for child in &node.children {
                    let child_text = Self::serialize_tree(child, include_attributes, next_depth);
                    if !child_text.trim().is_empty() {
                        formatted_text.push(child_text);
                    }
                }
            }
        }

        formatted_text.join("\n")
    }

    /// Serialize children only
    fn _serialize_children(
        node: &SimplifiedNode,
        include_attributes: &[&str],
        depth: usize,
    ) -> String {
        let mut parts = Vec::new();
        for child in &node.children {
            let child_text = Self::serialize_tree(child, include_attributes, depth);
            if !child_text.trim().is_empty() {
                parts.push(child_text);
            }
        }
        parts.join("\n")
    }

    /// Build attributes string
    fn _build_attributes_string(node: &EnhancedDOMTreeNode, include_attributes: &[&str]) -> String {
        let mut attrs = Vec::new();

        for attr_name in include_attributes {
            if let Some(value) = node.attributes.get(*attr_name) {
                if !value.is_empty() {
                    attrs.push(format!("{attr_name}=\"{value}\""));
                }
            }
        }

        attrs.join(" ")
    }

    /// Find interacted element for a node (helper)
    fn _find_interacted_element(
        &self,
        node: &EnhancedDOMTreeNode,
    ) -> Option<&DOMInteractedElement> {
        // Look up by backend_node_id in selector_map
        self.selector_map
            .values()
            .find(|elem| elem.backend_node_id == Some(node.backend_node_id as u32))
    }
}

/// Serialized DOM state (temporary - will be updated)
use crate::dom::views::SerializedDOMState;
