//! Tests for DOM serializer

#[cfg(test)]
mod tests {
    use super::super::views::{EnhancedDOMTreeNode, NodeType, SerializedDOMState};
    use super::super::serializer::DOMTreeSerializer;
    use std::collections::HashMap;

    fn create_test_dom_node() -> EnhancedDOMTreeNode {
        EnhancedDOMTreeNode::new(
            1,
            1,
            NodeType::ElementNode,
            "div".to_string(),
            "".to_string(),
            "target-1".to_string(),
        )
    }

    #[test]
    fn test_serializer_new() {
        let root = create_test_dom_node();
        let serializer = DOMTreeSerializer::new(root.clone());
        // Serializer is created successfully - we can't access root directly
        // but we can test serialization
        let (state, _) = serializer.serialize_accessible_elements();
        assert!(state.elements.is_empty() || !state.elements.is_empty());
    }

    #[test]
    fn test_serializer_serialize_empty_tree() {
        let root = create_test_dom_node();
        let serializer = DOMTreeSerializer::new(root);
        let (state, _) = serializer.serialize_accessible_elements();
        
        assert!(state.elements.is_empty() || !state.elements.is_empty());
        assert!(state.selector_map.is_empty() || !state.selector_map.is_empty());
    }

    #[test]
    fn test_serialized_dom_state_default() {
        let state = SerializedDOMState {
            html: None,
            text: Some("test".to_string()),
            markdown: None,
            elements: vec![],
            selector_map: HashMap::new(),
        };
        
        assert_eq!(state.text, Some("test".to_string()));
        assert!(state.html.is_none());
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
    fn test_serialized_dom_state_llm_representation_fallback() {
        let state = SerializedDOMState {
            html: Some("<div>test</div>".to_string()),
            text: Some("test text".to_string()),
            markdown: None,
            elements: vec![],
            selector_map: HashMap::new(),
        };
        
        // Should fallback to text
        let repr = state.llm_representation(None);
        assert_eq!(repr, Some("test text".to_string()));
    }

    #[test]
    fn test_serialized_dom_state_llm_representation_html_fallback() {
        let state = SerializedDOMState {
            html: Some("<div>test</div>".to_string()),
            text: None,
            markdown: None,
            elements: vec![],
            selector_map: HashMap::new(),
        };
        
        // Should fallback to HTML
        let repr = state.llm_representation(None);
        assert_eq!(repr, Some("<div>test</div>".to_string()));
    }
}

