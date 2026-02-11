//! Full integration test demonstrating browsing workflow
//!
//! This test shows the expected workflow for browser automation:
//! 1. Create browser with headless configuration
//! 2. Configure LLM (mock in tests)
//! 3. Create DOM processor
//! 4. Create agent with task
//! 5. Run agent to completion
//!
//! Note: This test requires Chrome/Chromium to be installed
//! In CI environments without a browser, the test will skip gracefully

#[cfg(test)]
mod integration_workflow {
    #[tokio::test]
    #[ignore] // Run manually to test full workflow
    async fn test_complete_web_automation_workflow() {
        use browsing::agent::service::Agent;
        use browsing::browser::{Browser, BrowserProfile};
        use browsing::agent::views::AgentSettings;
        use browsing::dom::DOMProcessorImpl;

        // Step 1: Create browser with headless configuration
        let profile = BrowserProfile {
            headless: Some(true), // Essential for CI/automation
            user_data_dir: None,
            allowed_domains: None, // Allow all domains
            downloads_path: Some("/tmp/browser_downloads".into()),
            proxy: None,
        };

        let browser = Box::new(Browser::new(profile));

        // Step 2: Configure LLM (mock for testing)
        let llm = create_mock_llm();

        // Step 3: Create DOM processor
        let dom_processor = Box::new(DOMProcessorImpl::new());

        // Step 4: Create agent with task
        let task = "Navigate to https://example.com and extract main heading text".to_string();

        let mut agent = Agent::new(task, browser, dom_processor, llm)
            .with_max_steps(10)
            .with_settings(AgentSettings {
                use_vision: browsing::agent::views::VisionMode::Auto,
                max_failures: 3,
                use_thinking: true,
                ..Default::default()
            });
        
        // Step 4: Run agent to completion
        match agent.run().await {
            Ok(history) => {
                // Verify agent completed successfully
                assert!(history.is_done());
                assert!(history.is_successful().unwrap_or(false));
                
                // Check that steps were taken
                assert!(history.number_of_steps() > 0);
                
                // Verify no errors occurred
                assert!(!history.has_errors());
                
                println!("Agent completed {} steps successfully", history.number_of_steps());
                println!("Total duration: {:.2}s", history.total_duration_seconds());
            }
            Err(e) => {
                // Log error for debugging
                eprintln!("Agent failed: {:?}", e);
                
                // In tests, we might want to assert certain types of failures
                match e {
                    browsing::error::BrowsingError::Browser(msg) 
                        if msg.contains("No browser executable") => {
                            // Expected in environments without Chrome
                            println!("Skipping test: {}", msg);
                            return;
                        }
                    _ => panic!("Unexpected error: {:?}", e),
                }
            }
        }
        
        // Cleanup would happen automatically when browser goes out of scope
        // In production, you might want explicit cleanup:
        // browser.stop().await?;
    }
    
    fn create_mock_llm() -> impl browsing::llm::base::ChatModel {
        use async_trait::async_trait;
        use browsing::llm::base::{
            ChatInvokeCompletion, ChatInvokeUsage, ChatMessage, ChatModel
        };
        use serde_json::json;
        use std::sync::Mutex;
        
        struct MockLLM {
            responses: Vec<String>,
            index: std::sync::Mutex<usize>,
        }
        
        #[async_trait]
        impl ChatModel for MockLLM {
            fn model(&self) -> &str {
                "mock-model"
            }

            fn provider(&self) -> &str {
                "mock-provider"
            }
            
            async fn chat(&self, _messages: &[ChatMessage]) -> browsing::error::Result<ChatInvokeCompletion<String>> {
                let index = {
                    let mut idx = self.index.lock().unwrap();
                    let current = *idx;
                    *idx += 1;
                    current
                };
                
                // Mock realistic agent responses
                let completion = if index == 0 {
                    // First response: plan navigation
                    json!({
                        "action": [
                            {
                                "action_type": "navigate",
                                "params": {"url": "https://example.com"}
                            }
                        ]
                    }).to_string()
                } else if index == 1 {
                    // Second response: extract heading
                    json!({
                        "action": [
                            {
                                "action_type": "extract_content",
                                "params": {"selector": "h1"}
                            }
                        ]
                    }).to_string()
                } else {
                    // Final response: task complete
                    json!({
                        "action": [
                            {
                                "action_type": "done",
                                "params": {}
                            }
                        ]
                    }).to_string()
                };
                
                Ok(ChatInvokeCompletion {
                    completion,
                    usage: Some(ChatInvokeUsage {
                        prompt_tokens: 100,
                        prompt_cached_tokens: None,
                        prompt_cache_creation_tokens: None,
                        prompt_image_tokens: None,
                        completion_tokens: 50,
                        total_tokens: 150,
                    }),
                    thinking: Some(format!("Mock thinking for step {}", index + 1)),
                    redacted_thinking: None,
                    stop_reason: Some("stop".to_string()),
                })
            }

            async fn chat_stream(
                &self,
                _messages: &[ChatMessage],
            ) -> browsing::error::Result<Box<dyn futures::Stream<Item = browsing::error::Result<String>> + Send + Unpin>> {
                // For testing purposes, return a simple stream with one message
                let response = "Mock response";
                Ok(Box::new(Box::pin(futures::stream::once(async move { 
                    Ok(response.to_string()) 
                }))))
            }
        }
        
        MockLLM {
            responses: vec![
                "I will navigate to example.com and extract heading".to_string(),
                "I've navigated and am now extracting the main heading".to_string(),
            ],
            index: std::sync::Mutex::new(0),
        }
    }
}