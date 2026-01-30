//! DOM service for page analysis

use crate::browser::{Browser, cdp::CdpClient};
use crate::dom::enhanced_snapshot::build_snapshot_lookup;
use crate::dom::serializer::DOMTreeSerializer;
use crate::dom::views::{
    DOMRect, EnhancedAXNode, EnhancedAXProperty, EnhancedDOMTreeNode, EnhancedSnapshotNode,
    NodeType, SerializedDOMState,
};
use crate::error::{BrowsingError, Result};
use futures::future::try_join4;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// DOM service for extracting and analyzing page content
pub struct DomService {
    /// Browser instance
    browser: Option<Arc<Browser>>,
    /// CDP client instance
    cdp_client: Option<Arc<CdpClient>>,
    /// Session ID
    session_id: Option<String>,
    /// Current target ID
    current_target_id: Option<String>,
    /// Whether to handle cross-origin iframes
    #[allow(dead_code)]
    cross_origin_iframes: bool,
    /// Whether to use paint order filtering
    #[allow(dead_code)]
    paint_order_filtering: bool,
    /// Maximum number of iframes to handle
    #[allow(dead_code)]
    max_iframes: usize,
    /// Maximum iframe depth to handle
    #[allow(dead_code)]
    max_iframe_depth: usize,
}

impl DomService {
    /// Creates a new DOM service
    pub fn new() -> Self {
        Self {
            browser: None,
            cdp_client: None,
            session_id: None,
            current_target_id: None,
            cross_origin_iframes: false,
            paint_order_filtering: true,
            max_iframes: 100,
            max_iframe_depth: 5,
        }
    }

    /// Sets the browser instance
    pub fn with_browser(mut self, browser: Arc<Browser>) -> Self {
        self.browser = Some(browser);
        // Extract CDP client, session ID, and target ID from browser
        if let Ok(client) = self.browser.as_ref().unwrap().get_cdp_client() {
            self.cdp_client = Some(client);
        }
        if let Ok(sid) = self.browser.as_ref().unwrap().get_session_id() {
            self.session_id = Some(sid);
        }
        if let Ok(target_id) = self.browser.as_ref().unwrap().get_current_target_id() {
            self.current_target_id = Some(target_id);
        }
        self
    }

    /// Sets the CDP client and session ID
    pub fn with_cdp_client(mut self, client: Arc<CdpClient>, session_id: String) -> Self {
        self.cdp_client = Some(client);
        self.session_id = Some(session_id);
        self
    }

    /// Sets the target ID
    pub fn with_target_id(mut self, target_id: String) -> Self {
        self.current_target_id = Some(target_id);
        self
    }

    /// Extract page content from HTML
    pub async fn extract_page_content(&self, html: &str) -> Result<String> {
        // Convert HTML to markdown
        let markdown = self.html_to_markdown(html)?;
        Ok(markdown)
    }

