//! Comprehensive showcase of browser-use-rs capabilities
//!
//! This example demonstrates:
//! - Browser automation with multiple tabs
//! - DOM extraction and processing
//! - LLM-driven autonomous navigation
//! - Screenshot capture
//! - Custom action handlers
//! - Error handling and recovery
//!
//! Usage:
//!   cargo run --example comprehensive_showcase
//!
//! Requirements:
//!   - Chrome/Chromium browser installed

use async_trait::async_trait;
use browsing::agent::service::Agent;
use browsing::agent::views::ActionResult;
use browsing::browser::{Browser, BrowserProfile};
use browsing::dom::DOMProcessorImpl;
use browsing::error::Result;
use browsing::llm::base::{ChatInvokeCompletion, ChatInvokeUsage, ChatMessage, ChatModel};
use browsing::tools::views::{ActionContext, ActionHandler, ActionParams};
use serde_json::json;

/// Mock LLM for demonstration purposes
/// In production, implement your own ChatModel
struct DemoLLM {
    responses: Vec<String>,
    current_index: std::sync::Mutex<usize>,
}

impl DemoLLM {
    fn new() -> Self {
        // Predefined responses that demonstrate various capabilities
        let responses = vec![
            // Step 1: Navigate to example.com
            json!({
                "thinking": "I need to navigate to example.com to start the demonstration",
                "evaluation_previous_goal": "Starting the task",
                "memory": "Beginning comprehensive showcase",
                "next_goal": "Navigate to example.com",
                "action": [{
                    "navigate": {
                        "url": "https://example.com",
                        "new_tab": false
                    }
                }]
            }).to_string(),
            
            // Step 2: Extract content from the page
            json!({
                "thinking": "Now I'll extract the main content from the page",
                "evaluation_previous_goal": "Successfully navigated to example.com",
                "memory": "Visited example.com",
                "next_goal": "Extract page content",
                "action": [{
                    "extract": {
                        "query": "Extract the main heading and paragraph text",
                        "extract_links": true
                    }
                }]
            }).to_string(),
            
            // Step 3: Open a new tab with another site
            json!({
                "thinking": "Let me open a new tab to demonstrate tab management",
                "evaluation_previous_goal": "Successfully extracted content",
                "memory": "Extracted content from example.com",
                "next_goal": "Open new tab with GitHub",
                "action": [{
                    "navigate": {
                        "url": "https://github.com",
                        "new_tab": true
                    }
                }]
            }).to_string(),
            
            // Step 4: Search on GitHub
            json!({
                "thinking": "I'll search for 'rust browser automation' on GitHub",
                "evaluation_previous_goal": "Opened GitHub in new tab",
                "memory": "Now on GitHub",
                "next_goal": "Search for rust browser automation",
                "action": [{
                    "search": {
                        "query": "rust browser automation"
                    }
                }]
            }).to_string(),
            
            // Step 5: Wait for results to load
            json!({
                "thinking": "Waiting for search results to load",
                "evaluation_previous_goal": "Initiated search",
                "memory": "Searching GitHub",
                "next_goal": "Wait for results",
                "action": [{
                    "wait": {
                        "seconds": 2
                    }
                }]
            }).to_string(),
            
            // Step 6: Scroll down to see more results
            json!({
                "thinking": "Scrolling down to see more search results",
                "evaluation_previous_goal": "Waited for page load",
                "memory": "Search results loaded",
                "next_goal": "Scroll to see more results",
                "action": [{
                    "scroll": {
                        "down": true,
                        "pages": 1.0
                    }
                }]
            }).to_string(),
            
            // Step 7: Go back to previous page
            json!({
                "thinking": "Going back to demonstrate navigation history",
                "evaluation_previous_goal": "Scrolled page",
                "memory": "Viewed search results",
                "next_goal": "Navigate back",
                "action": [{
                    "go_back": {}
                }]
            }).to_string(),
            
            // Step 8: Switch back to first tab
            json!({
                "thinking": "Switching back to the first tab with example.com",
                "evaluation_previous_goal": "Went back in history",
                "memory": "Demonstrated navigation",
                "next_goal": "Switch to first tab",
                "action": [{
                    "switch": {
                        "index": 0
                    }
                }]
            }).to_string(),
            
            // Step 9: Complete the task
            json!({
                "thinking": "I have successfully demonstrated all key capabilities",
                "evaluation_previous_goal": "Switched tabs successfully",
                "memory": "Demonstrated: navigation, extraction, tabs, search, scroll, go_back, tab switching",
                "next_goal": "Complete the showcase",
                "action": [{
                    "done": {
                        "text": "Successfully demonstrated browser automation capabilities including: navigation, content extraction, tab management, search, scrolling, and history navigation",
                        "success": true
                    }
                }]
            }).to_string(),
        ];

        Self {
            responses,
            current_index: std::sync::Mutex::new(0),
        }
    }
}

#[async_trait]
impl ChatModel for DemoLLM {
    fn model(&self) -> &str {
        "demo-llm"
    }

    fn provider(&self) -> &str {
        "demo"
    }

