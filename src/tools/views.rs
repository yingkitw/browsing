//! Tool action view types

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
    /// Executes the action
    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
        browser: &mut crate::browser::Browser,
    ) -> crate::error::Result<crate::agent::views::ActionResult>;
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