    /// Convert HTML to markdown
    fn html_to_markdown(&self, html: &str) -> Result<String> {
        // Basic HTML to markdown conversion
        // This is a simplified version - full implementation would use a proper HTML parser
        let cleaned_html = self.remove_script_style_tags(html);

        // Use pulldown-cmark to parse markdown (if input is already markdown)
        // For HTML, we'll do basic text extraction for now
        let text = self.extract_text(&cleaned_html);

        // Basic markdown formatting
        let mut markdown = String::new();

        // Split by paragraphs and format
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                markdown.push_str(trimmed);
                markdown.push_str("\n\n");
            }
        }

        Ok(markdown.trim().to_string())
    }

    /// Remove script and style tags from HTML
    fn remove_script_style_tags(&self, html: &str) -> String {
        let script_re = Regex::new(r"(?s)<script[^>]*>.*?</script>").unwrap();
        let style_re = Regex::new(r"(?s)<style[^>]*>.*?</style>").unwrap();

        let cleaned = script_re.replace_all(html, "");
        let cleaned = style_re.replace_all(&cleaned, "");
        cleaned.to_string()
    }

    /// Get all trees (snapshot, DOM tree, AX tree, device pixel ratio) for a target
    async fn _get_all_trees(&self, target_id: &str) -> Result<(Value, Value, Value, f64)> {
        let client = self
            .cdp_client
            .as_ref()
            .ok_or_else(|| BrowsingError::Dom("No CDP client available".to_string()))?;
        let session_id = self.session_id.as_deref();

        // Required computed styles for snapshot
        let required_computed_styles = vec![
            "display",
            "visibility",
            "opacity",
            "overflow",
            "overflow-x",
            "overflow-y",
            "position",
            "z-index",
            "transform",
            "transform-origin",
        ];

        // Create snapshot request
        let snapshot_params = serde_json::json!({
            "computedStyles": required_computed_styles,
            "includePaintOrder": true,
            "includeDOMRects": true,
            "includeBlendedBackgroundColors": false,
            "includeTextColorOpacities": false,
        });

        // Create DOM tree request
        let dom_tree_params = serde_json::json!({
            "depth": -1,
            "pierce": true
        });

        // Create accessibility tree request
        let ax_tree_params = serde_json::json!({});

        // Get viewport ratio
        let _viewport_params = serde_json::json!({});

        // Execute all requests in parallel
        let snapshot_fut = client.send_command_with_session(
            "DOMSnapshot.captureSnapshot",
            snapshot_params,
            session_id,
        );
        let dom_tree_fut =
            client.send_command_with_session("DOM.getDocument", dom_tree_params, session_id);
        let ax_tree_fut = client.send_command_with_session(
            "Accessibility.getFullAXTree",
            ax_tree_params,
            session_id,
        );
        let viewport_fut = self._get_viewport_ratio(target_id);

        // Wait for all with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            try_join4(snapshot_fut, dom_tree_fut, ax_tree_fut, viewport_fut),
        )
        .await
        .map_err(|_| BrowsingError::Dom("Timeout waiting for CDP responses".to_string()))??;

        let (snapshot_result, dom_tree_result, ax_tree_result, device_pixel_ratio) = result;

        Ok((
            snapshot_result,
            dom_tree_result,
            ax_tree_result,
            device_pixel_ratio,
        ))
    }

    /// Get viewport ratio (device pixel ratio)
    async fn _get_viewport_ratio(&self, _target_id: &str) -> Result<f64> {
        let client = self
            .cdp_client
            .as_ref()
            .ok_or_else(|| BrowsingError::Dom("No CDP client available".to_string()))?;
        let session_id = self.session_id.as_deref();

        // Get layout metrics
        let metrics = client
            .send_command_with_session("Page.getLayoutMetrics", serde_json::json!({}), session_id)
            .await?;

        // Extract device pixel ratio
        if let Some(visual_viewport) = metrics.get("visualViewport") {
            if let Some(css_visual_viewport) = metrics.get("cssVisualViewport") {
                let device_width = visual_viewport
                    .get("clientWidth")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1920.0);
                let css_width = css_visual_viewport
                    .get("clientWidth")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1920.0);

                if css_width > 0.0 {
                    return Ok(device_width / css_width);
                }
            }
        }

        // Fallback to default
        Ok(1.0)
    }

    /// Build enhanced AX node from CDP AX node data
    fn build_enhanced_ax_node(&self, ax_node: &Value) -> Option<EnhancedAXNode> {
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

        let properties_opt: Option<Vec<EnhancedAXProperty>> = properties
            .and_then(|p: Vec<EnhancedAXProperty>| if p.is_empty() { None } else { Some(p) });
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

    /// Get DOM tree for the current target
    pub async fn get_dom_tree(&self, target_id: Option<&str>) -> Result<EnhancedDOMTreeNode> {
        // Get target ID from parameter, stored value, or browser
        let target = if let Some(tid) = target_id {
            tid.to_string()
        } else if let Some(ref tid) = self.current_target_id {
            tid.clone()
        } else if let Some(ref browser) = self.browser {
            browser.get_current_target_id()?
        } else {
            return Err(BrowsingError::Dom(
                "Target ID required for DOM tree extraction".to_string(),
            ));
        };

        self.get_dom_tree_by_target(&target).await
    }

    /// Get DOM tree for a specific target ID
    async fn get_dom_tree_by_target(&self, target_id: &str) -> Result<EnhancedDOMTreeNode> {
        let (snapshot, dom_tree, ax_tree, device_pixel_ratio) =
            self._get_all_trees(target_id).await?;

        // Build AX tree lookup
        let mut ax_tree_lookup: HashMap<u64, Value> = HashMap::new();
        if let Some(nodes) = ax_tree.get("nodes").and_then(|v| v.as_array()) {
            for node in nodes {
                if let Some(backend_node_id) = node.get("backendDOMNodeId").and_then(|v| v.as_u64())
                {
                    ax_tree_lookup.insert(backend_node_id, node.clone());
                }
            }
        }

        // Build snapshot lookup
        let snapshot_lookup = build_snapshot_lookup(&snapshot, device_pixel_ratio)?;

        // Build enhanced DOM tree node lookup (memoization)
        let mut enhanced_dom_tree_node_lookup: HashMap<u64, EnhancedDOMTreeNode> = HashMap::new();

        // Get root node from DOM tree
        let root_node = dom_tree
            .get("root")
            .ok_or_else(|| BrowsingError::Dom("No root node in DOM tree".to_string()))?;

        // Recursively construct enhanced nodes
        let enhanced_root = self._construct_enhanced_node(
            root_node,
            &ax_tree_lookup,
            &snapshot_lookup,
            target_id,
            &mut enhanced_dom_tree_node_lookup,
            None::<&mut Vec<*const EnhancedDOMTreeNode>>,
            None,
        )?;

        Ok(enhanced_root)
    }

    /// Recursively construct enhanced DOM tree nodes
    fn _construct_enhanced_node(
        &self,
        node: &Value,
        ax_tree_lookup: &HashMap<u64, Value>,
        snapshot_lookup: &HashMap<u64, EnhancedSnapshotNode>,
        target_id: &str,
        node_lookup: &mut HashMap<u64, EnhancedDOMTreeNode>,
        _html_frames: Option<&mut Vec<*const EnhancedDOMTreeNode>>,
        total_frame_offset: Option<DOMRect>,
    ) -> Result<EnhancedDOMTreeNode> {
        let node_id = node
            .get("nodeId")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowsingError::Dom("No nodeId in node".to_string()))?;

        // Check memoization
        if let Some(existing) = node_lookup.get(&node_id) {
            return Ok(existing.clone());
        }

        let backend_node_id = node
            .get("backendNodeId")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| BrowsingError::Dom("No backendNodeId in node".to_string()))?;

        // Get AX node
        let ax_node = ax_tree_lookup
            .get(&backend_node_id)
            .and_then(|ax| self.build_enhanced_ax_node(ax));

        // Parse attributes
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

        // Get node type
        let node_type_val = node.get("nodeType").and_then(|v| v.as_u64()).unwrap_or(1);
        let node_type = match node_type_val {
            1 => NodeType::ElementNode,
            2 => NodeType::AttributeNode,
            3 => NodeType::TextNode,
            4 => NodeType::CdataSectionNode,
            5 => NodeType::EntityReferenceNode,
            6 => NodeType::EntityNode,
            7 => NodeType::ProcessingInstructionNode,
            8 => NodeType::CommentNode,
            9 => NodeType::DocumentNode,
            10 => NodeType::DocumentTypeNode,
            11 => NodeType::DocumentFragmentNode,
            12 => NodeType::NotationNode,
            _ => NodeType::ElementNode,
        };

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

        // Get snapshot data
        let snapshot_data = snapshot_lookup.get(&backend_node_id).cloned();

        // Calculate absolute position
        let absolute_position =
            if let (Some(snapshot), Some(offset)) = (snapshot_data.as_ref(), total_frame_offset) {
                snapshot.bounds.map(|bounds| {
                    DOMRect::new(
                        bounds.x + offset.x,
                        bounds.y + offset.y,
                        bounds.width,
                        bounds.height,
                    )
                })
            } else {
                snapshot_data.as_ref().and_then(|s| s.bounds)
            };

        // Create enhanced node
        let mut enhanced_node = EnhancedDOMTreeNode::new(
            node_id,
            backend_node_id,
            node_type,
            node_name.clone(),
            node_value,
            target_id.to_string(),
        );

        enhanced_node.attributes = attributes;
        enhanced_node.ax_node = ax_node;
        enhanced_node.snapshot_node = snapshot_data;
        enhanced_node.absolute_position = absolute_position;
        enhanced_node.frame_id = node
            .get("frameId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        enhanced_node.is_scrollable = node.get("isScrollable").and_then(|v| v.as_bool());
        enhanced_node.shadow_root_type = node
            .get("shadowRootType")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        enhanced_node.session_id = self.session_id.clone();

        // Store in lookup before processing children (to handle circular references)
        node_lookup.insert(node_id, enhanced_node.clone());

        // Process children
        if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
            let mut children_nodes = Vec::new();
            for child in children {
                let child_node = self._construct_enhanced_node(
                    child,
                    ax_tree_lookup,
                    snapshot_lookup,
                    target_id,
                    node_lookup,
                    None,
                    total_frame_offset,
                )?;
                children_nodes.push(child_node);
            }
            enhanced_node.children_nodes = Some(children_nodes);
        }

        // Process content document (iframe)
        if let Some(content_doc) = node.get("contentDocument") {
            let content_doc_node = self._construct_enhanced_node(
                content_doc,
                ax_tree_lookup,
                snapshot_lookup,
                target_id,
                node_lookup,
                None,
                total_frame_offset,
            )?;
            enhanced_node.content_document = Some(Box::new(content_doc_node));
        }

        // Process shadow roots
        if let Some(shadow_roots) = node.get("shadowRoots").and_then(|v| v.as_array()) {
            let mut shadow_root_nodes = Vec::new();
            for shadow_root in shadow_roots {
                let shadow_node = self._construct_enhanced_node(
                    shadow_root,
                    ax_tree_lookup,
                    snapshot_lookup,
                    target_id,
                    node_lookup,
                    None,
                    total_frame_offset,
                )?;
                shadow_root_nodes.push(shadow_node);
            }
            enhanced_node.shadow_roots = Some(shadow_root_nodes);
        }

        // Update lookup with final node
        node_lookup.insert(node_id, enhanced_node.clone());

        Ok(enhanced_node)
    }

    /// Get serialized DOM tree representation for LLM consumption
    pub async fn get_serialized_dom_tree(
        &self,
        target_id: Option<&str>,
    ) -> Result<(
        SerializedDOMState,
        EnhancedDOMTreeNode,
        HashMap<String, f64>,
    )> {
        // Get enhanced DOM tree
        let enhanced_dom_tree = self.get_dom_tree(target_id).await?;

        // Serialize the tree
        let serializer = DOMTreeSerializer::new(enhanced_dom_tree.clone());
        let (serialized_state, timing_info) = serializer.serialize_accessible_elements();

        Ok((serialized_state, enhanced_dom_tree, timing_info))
    }

    /// Get serialized DOM state from browser
    pub async fn get_serialized_dom_state(&self) -> Result<SerializedDOMState> {
        let (serialized_state, _, _) = self.get_serialized_dom_tree(None).await?;
        Ok(serialized_state)
    }

    /// Get page state as string for LLM consumption
    pub async fn get_page_state_string(&self) -> Result<String> {
        let (serialized_state, _, _) = self.get_serialized_dom_tree(None).await?;
        Ok(serialized_state
            .llm_representation(None)
            .unwrap_or_else(|| "Empty DOM tree".to_string()))
    }

    /// Get selector map (index -> element mapping)
    pub async fn get_selector_map(
        &self,
    ) -> Result<std::collections::HashMap<u32, crate::dom::views::DOMInteractedElement>> {
        let (serialized_state, _, _) = self.get_serialized_dom_tree(None).await?;
        Ok(serialized_state.selector_map)
    }

    /// Extract text content from HTML
    pub fn extract_text(&self, html: &str) -> String {
        // Basic text extraction - remove HTML tags
        let tag_re = Regex::new(r"<[^>]+>").unwrap();
        let text = tag_re.replace_all(html, " ");

        // Clean up whitespace
        let whitespace_re = Regex::new(r"\s+").unwrap();
        let cleaned = whitespace_re.replace_all(&text, " ");
        cleaned.trim().to_string()
    }
}

impl Default for DomService {
    fn default() -> Self {
        Self::new()
    }
}