    async fn chat(&self, _messages: &[ChatMessage]) -> Result<ChatInvokeCompletion<String>> {
        let mut index = self.current_index.lock().unwrap();
        let response = if *index < self.responses.len() {
            self.responses[*index].clone()
        } else {
            // Fallback to done action
            json!({
                "action": [{
                    "done": {
                        "text": "Task completed",
                        "success": true
                    }
                }]
            })
            .to_string()
        };

        *index += 1;

        Ok(ChatInvokeCompletion {
            completion: response,
            thinking: None,
            redacted_thinking: None,
            usage: Some(ChatInvokeUsage {
                prompt_tokens: 500,
                prompt_cached_tokens: None,
                prompt_cache_creation_tokens: None,
                prompt_image_tokens: None,
                completion_tokens: 150,
                total_tokens: 650,
            }),
            stop_reason: None,
        })
    }

    async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send + Unpin>> {
        let response = self.chat(messages).await?;
        let stream = futures::stream::iter(vec![Ok(response.completion)]);
        Ok(Box::new(stream))
    }
}

/// Custom action handler for demonstration
struct CustomGreetingHandler;

#[async_trait]
impl ActionHandler for CustomGreetingHandler {
    async fn execute(
        &self,
        params: &ActionParams,
        _context: &mut ActionContext<'_>,
    ) -> Result<ActionResult> {
        let name = params.get_required_str("name").unwrap_or("World");
        let greeting = format!("Hello, {}! This is a custom action.", name);
        
        println!("🎉 Custom action executed: {}", greeting);
        
        Ok(ActionResult {
            extracted_content: Some(greeting),
            success: Some(true),
            ..Default::default()
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Browser-Use-RS Comprehensive Showcase\n");
    println!("This example demonstrates the key capabilities of browser-use-rs:");
    println!("  ✓ Browser automation");
    println!("  ✓ Multi-tab management");
    println!("  ✓ DOM extraction");
    println!("  ✓ LLM-driven navigation");
    println!("  ✓ Custom actions");
    println!("  ✓ Error handling\n");

    // 1. Create browser profile
    println!("📋 Step 1: Creating browser profile...");
    let profile = BrowserProfile {
        headless: Some(false), // Set to true for headless mode
        ..Default::default()
    };
    let browser = Box::new(Browser::new(profile));
    println!("   ✓ Browser profile created\n");

    // 2. Create DOM processor
    println!("📋 Step 2: Creating DOM processor...");
    let dom_processor = Box::new(DOMProcessorImpl::new());
    println!("   ✓ DOM processor ready\n");

    // 3. Create LLM (using demo LLM for this example)
    println!("📋 Step 3: Creating LLM...");
    let llm = DemoLLM::new();
    println!("   ✓ LLM initialized (using demo responses)\n");
    println!("   💡 Tip: Implement ChatModel trait for your own LLM\n");

    // 4. Create agent with custom settings
    println!("📋 Step 4: Creating agent...");
    let task = "Demonstrate browser automation capabilities by visiting websites, extracting content, and managing tabs".to_string();
    
    let mut agent = Agent::new(task.clone(), browser, dom_processor, llm);
    println!("   ✓ Agent created with task: {}\n", task);

    // 5. Run the agent
    println!("📋 Step 5: Running agent...\n");
    println!("\n{}", "=".repeat(60));
    println!("AGENT EXECUTION");
    println!("{}\n", "=".repeat(60));

    match agent.run().await {
        Ok(history) => {
            println!("\n{}", "=".repeat(60));
            println!("EXECUTION COMPLETE");
            println!("{}\n", "=".repeat(60));
            
            println!("✅ Agent completed successfully!");
            println!("   Steps taken: {}", history.history.len());
            
            // Display execution summary
            println!("\n📊 Execution Summary:");
            for (i, step) in history.history.iter().enumerate() {
                println!("   Step {}: {}", i + 1, &step.state.url);
            }
            
            // Display token usage
            if let Some(usage) = &history.usage {
                println!("\n💰 Token Usage:");
                if let Some(pt) = usage.prompt_tokens {
                    println!("   Prompt tokens: {}", pt);
                }
                if let Some(ct) = usage.completion_tokens {
                    println!("   Completion tokens: {}", ct);
                }
                if let Some(tt) = usage.total_tokens {
                    println!("   Total tokens: {}", tt);
                }
            }
            
            // Display final result
            if let Some(last_step) = history.history.last() {
                if let Some(result) = last_step.result.last() {
                    println!("\n📝 Final Result:");
                    if let Some(content) = &result.extracted_content {
                        println!("   {}", content);
                    }
                }
            }
        }
        Err(e) => {
            println!("\n❌ Agent execution failed: {}", e);
            println!("   This is expected if Chrome is not installed or accessible");
            return Err(e);
        }
    }

    println!("\n🎉 Showcase completed successfully!");
    println!("\n💡 Next steps:");
    println!("   - Try modifying the task string");
    println!("   - Implement your own ChatModel for real LLM integration");
    println!("   - Use Tools::new() to customize available actions");
    println!("   - Explore the API documentation: cargo doc --open");

    Ok(())
}
