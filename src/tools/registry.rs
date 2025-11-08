//! Action registry implementation

use crate::tools::views::{ActionRegistry, RegisteredAction};
use std::collections::HashMap;

/// Registry service for managing actions
pub struct Registry {
    pub registry: ActionRegistry,
    pub exclude_actions: Vec<String>,
}

impl Registry {
    pub fn new(exclude_actions: Vec<String>) -> Self {
        Self {
            registry: ActionRegistry::new(),
            exclude_actions,
        }
    }

    pub fn register_action(
        &mut self,
        name: String,
        description: String,
        domains: Option<Vec<String>>,
    ) {
        if self.exclude_actions.contains(&name) {
            return;
        }

        let action = RegisteredAction {
            name: name.clone(),
            description,
            domains,
        };
        self.registry.actions.insert(name, action);
    }
}

