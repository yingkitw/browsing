//! Enhanced snapshot processing for browsing DOM tree extraction

use crate::dom::views::{DOMRect, EnhancedSnapshotNode};
use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Required computed styles for interactivity and visibility detection
pub const REQUIRED_COMPUTED_STYLES: &[&str] = &[
    "display",
    "visibility",
    "opacity",
    "overflow",
    "overflow-x",
    "overflow-y",
    "cursor",
    "pointer-events",
    "position",
    "background-color",
];

/// Parse rare boolean data from snapshot
fn parse_rare_boolean_data(rare_data: &Value, index: usize) -> Option<bool> {
    if let Some(indices) = rare_data.get("index").and_then(|v| v.as_array()) {
        return Some(indices.contains(&serde_json::json!(index)));
    }
    None
}

/// Parse computed styles from layout tree using string indices
fn parse_computed_styles(strings: &[String], style_indices: &[usize]) -> HashMap<String, String> {
    let mut styles = HashMap::new();
    for (i, &style_index) in style_indices.iter().enumerate() {
        if i < REQUIRED_COMPUTED_STYLES.len() && style_index < strings.len() {
            styles.insert(
                REQUIRED_COMPUTED_STYLES[i].to_string(),
                strings[style_index].clone(),
            );
        }
    }
    styles
}

