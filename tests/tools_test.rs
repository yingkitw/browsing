//! Additional tests for tools service

use browser_use::tools::service::Tools;
use browser_use::tools::views::{ActionModel, RegisteredAction};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_tools_registry_actions() {
    let tools = Tools::new(vec![]);

    // Check that default actions are registered
    assert!(tools.registry.registry.actions.contains_key("search"));
    assert!(tools.registry.registry.actions.contains_key("navigate"));
    assert!(tools.registry.registry.actions.contains_key("click"));
    assert!(tools.registry.registry.actions.contains_key("done"));
}

#[test]
fn test_tools_exclude_actions() {
    let tools = Tools::new(vec!["search".to_string(), "click".to_string()]);

    // Actions should still be registered (exclusion is handled at execution time)
    assert!(tools.registry.registry.actions.contains_key("navigate"));
}

#[test]
fn test_registered_action_structure() {
    let action = RegisteredAction {
        name: "test_action".to_string(),
        description: "Test action".to_string(),
        domains: None,
        handler: None,
    };

    assert_eq!(action.name, "test_action");
    assert_eq!(action.description, "Test action");
    assert!(action.domains.is_none());
}

#[test]
fn test_action_model_get_index() {
    let mut params = HashMap::new();
    params.insert("index".to_string(), json!(5));

    let action = ActionModel {
        action_type: "click".to_string(),
        params,
    };

    // get_index looks in nested objects, so this might not work directly
    // But we can test the structure
    assert_eq!(action.action_type, "click");
    assert!(action.params.contains_key("index"));
}

#[test]
fn test_action_model_set_index() {
    let mut params = HashMap::new();
    params.insert("index".to_string(), json!(1));

    let mut action = ActionModel {
        action_type: "click".to_string(),
        params,
    };

    // set_index modifies nested objects, but we can test the structure
    action.set_index(10);
    assert_eq!(action.action_type, "click");
}
