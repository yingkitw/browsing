//! Tool action view types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base model for dynamically created action models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionModel {
    pub action_type: String,
    pub params: HashMap<String, serde_json::Value>,
}

impl ActionModel {
    pub fn get_index(&self) -> Option<u32> {
        // Extract index from params if present
        self.params
            .values()
            .find_map(|v| {
                if let Some(obj) = v.as_object() {
                    obj.get("index")?.as_u64().map(|i| i as u32)
                } else {
                    None
                }
            })
    }

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

/// Model for a registered action
#[derive(Debug, Clone)]
pub struct RegisteredAction {
    pub name: String,
    pub description: String,
    pub domains: Option<Vec<String>>,
    // In Rust, we'll use a function pointer or trait object for the function
    // This is a simplified version - full implementation would use async trait
}

impl RegisteredAction {
    pub fn prompt_description(&self) -> String {
        format!("{}: {}", self.name, self.description)
    }
}

/// Model representing the action registry
#[derive(Debug, Clone, Default)]
pub struct ActionRegistry {
    pub actions: HashMap<String, RegisteredAction>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

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

