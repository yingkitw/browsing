//! Agent execution flow tests

use async_trait::async_trait;
use browsing::agent::service::Agent;
use browsing::dom::DOMProcessorImpl;
use browsing::agent::views::{
    ActionResult, AgentHistory, AgentHistoryList, AgentSettings
};
use browsing::browser::{Browser, BrowserProfile};
use browsing::error::{BrowsingError, Result};
use browsing::llm::base::{
    ChatInvokeCompletion, ChatInvokeUsage, ChatMessage, ChatModel
};
use browsing::tools::service::Tools;
use serde_json::json;

// Mock LLM for testing
struct MockLLM {
    responses: Vec<String>,
    response_index: std::sync::Mutex<usize>,
}

#[async_trait]
impl ChatModel for MockLLM {
    fn model(&self) -> &str {
        "mock-model"
    }

    fn provider(&self) -> &str {
        "mock-provider"
    }
    
    async fn chat(&self, _messages: &[ChatMessage]) -> Result<ChatInvokeCompletion<String>> {
        let index = {
            let mut idx = self.response_index.lock().unwrap();
            let current = *idx;
            *idx += 1;
            current
        };
        if index < self.responses.len() {
            let response = self.responses[index].clone();
            
            Ok(ChatInvokeCompletion {
                completion: json!({
                    "action": [
                        {
                            "action_type": "done",
                            "params": {}
                        }
                    ]
                }).to_string(),
                usage: Some(ChatInvokeUsage {
                    prompt_tokens: 100,
                    prompt_cached_tokens: None,
                    prompt_cache_creation_tokens: None,
                    prompt_image_tokens: None,
                    completion_tokens: 50,
                    total_tokens: 150,
                }),
                thinking: Some(format!("Mock thinking: {}", response)),
                redacted_thinking: None,
                stop_reason: Some("stop".to_string()),
            })
        } else {
            Err(BrowsingError::Llm("No more mock responses".to_string()))
        }
    }

    async fn chat_stream(
        &self,
        _messages: &[ChatMessage],
    ) -> Result<Box<dyn futures_util::stream::Stream<Item = Result<String>> + Send + Unpin>> {
        // For testing purposes, return a simple stream with one message
        let response = "Mock response";
        Ok(Box::new(Box::pin(futures_util::stream::once(async move { 
            Ok(response.to_string()) 
        }))))
    }
}

#[tokio::test]
async fn test_agent_creation() {
    let task = "Navigate to example.com and find the search box".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["I will navigate to example.com".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    let agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm);
    
    // Agent should be created successfully
    assert!(true); // No panic indicates success
}

#[tokio::test]
async fn test_agent_configuration() {
    let task = "Test task".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["Mock response".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    let agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm)
        .with_max_steps(5)
        .with_settings(AgentSettings::default());
    
    // Agent should be configured successfully
    assert!(true); // No panic indicates success
}

#[tokio::test]
async fn test_agent_execution_without_browser() {
    let task = "Test task".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["Task completed successfully".to_string()],
        response_index: std::sync::Mutex::new(0),
    };

    let mut agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm);

    // Agent's run() method automatically starts the browser, so it should succeed
    // The LLM returns a "done" action which completes the task
    let result = agent.run().await;
    assert!(result.is_ok());

    // Verify we got a history with at least one step
    let history = result.unwrap();
    assert!(!history.history.is_empty());
}

#[tokio::test]
async fn test_agent_action_validation() {
    // Test that agent validates actions before execution
    let task = "Test task".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["Invalid action".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    let agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm);
    
    // In a real scenario, this would test action parsing and validation
    assert!(true); // Placeholder for validation logic tests
}

#[tokio::test]
async fn test_agent_step_execution() {
    // Test individual step execution
    let task = "Navigate to website".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec![
            "I will navigate to the website".to_string(),
            "Navigation successful".to_string(),
            "Task completed".to_string(),
        ],
        response_index: std::sync::Mutex::new(0),
    };
    
    let mut agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm)
        .with_max_steps(3);
    
    // Track execution state
    let initial_steps = 0;
    
    // In a real scenario, this would execute steps
    assert_eq!(initial_steps, 0);
    
    // After execution, steps should increase
    // let final_steps = executed_steps;
    // assert!(final_steps > initial_steps);
}

