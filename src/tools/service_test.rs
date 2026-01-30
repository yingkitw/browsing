//! Tests for tools service

#[cfg(test)]
mod tests {
    use super::super::service::Tools;
    use super::super::views::ActionModel;
    use serde_json::json;

    #[test]
    fn test_tools_new() {
        let tools = Tools::new(vec![]);
        assert!(!tools.registry.registry.actions.is_empty());
    }

    #[test]
    fn test_tools_exclude_actions() {
        let tools = Tools::new(vec!["search".to_string(), "click".to_string()]);
        // Actions should be registered but we can check exclusion logic
        assert!(!tools.registry.registry.actions.is_empty());
    }

    #[test]
    fn test_action_model_parsing() {
        let params_json = json!({
            "query": "test query",
            "engine": "duckduckgo"
        });
        let params: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(params_json).unwrap();

        let action = ActionModel {
            action_type: "search".to_string(),
            params,
        };

        assert_eq!(action.action_type, "search");
        assert!(action.params.get("query").is_some());
    }

    #[test]
    fn test_action_model_navigate() {
        let params_json = json!({
            "url": "https://example.com"
        });
        let params: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(params_json).unwrap();

        let action = ActionModel {
            action_type: "navigate".to_string(),
            params,
        };

        assert_eq!(action.action_type, "navigate");
        assert_eq!(
            action.params.get("url").and_then(|v| v.as_str()),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_action_model_click() {
        let params_json = json!({
            "index": 1
        });
        let params: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(params_json).unwrap();

        let action = ActionModel {
            action_type: "click".to_string(),
            params,
        };

        assert_eq!(action.action_type, "click");
        assert_eq!(action.params.get("index").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn test_action_model_input() {
        let params_json = json!({
            "index": 2,
            "text": "test input"
        });
        let params: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(params_json).unwrap();

        let action = ActionModel {
            action_type: "input".to_string(),
            params,
        };

        assert_eq!(action.action_type, "input");
        assert_eq!(
            action.params.get("text").and_then(|v| v.as_str()),
            Some("test input")
        );
    }

    #[test]
    fn test_action_model_extract() {
        let params_json = json!({
            "query": "extract all links",
            "extract_links": true,
            "start_from_char": 0
        });

        let params: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(params_json).unwrap();

        let action = ActionModel {
            action_type: "extract".to_string(),
            params,
        };

        assert_eq!(action.action_type, "extract");
        assert_eq!(
            action.params.get("query").and_then(|v| v.as_str()),
            Some("extract all links")
        );
        assert_eq!(
            action.params.get("extract_links").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
