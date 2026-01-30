//! DOM processor - orchestration layer
//!
//! This module provides the main DOMProcessor implementation.

use super::cdp_client::DOMCDPClient;
use super::html_converter::HTMLConverter;
use super::tree_builder::DOMTreeBuilder;
use super::views::SerializedDOMState;
use crate::browser::cdp::CdpClient;
use crate::dom::serializer::DOMTreeSerializer;
use crate::dom::views::DOMInteractedElement;
use crate::error::Result;
use crate::traits::DOMProcessor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// DOM processor implementation
pub struct DOMProcessorImpl {
    cdp_client: Option<Arc<DOMCDPClient>>,
    current_target_id: Option<String>,
}

impl DOMProcessorImpl {
    /// Create a new DOM processor
    pub fn new() -> Self {
        Self {
            cdp_client: None,
            current_target_id: None,
        }
    }

    /// Set the CDP client
    pub fn with_cdp_client(mut self, client: Arc<CdpClient>, session_id: String) -> Self {
        self.cdp_client = Some(Arc::new(DOMCDPClient::new(client.clone(), Some(session_id))));
        self
    }

    /// Set the target ID
    pub fn with_target_id(mut self, target_id: String) -> Self {
        self.current_target_id = Some(target_id);
        self
    }

    /// Extract page content from HTML
    pub fn extract_page_content(&self, html: &str) -> Result<String> {
        HTMLConverter::extract_page_content(html)
    }
}

impl Default for DOMProcessorImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DOMProcessor for DOMProcessorImpl {
    async fn get_serialized_dom(&self) -> Result<SerializedDOMState> {
        let cdp_client = self
            .cdp_client
            .as_ref()
            .ok_or_else(|| crate::error::BrowsingError::Dom("No CDP client available".to_string()))?;

        let tree_builder =
            DOMTreeBuilder::new(Arc::clone(cdp_client), self.current_target_id.clone());
        let enhanced_dom_tree = tree_builder.build_tree().await?;

        // Serialize the tree
        let serializer = DOMTreeSerializer::new(enhanced_dom_tree.clone());
        let (serialized_state, _timing_info) = serializer.serialize_accessible_elements();

        Ok(serialized_state)
    }

    async fn get_page_state_string(&self) -> Result<String> {
        let (serialized_state, _, _) = self.get_serialized_dom_tree_internal(None).await?;
        Ok(serialized_state
            .llm_representation(None)
            .unwrap_or_else(|| "Empty DOM tree".to_string()))
    }

    async fn get_selector_map(&self) -> Result<HashMap<u32, DOMInteractedElement>> {
        let (serialized_state, _, _) = self.get_serialized_dom_tree_internal(None).await?;
        Ok(serialized_state.selector_map)
    }
}

impl DOMProcessorImpl {
    /// Get serialized DOM tree with timing info (internal method)
    async fn get_serialized_dom_tree_internal(
        &self,
        target_id: Option<&str>,
    ) -> Result<(SerializedDOMState, crate::dom::views::EnhancedDOMTreeNode, HashMap<String, f64>)> {
        let cdp_client = self
            .cdp_client
            .as_ref()
            .ok_or_else(|| crate::error::BrowsingError::Dom("No CDP client available".to_string()))?;

        let tree_builder = DOMTreeBuilder::new(
            Arc::clone(cdp_client),
            target_id.or(self.current_target_id.as_deref()).map(|s| s.to_string()),
        );
        let enhanced_dom_tree = tree_builder.build_tree().await?;

        // Serialize the tree
        let serializer = crate::dom::serializer::DOMTreeSerializer::new(enhanced_dom_tree.clone());
        let (serialized_state, timing_info) = serializer.serialize_accessible_elements();

        Ok((serialized_state, enhanced_dom_tree, timing_info))
    }

    /// Get serialized DOM tree (public method for backward compatibility)
    pub async fn get_serialized_dom_tree(
        &self,
        target_id: Option<&str>,
    ) -> Result<(SerializedDOMState, crate::dom::views::EnhancedDOMTreeNode, HashMap<String, f64>)> {
        self.get_serialized_dom_tree_internal(target_id).await
    }
}
