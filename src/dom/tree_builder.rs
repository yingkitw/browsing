//! DOM tree construction
//!
//! This module handles the construction of enhanced DOM trees from CDP data.

use crate::dom::ax_node::build_enhanced_ax_node;
use crate::dom::cdp_client::DOMCDPClient;
use crate::dom::enhanced_snapshot::build_snapshot_lookup;
use crate::dom::views::{
    EnhancedAXNode, EnhancedDOMTreeNode, EnhancedSnapshotNode, NodeType,
};
use crate::error::{BrowsingError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Builder for enhanced DOM trees
pub struct DOMTreeBuilder {
    cdp_client: Arc<DOMCDPClient>,
    current_target_id: Option<String>,
}

impl DOMTreeBuilder {
    /// Create a new DOM tree builder
    pub fn new(cdp_client: Arc<DOMCDPClient>, current_target_id: Option<String>) -> Self {
        Self {
            cdp_client,
            current_target_id,
        }
    }

    /// Build enhanced DOM tree for the current target
    pub async fn build_tree(&self) -> Result<EnhancedDOMTreeNode> {
        let target = self.current_target_id.clone().ok_or_else(|| {
            BrowsingError::Dom("Target ID required for DOM tree extraction".to_string())
        })?;

        self.build_tree_by_target(&target).await
    }

    /// Build enhanced DOM tree for a specific target ID
    async fn build_tree_by_target(&self, target_id: &str) -> Result<EnhancedDOMTreeNode> {
        let (snapshot, dom_tree, ax_tree, _device_pixel_ratio) =
            self.cdp_client.get_all_trees(target_id).await?;

        // Build AX tree lookup
        let mut ax_tree_lookup: HashMap<u64, Value> = HashMap::new();
        if let Some(nodes) = ax_tree.get("nodes").and_then(|v| v.as_array()) {
            for node in nodes {
                if let Some(backend_node_id) =
                    node.get("backendDOMNodeId").and_then(|v| v.as_u64())
                {
                    ax_tree_lookup.insert(backend_node_id, node.clone());
                }
            }
        }

        // Build snapshot lookup
        let snapshot_lookup = build_snapshot_lookup(&snapshot, 1.0)?;

        // Build enhanced DOM tree node lookup (memoization)
        let mut enhanced_dom_tree_node_lookup: HashMap<u64, EnhancedDOMTreeNode> =
            HashMap::new();

        // Get root node from DOM tree
        let root_node = dom_tree
            .get("root")
            .ok_or_else(|| BrowsingError::Dom("No root node in DOM tree".to_string()))?;

        // Recursively construct enhanced nodes
        let context = BuildContext {
            ax_tree_lookup,
            snapshot_lookup,
            target_id: target_id.to_string(),
        };

        let enhanced_root = self.construct_enhanced_node(
            root_node,
            &context,
            &mut enhanced_dom_tree_node_lookup,
        )?;

        Ok(enhanced_root)
    }

    /// Recursively construct enhanced DOM tree nodes
    fn construct_enhanced_node(
        &self,
        node: &Value,
        context: &BuildContext,
        node_lookup: &mut HashMap<u64, EnhancedDOMTreeNode>,
    ) -> Result<EnhancedDOMTreeNode> {
        let (node_id, backend_node_id) = self.extract_node_ids(node)?;

        // Check memoization
        if let Some(existing) = node_lookup.get(&node_id) {
            return Ok(existing.clone());
        }

        let attributes = self.parse_attributes(node);
        let node_type = self.get_node_type(node);
        let (node_name, node_value) = self.get_node_basic_info(node);

        // Get AX node and snapshot data
        let ax_node = context
            .ax_tree_lookup
            .get(&backend_node_id)
            .and_then(|ax| build_enhanced_ax_node(ax));
        let snapshot_data = context.snapshot_lookup.get(&backend_node_id).cloned();

        // Create and configure enhanced node
        let mut enhanced_node = self.build_enhanced_node(
            node_id,
            backend_node_id,
            node_type,
            node_name,
            node_value,
            context,
            attributes,
            ax_node,
            snapshot_data,
            node,
        )?;

        // Store in lookup before processing children (to handle circular references)
        node_lookup.insert(node_id, enhanced_node.clone());

        // Process children
        enhanced_node.children_nodes = self.process_children(node, context, node_lookup)?;

        // Update lookup with final node
        node_lookup.insert(node_id, enhanced_node.clone());

        Ok(enhanced_node)
    }

    /// Extract node IDs from CDP node data
    fn extract_node_ids(&self, node: &Value) -> Result<(u64, u64)> {
        let node_id = node
            .get("nodeId")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowsingError::Dom("No nodeId in node".to_string()))?;

        let backend_node_id = node
            .get("backendNodeId")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowsingError::Dom("No backendNodeId in node".to_string()))?;

        Ok((node_id, backend_node_id))
    }

    /// Parse attributes from CDP node data
    fn parse_attributes(&self, node: &Value) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        if let Some(attrs) = node.get("attributes").and_then(|v| v.as_array()) {
            for chunk in attrs.chunks(2) {
                if chunk.len() == 2 {
                    if let (Some(key), Some(val)) = (chunk[0].as_str(), chunk[1].as_str()) {
                        attributes.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
        attributes
    }

    /// Get node type from CDP node data
    fn get_node_type(&self, node: &Value) -> NodeType {
        let node_type_val = node.get("nodeType").and_then(|v| v.as_u64()).unwrap_or(1);
        match node_type_val {
            1 => NodeType::ElementNode,
            2 => NodeType::AttributeNode,
            3 => NodeType::TextNode,
            9 => NodeType::DocumentNode,
            _ => NodeType::ElementNode,
        }
    }

    /// Get basic node info (name and value)
    fn get_node_basic_info(&self, node: &Value) -> (String, String) {
        let node_name = node
            .get("nodeName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let node_value = node
            .get("nodeValue")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        (node_name, node_value)
    }

    /// Build enhanced node with all properties
    fn build_enhanced_node(
        &self,
        node_id: u64,
        backend_node_id: u64,
        node_type: NodeType,
        node_name: String,
        node_value: String,
        context: &BuildContext,
        attributes: HashMap<String, String>,
        ax_node: Option<EnhancedAXNode>,
        snapshot_data: Option<EnhancedSnapshotNode>,
        node: &Value,
    ) -> Result<EnhancedDOMTreeNode> {
        let mut enhanced_node = EnhancedDOMTreeNode::new(
            node_id,
            backend_node_id,
            node_type,
            node_name,
            node_value,
            context.target_id.clone(),
        );

        enhanced_node.attributes = attributes;
        enhanced_node.ax_node = ax_node;
        enhanced_node.snapshot_node = snapshot_data;
        enhanced_node.frame_id = node
            .get("frameId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        enhanced_node.is_scrollable = node.get("isScrollable").and_then(|v| v.as_bool());
        enhanced_node.shadow_root_type = node
            .get("shadowRootType")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        enhanced_node.session_id = self.cdp_client.session_id().map(|s| s.to_string());

        Ok(enhanced_node)
    }

    /// Process children recursively
    fn process_children(
        &self,
        node: &Value,
        context: &BuildContext,
        node_lookup: &mut HashMap<u64, EnhancedDOMTreeNode>,
    ) -> Result<Option<Vec<EnhancedDOMTreeNode>>> {
        if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
            let mut children_nodes = Vec::new();
            for child in children {
                let child_node = self.construct_enhanced_node(child, context, node_lookup)?;
                children_nodes.push(child_node);
            }
            Ok(Some(children_nodes))
        } else {
            Ok(None)
        }
    }
}

/// Context for building DOM trees
struct BuildContext {
    ax_tree_lookup: HashMap<u64, Value>,
    snapshot_lookup: HashMap<u64, EnhancedSnapshotNode>,
    target_id: String,
}
