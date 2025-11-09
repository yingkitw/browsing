//! Action registry implementation

use crate::tools::views::{ActionRegistry, RegisteredAction, ActionHandler};
use std::sync::Arc;

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
            handler: None,
        };
        self.registry.actions.insert(name, action);
    }

    /// Register a custom action with a handler
    pub fn register_custom_action<H: ActionHandler + 'static>(
        &mut self,
        name: String,
        description: String,
        domains: Option<Vec<String>>,
        handler: H,
    ) {
        if self.exclude_actions.contains(&name) {
            return;
        }

        let action = RegisteredAction {
            name: name.clone(),
            description,
            domains,
            handler: Some(Arc::new(handler)),
        };
        self.registry.actions.insert(name, action);
    }

    /// Check if an action has a custom handler
    pub fn has_custom_handler(&self, name: &str) -> bool {
        self.registry
            .actions
            .get(name)
            .and_then(|a| a.handler.as_ref())
            .is_some()
    }

    /// Get the custom handler for an action
    pub fn get_handler(&self, name: &str) -> Option<Arc<dyn ActionHandler>> {
        self.registry
            .actions
            .get(name)
            .and_then(|a| a.handler.clone())
    }
}

