//! Tests for DOM service functionality

use browser_use::dom::views::{EnhancedDOMTreeNode, NodeType, SerializedDOMState};
use std::collections::HashMap;

#[test]
fn test_enhanced_dom_node_creation() {
    let node = EnhancedDOMTreeNode::new(
        1,
        1,
        NodeType::ElementNode,
        "div".to_string(),
        "".to_string(),
        "target-1".to_string(),
    );

    assert_eq!(node.node_id, 1);
    assert_eq!(node.backend_node_id, 1);
    assert_eq!(node.node_name, "div");
    assert_eq!(node.node_type, NodeType::ElementNode);
}

#[test]
fn test_serialized_dom_state_llm_representation() {
    let state = SerializedDOMState {
        html: Some("<div>test</div>".to_string()),
        text: Some("test".to_string()),
        markdown: Some("# Test".to_string()),
        elements: vec![],
        selector_map: HashMap::new(),
    };

    // Should prefer markdown
    let repr = state.llm_representation(None);
    assert_eq!(repr, Some("# Test".to_string()));
}

#[test]
fn test_serialized_dom_state_fallbacks() {
    // Test text fallback
    let state = SerializedDOMState {
        html: Some("<div>test</div>".to_string()),
        text: Some("test text".to_string()),
        markdown: None,
        elements: vec![],
        selector_map: HashMap::new(),
    };

    let repr = state.llm_representation(None);
    assert_eq!(repr, Some("test text".to_string()));

    // Test HTML fallback
    let state = SerializedDOMState {
        html: Some("<div>test</div>".to_string()),
        text: None,
        markdown: None,
        elements: vec![],
        selector_map: HashMap::new(),
    };

    let repr = state.llm_representation(None);
    assert_eq!(repr, Some("<div>test</div>".to_string()));
}

#[test]
fn test_node_type_variants() {
    assert_eq!(NodeType::ElementNode as u8, 1);
    assert_eq!(NodeType::TextNode as u8, 3);
    assert_eq!(NodeType::DocumentNode as u8, 9);
}
