//! DOM extraction and serialization tests

use browsing::dom::serializer::SimplifiedNode;
use browsing::dom::service::DomService;
use browsing::dom::views::{
    DOMElement, DOMRect, EnhancedAXNode, EnhancedDOMTreeNode, 
    NodeType, SerializedDOMState
};
use browsing::browser::{Browser, BrowserProfile};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_dom_service_creation() {
    let dom_service = DomService::new();
    
    // Should be created without browser initially - fields are private
}

#[tokio::test]
async fn test_dom_service_configuration() {
    let dom_service = DomService::new()
        .with_browser(std::sync::Arc::new(Browser::new(BrowserProfile::default())))
        .with_cdp_client(
            std::sync::Arc::new(browsing::browser::cdp::CdpClient::new(
                "ws://localhost:9222".to_string()
            )),
            "test-session".to_string()
        )
        .with_target_id("test-target".to_string());
    
    // Should store all configuration - fields are private
    // The configuration is stored internally when using with_browser, with_cdp_client, with_target_id
}

#[test]
fn test_dom_node_creation() {
    let node = EnhancedDOMTreeNode::new(
        1,
        1,
        NodeType::ElementNode,
        "DIV".to_string(),
        "".to_string(),
        "test-target".to_string()
    );
    
    assert_eq!(node.node_id, 1);
    assert_eq!(node.backend_node_id, 1);
    assert_eq!(node.node_type, NodeType::ElementNode);
    assert_eq!(node.node_name, "DIV");
    assert_eq!(node.target_id, "test-target");
}

#[test]
fn test_dom_node_attributes() {
    let mut attrs = HashMap::new();
    attrs.insert("id".to_string(), "test-id".to_string());
    attrs.insert("class".to_string(), "test-class".to_string());
    
    let mut node = EnhancedDOMTreeNode::new(
        1,
        1,
        NodeType::ElementNode,
        "DIV".to_string(),
        "test content".to_string(),
        "test-target".to_string()
    );
    node.attributes = attrs;
    
    assert_eq!(node.attributes.get("id"), Some(&"test-id".to_string()));
    assert_eq!(node.attributes.get("class"), Some(&"test-class".to_string()));
}

#[test]
fn test_simplified_node_creation() {
    let mut attrs = HashMap::new();
    attrs.insert("role".to_string(), "button".to_string());
    
    let mut enhanced_node = EnhancedDOMTreeNode::new(
        1,
        1,
        NodeType::ElementNode,
        "BUTTON".to_string(),
        "Click me".to_string(),
        "test-target".to_string()
    );
    enhanced_node.attributes = attrs;
    
    let simplified = SimplifiedNode::new(enhanced_node);
    
    assert!(simplified.should_display);
    // is_interactive is false by default in SimplifiedNode::new
    // It gets set to true only through DOMTreeSerializer._assign_interactive_indices
    assert!(!simplified.is_interactive);
    assert_eq!(simplified.interactive_index, None);
}

#[test]
fn test_simplified_dom_creation() {
    let root_node = EnhancedDOMTreeNode::new(
        0,
        0,
        NodeType::DocumentNode,
        "#document".to_string(),
        "".to_string(),
        "test-target".to_string()
    );
    
    let simplified_dom = SimplifiedNode::new(root_node);
    
    assert!(simplified_dom.should_display);
    assert!(!simplified_dom.is_interactive);
}

#[test]
fn test_dom_element_serialization() {
    let mut attrs = HashMap::new();
    attrs.insert("id".to_string(), "test-element".to_string());
    
    let element = DOMElement {
        index: 1,
        tag: "div".to_string(),
        text: Some("Test content".to_string()),
        attributes: attrs,
        children: vec![],
    };
    
    // DOMElement doesn't have to_dict method - it's only for DOMInteractedElement
    let json_str = serde_json::to_string(&element).unwrap();
    let dict: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(dict.get("index"), Some(&json!(1)));
    assert_eq!(dict.get("tag"), Some(&json!("div")));
    assert_eq!(dict.get("text"), Some(&json!("Test content")));
}

