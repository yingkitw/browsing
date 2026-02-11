//! Accessibility (AX) node building from CDP data

use crate::dom::views::{EnhancedAXNode, EnhancedAXProperty};
use serde_json::Value;

/// Build enhanced AX node from CDP AX node data
pub fn build_enhanced_ax_node(ax_node: &Value) -> Option<EnhancedAXNode> {
    let ax_node_id = ax_node.get("nodeId")?.as_str()?.to_string();
    let ignored = ax_node.get("ignored")?.as_bool().unwrap_or(false);

    let role = ax_node
        .get("role")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let name = ax_node
        .get("name")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let description = ax_node
        .get("description")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let properties = ax_node
        .get("properties")
        .and_then(|v| v.as_array())
        .map(|props| {
            props
                .iter()
                .filter_map(|prop| {
                    let name = prop.get("name")?.as_str()?.to_string();
                    let value = prop.get("value").and_then(|v| v.get("value")).cloned();
                    Some(EnhancedAXProperty { name, value })
                })
                .collect()
        });

    let child_ids = ax_node
        .get("childIds")
        .and_then(|v| v.as_array())
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(|s| s.to_string()))
                .collect()
        });

    let properties_opt: Option<Vec<EnhancedAXProperty>> =
        properties.and_then(|p: Vec<EnhancedAXProperty>| if p.is_empty() { None } else { Some(p) });
    let child_ids_opt: Option<Vec<String>> =
        child_ids.and_then(|c: Vec<String>| if c.is_empty() { None } else { Some(c) });

    Some(EnhancedAXNode {
        ax_node_id,
        ignored,
        role,
        name,
        description,
        properties: properties_opt,
        child_ids: child_ids_opt,
    })
}
