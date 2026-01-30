//! Tool action view types

use crate::traits::BrowserClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base model for dynamically created action models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionModel {
    /// Type of action
    pub action_type: String,
    /// Parameters for the action
    pub params: HashMap<String, serde_json::Value>,
}

impl ActionModel {
    /// Gets the index from action parameters
    pub fn get_index(&self) -> Option<u32> {
        // Extract index from params if present
        self.params.values().find_map(|v| {
            if let Some(obj) = v.as_object() {
                obj.get("index")?.as_u64().map(|i| i as u32)
            } else {
                None
            }
        })
    }

    /// Sets the index in action parameters
    pub fn set_index(&mut self, index: u32) {
        // Set index in the first param object that has an index field
        for value in self.params.values_mut() {
            if let Some(obj) = value.as_object_mut() {
                if obj.contains_key("index") {
                    obj.insert("index".to_string(), serde_json::Value::Number(index.into()));
                    return;
                }
            }
        }
    }
}

/// Trait for custom action handlers
#[async_trait::async_trait]
pub trait ActionHandler: Send + Sync {
    /// Executes the action with context
    async fn execute(
        &self,
        params: &ActionParams,
        context: &mut ActionContext<'_>,
    ) -> crate::error::Result<crate::agent::views::ActionResult>;
}

/// Context provided to action handlers during execution
pub struct ActionContext<'a> {
    /// Reference to the browser client
    pub browser: &'a mut dyn BrowserClient,
    /// Optional selector map for element resolution
    pub selector_map: Option<&'a HashMap<u32, crate::dom::views::DOMInteractedElement>>,
}

/// Action parameters wrapper with helper methods for parameter extraction
pub struct ActionParams<'a> {
    params: &'a HashMap<String, serde_json::Value>,
    action_type: Option<String>,
}

impl<'a> ActionParams<'a> {
    /// Create new ActionParams from a HashMap
    pub fn new(params: &'a HashMap<String, serde_json::Value>) -> Self {
        Self {
            params,
            action_type: None,
        }
    }

    /// Set the action type
    pub fn with_action_type(mut self, action_type: String) -> Self {
        self.action_type = Some(action_type);
        self
    }

    /// Get the action type
    pub fn get_action_type(&self) -> Option<&str> {
        self.action_type.as_deref()
    }

    /// Get a required parameter as u32
    pub fn get_required_u32(&self, key: &str) -> crate::error::Result<u32> {
        self.params
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|i| i as u32)
            .ok_or_else(|| {
                crate::error::BrowsingError::Tool(format!("Missing '{}' parameter", key))
            })
    }

    /// Get a required parameter as string
    pub fn get_required_str(&self, key: &str) -> crate::error::Result<&str> {
        self.params
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::BrowsingError::Tool(format!("Missing '{}' parameter", key))
            })
    }

    /// Get an optional parameter as bool
    pub fn get_optional_bool(&self, key: &str) -> bool {
        self.params
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get an optional parameter as f64
    pub fn get_optional_f64(&self, key: &str) -> Option<f64> {
        self.params.get(key)?.as_f64()
    }

    /// Get an optional parameter as u64
    pub fn get_optional_u64(&self, key: &str) -> Option<u64> {
        self.params.get(key)?.as_u64()
    }

    /// Get the raw parameters map
    pub fn inner(&self) -> &HashMap<String, serde_json::Value> {
        self.params
    }

    /// Resolve backend node ID from index using selector map
    pub fn backend_node_id_from_index(
        &self,
        index: u32,
        selector_map: Option<&HashMap<u32, crate::dom::views::DOMInteractedElement>>,
    ) -> u32 {
        if let Some(map) = selector_map {
            if let Some(element) = map.get(&index) {
                return element.backend_node_id.unwrap_or(index);
            }
        }
        index
    }
}

/// Model for a registered action
#[derive(Clone)]
pub struct RegisteredAction {
    /// Name of the action
    pub name: String,
    /// Description of the action
    pub description: String,
    /// Domains where this action can be used
    pub domains: Option<Vec<String>>,
    /// Handler for the action
    pub handler: Option<std::sync::Arc<dyn ActionHandler>>,
}

// Manual Debug implementation since we can't derive it due to trait object
impl std::fmt::Debug for RegisteredAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredAction")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("domains", &self.domains)
            .field(
                "handler",
                &if self.handler.is_some() {
                    "Some(handler)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

impl RegisteredAction {
    /// Gets the description for use in prompts
    pub fn prompt_description(&self) -> String {
        format!("{}: {}", self.name, self.description)
    }
}

/// Model representing the action registry
#[derive(Debug, Clone, Default)]
pub struct ActionRegistry {
    /// Registered actions
    pub actions: HashMap<String, RegisteredAction>,
}

impl ActionRegistry {
    /// Creates a new action registry
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Checks if URL matches any of the domains
    pub fn _match_domains(domains: &Option<Vec<String>>, url: &str) -> bool {
        if domains.is_none() || url.is_empty() {
            return true;
        }

        let domains = domains.as_ref().unwrap();
        for domain_pattern in domains {
            if crate::utils::match_url_with_domain_pattern(url, domain_pattern) {
                return true;
            }
        }
        false
    }

    /// Gets the description for use in prompts
    pub fn get_prompt_description(&self, page_url: Option<&str>) -> String {
        if page_url.is_none() {
            // For system prompt, include only actions with no filters
            return self
                .actions
                .values()
                .filter(|action| action.domains.is_none())
                .map(|action| action.prompt_description())
                .collect::<Vec<_>>()
                .join("\n");
        }

        let page_url = page_url.unwrap();
        // Only include filtered actions for the current page URL
        self.actions
            .values()
            .filter(|action| {
                if action.domains.is_none() {
                    return false; // Skip actions with no filters
                }
                Self::_match_domains(&action.domains, page_url)
            })
            .map(|action| action.prompt_description())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