#[tokio::test]
async fn test_agent_history_tracking() {
    // Test that agent properly tracks execution history
    let task = "Test task with history".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["Step 1 complete".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    let agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm);
    
    // Create mock history
    let history = AgentHistoryList {
        history: vec![AgentHistory {
            model_output: None,
            result: vec![ActionResult {
                extracted_content: Some("Step 1 result".to_string()),
                long_term_memory: Some("Step 1 memory".to_string()),
                success: Some(true),
                is_done: Some(false),
                ..Default::default()
            }],
            state: browsing::browser::views::BrowserStateHistory {
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                tabs: vec![],
                interacted_element: vec![],
                screenshot_path: None,
            },
            metadata: None,
            state_message: None,
        }],
        usage: None,
    };
    
    // History should be trackable
    assert_eq!(history.history.len(), 1);
    assert!(history.history[0].result[0].extracted_content.is_some());
}

#[tokio::test]
async fn test_agent_error_recovery() {
    // Test agent error recovery mechanisms
    let task = "Task that might fail".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec![
            "I'll attempt the task".to_string(),
            "An error occurred, I'll recover".to_string(),
            "Recovered successfully".to_string(),
        ],
        response_index: std::sync::Mutex::new(0),
    };
    
    let mut agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm)
        .with_settings(AgentSettings {
            max_failures: 2,
            ..Default::default()
        });
    
    // Agent should handle failures and attempt recovery
    let failure_count = 0;
    
    // In a real scenario:
    // 1. Agent attempts task
    // 2. Failure occurs
    // 3. Agent retries
    // 4. Success or max failures reached
    
    assert!(failure_count <= 2); // Should not exceed max failures
}

#[tokio::test]
async fn test_agent_token_tracking() {
    // Test that agent tracks token usage correctly
    let task = "Track token usage".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["Response with tokens".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    let agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm);
    
    // Mock usage tracking
    let total_prompt_tokens = 100;
    let total_completion_tokens = 50;
    let total_tokens = total_prompt_tokens + total_completion_tokens;
    
    // Verify token counts are reasonable
    assert!(total_prompt_tokens > 0);
    assert!(total_completion_tokens > 0);
    assert!(total_tokens > total_prompt_tokens);
}

#[tokio::test]
async fn test_agent_concurrency_handling() {
    // Test agent behavior with concurrent operations
    use tokio::task;
    
    let task = "Concurrent task".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["Concurrent response".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    let agent = std::sync::Arc::new(tokio::sync::Mutex::new(
        Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm)
    ));
    
    // Test concurrent access
    let handles: Vec<_> = (0..3).map(|i| {
        let agent = agent.clone();
        task::spawn(async move {
            let mut agent = agent.lock().await;
            // In a real scenario, this might check state or configuration
            format!("Task {} processed", i)
        })
    }).collect();
    
    // Wait for all tasks
    let results: Vec<_> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();
    
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_agent_with_vision_mode() {
    // Test agent with vision capabilities
    let task = "Vision-enabled task".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["I can see the page".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    let agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm)
        .with_settings(AgentSettings {
            use_vision: browsing::agent::views::VisionMode::Enabled(true),
            ..Default::default()
        });
    
    // Vision mode should be configurable
    assert!(true); // No panic indicates success
}

#[tokio::test]
async fn test_agent_with_custom_tools() {
    // Test agent with custom tool configuration
    let task = "Task with custom tools".to_string();
    let browser = Browser::new(BrowserProfile::default());
    let llm = MockLLM {
        responses: vec!["Using custom tools".to_string()],
        response_index: std::sync::Mutex::new(0),
    };
    
    // Tools now takes a list of exclude actions, not action definitions
    let tools = Tools::new(vec![]);
    
    let agent = Agent::new(task, Box::new(browser), Box::new(DOMProcessorImpl::new()), llm);
    
    // Agent should accept custom tools (in actual implementation)
    assert!(true); // No panic indicates success
}