#[test]
fn test_dom_rect_creation() {
    let rect = DOMRect::new(10.0, 20.0, 100.0, 200.0);
    
    assert_eq!(rect.x, 10.0);
    assert_eq!(rect.y, 20.0);
    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 200.0);
}

#[test]
fn test_ax_node_creation() {
    let mut properties = Vec::new();
    properties.push(browsing::dom::views::EnhancedAXProperty {
        name: "role".to_string(),
        value: Some(json!("button")),
    });
    
    let ax_node = EnhancedAXNode {
        ax_node_id: "ax-1".to_string(),
        ignored: false,
        role: Some("button".to_string()),
        name: Some("Submit".to_string()),
        description: Some("Submit form".to_string()),
        properties: Some(properties),
        child_ids: None,
    };
    
    assert_eq!(ax_node.role, Some("button".to_string()));
    assert_eq!(ax_node.name, Some("Submit".to_string()));
    assert!(!ax_node.ignored);
}

#[test]
fn test_dom_state_serialization() {
    let state = SerializedDOMState {
        html: Some("<html><body>Test</body></html>".to_string()),
        text: Some("Test content".to_string()),
        markdown: Some("# Test".to_string()),
        elements: vec![],
        selector_map: HashMap::new(),
    };
    
    // Should be serializable to JSON
    let json_str = serde_json::to_string(&state);
    assert!(json_str.is_ok());
    
    // Should be deserializable from JSON
    let deserialized: Result<SerializedDOMState, _> = serde_json::from_str(&json_str.unwrap());
    assert!(deserialized.is_ok());
}

#[test]
fn test_node_type_variants() {
    let types = [
        NodeType::ElementNode,
        NodeType::AttributeNode,
        NodeType::TextNode,
        NodeType::CdataSectionNode,
        NodeType::EntityReferenceNode,
        NodeType::EntityNode,
        NodeType::ProcessingInstructionNode,
        NodeType::CommentNode,
        NodeType::DocumentNode,
        NodeType::DocumentTypeNode,
        NodeType::DocumentFragmentNode,
        NodeType::NotationNode,
    ];
    
    // All node types should be creatable
    for node_type in types {
        // This is a basic test - in real scenarios, each type would have specific behavior
        let _node = EnhancedDOMTreeNode::new(
            1,
            1,
            node_type,
            "TEST".to_string(),
            "".to_string(),
            "test-target".to_string()
        );
    }
}

#[tokio::test]
async fn test_dom_extraction_flow() {
    // This tests the logical flow without actual browser interaction
    
    let dom_service = DomService::new();
    
    // In a real scenario, this would:
    // 1. Connect to browser CDP
    // 2. Get DOM document
    // 3. Extract nodes
    // 4. Serialize to simplified format
    
    // For testing, we verify the service is properly configured
    // Fields are private, we just verify it doesn't panic
    
    // Configure with mock browser
    let _configured_service = dom_service
        .with_browser(std::sync::Arc::new(Browser::new(BrowserProfile::default())));
    
    // Service configured successfully - fields are private
}

#[test]
fn test_dom_node_with_children() {
    let mut parent_attrs = HashMap::new();
    parent_attrs.insert("class".to_string(), "parent".to_string());
    
    let mut child_attrs = HashMap::new();
    child_attrs.insert("class".to_string(), "child".to_string());
    
    let child_node = EnhancedDOMTreeNode::new(
        2,
        2,
        NodeType::ElementNode,
        "SPAN".to_string(),
        "Child text".to_string(),
        "test-target".to_string()
    );
    
    let mut parent_node = EnhancedDOMTreeNode::new(
        1,
        1,
        NodeType::ElementNode,
        "DIV".to_string(),
        "".to_string(),
        "test-target".to_string()
    );
    
    parent_node.attributes = parent_attrs;
    parent_node.children_nodes = Some(vec![child_node]);
    
    // Test that parent-child relationship is maintained
    assert!(parent_node.children_nodes.is_some());
    let children = parent_node.children_nodes.as_ref().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].node_name, "SPAN");
}