/// Build a lookup table of backend node ID to enhanced snapshot data
pub fn build_snapshot_lookup(
    snapshot: &Value,
    device_pixel_ratio: f64,
) -> Result<HashMap<u64, EnhancedSnapshotNode>> {
    let mut snapshot_lookup: HashMap<u64, EnhancedSnapshotNode> = HashMap::new();

    let documents = snapshot
        .get("documents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            crate::error::BrowsingError::Dom("No documents in snapshot".to_string())
        })?;

    if documents.is_empty() {
        return Ok(snapshot_lookup);
    }

    let strings: Vec<String> = snapshot
        .get("strings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    for document in documents {
        let nodes = document.get("nodes").ok_or_else(|| {
            crate::error::BrowsingError::Dom("No nodes in document".to_string())
        })?;
        let layout = document.get("layout");

        // Build backend node id to snapshot index lookup
        let mut backend_node_to_snapshot_index: HashMap<u64, usize> = HashMap::new();
        if let Some(backend_node_ids) = nodes.get("backendNodeId").and_then(|v| v.as_array()) {
            for (i, node_id_val) in backend_node_ids.iter().enumerate() {
                if let Some(node_id) = node_id_val.as_u64() {
                    backend_node_to_snapshot_index.insert(node_id, i);
                }
            }
        }

        // Build layout index map
        let mut layout_index_map: HashMap<usize, usize> = HashMap::new();
        if let Some(layout) = layout {
            if let Some(node_indices) = layout.get("nodeIndex").and_then(|v| v.as_array()) {
                for (layout_idx, node_index_val) in node_indices.iter().enumerate() {
                    if let Some(node_index) = node_index_val.as_u64().map(|v| v as usize) {
                        layout_index_map.entry(node_index).or_insert(layout_idx);
                    }
                }
            }
        }

        // Build snapshot lookup for each backend node id
        for (backend_node_id, snapshot_index) in backend_node_to_snapshot_index {
            let is_clickable = nodes
                .get("isClickable")
                .and_then(|v| parse_rare_boolean_data(v, snapshot_index));

            // Find corresponding layout node
            let mut cursor_style = None;
            let mut bounding_box = None;
            let mut computed_styles = HashMap::new();
            let mut paint_order = None;
            let mut client_rects = None;
            let mut scroll_rects = None;
            let mut stacking_contexts = None;

            if let Some(layout) = layout {
                if let Some(&layout_idx) = layout_index_map.get(&snapshot_index) {
                    // Parse bounding box
                    if let Some(bounds_array) = layout.get("bounds").and_then(|v| v.as_array()) {
                        if layout_idx < bounds_array.len() {
                            if let Some(bounds) = bounds_array[layout_idx].as_array() {
                                if bounds.len() >= 4 {
                                    let raw_x = bounds[0].as_f64().unwrap_or(0.0);
                                    let raw_y = bounds[1].as_f64().unwrap_or(0.0);
                                    let raw_width = bounds[2].as_f64().unwrap_or(0.0);
                                    let raw_height = bounds[3].as_f64().unwrap_or(0.0);

                                    // Convert device pixels to CSS pixels
                                    bounding_box = Some(DOMRect::new(
                                        raw_x / device_pixel_ratio,
                                        raw_y / device_pixel_ratio,
                                        raw_width / device_pixel_ratio,
                                        raw_height / device_pixel_ratio,
                                    ));
                                }
                            }
                        }
                    }

                    // Parse computed styles
                    if let Some(styles_array) = layout.get("styles").and_then(|v| v.as_array()) {
                        if layout_idx < styles_array.len() {
                            if let Some(style_indices) = styles_array[layout_idx].as_array() {
                                let indices: Vec<usize> = style_indices
                                    .iter()
                                    .filter_map(|v| v.as_u64().map(|v| v as usize))
                                    .collect();
                                computed_styles = parse_computed_styles(&strings, &indices);
                                cursor_style = computed_styles.get("cursor").cloned();
                            }
                        }
                    }

                    // Extract paint order
                    if let Some(paint_orders) = layout.get("paintOrders").and_then(|v| v.as_array())
                    {
                        if layout_idx < paint_orders.len() {
                            paint_order = paint_orders[layout_idx].as_i64().map(|v| v as i32);
                        }
                    }

                    // Extract client rects
                    if let Some(client_rects_data) =
                        layout.get("clientRects").and_then(|v| v.as_array())
                    {
                        if layout_idx < client_rects_data.len() {
                            if let Some(rect_array) = client_rects_data[layout_idx].as_array() {
                                if rect_array.len() >= 4 {
                                    let x = rect_array[0].as_f64().unwrap_or(0.0);
                                    let y = rect_array[1].as_f64().unwrap_or(0.0);
                                    let width = rect_array[2].as_f64().unwrap_or(0.0);
                                    let height = rect_array[3].as_f64().unwrap_or(0.0);
                                    client_rects = Some(DOMRect::new(x, y, width, height));
                                }
                            }
                        }
                    }

                    // Extract scroll rects
                    if let Some(scroll_rects_data) =
                        layout.get("scrollRects").and_then(|v| v.as_array())
                    {
                        if layout_idx < scroll_rects_data.len() {
                            if let Some(rect_array) = scroll_rects_data[layout_idx].as_array() {
                                if rect_array.len() >= 4 {
                                    let x = rect_array[0].as_f64().unwrap_or(0.0);
                                    let y = rect_array[1].as_f64().unwrap_or(0.0);
                                    let width = rect_array[2].as_f64().unwrap_or(0.0);
                                    let height = rect_array[3].as_f64().unwrap_or(0.0);
                                    scroll_rects = Some(DOMRect::new(x, y, width, height));
                                }
                            }
                        }
                    }

                    // Extract stacking contexts
                    if let Some(stacking_contexts_obj) = layout.get("stackingContexts") {
                        if let Some(index_array) = stacking_contexts_obj
                            .get("index")
                            .and_then(|v| v.as_array())
                        {
                            if layout_idx < index_array.len() {
                                stacking_contexts =
                                    index_array[layout_idx].as_i64().map(|v| v as i32);
                            }
                        }
                    }
                }
            }

            let computed_styles_opt = if computed_styles.is_empty() {
                None
            } else {
                Some(computed_styles)
            };

            snapshot_lookup.insert(
                backend_node_id,
                EnhancedSnapshotNode {
                    is_clickable,
                    cursor_style,
                    bounds: bounding_box,
                    client_rects,
                    scroll_rects,
                    computed_styles: computed_styles_opt,
                    paint_order,
                    stacking_contexts,
                },
            );
        }
    }

    Ok(snapshot_lookup)
}
