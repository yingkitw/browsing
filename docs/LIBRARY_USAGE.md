# Library Usage Guide

Use `browsing` as a Rust library in your own projects.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
browsing = "0.1"
tokio = { version = "1.40", features = ["full"] }
anyhow = "1.0"
```

## Quick Start

```rust
use anyhow::Result;
use browsing::{Browser, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    browsing::init();
    
    // Load configuration
    let config = Config::from_env();
    
    // Launch browser
    let browser = Browser::launch(config.browser_profile).await?;
    
    // Navigate to a page
    browser.navigate("https://example.com").await?;
    
    // Get page state
    let state = browser.get_browser_state_summary(true).await?;
    println!("Title: {}", state.title);
    
    Ok(())
}
```

## Core Components

### Browser

The `Browser` struct provides browser automation capabilities.

```rust
use browsing::Browser;

// Launch a new browser
let browser = Browser::launch(config.browser_profile).await?;

// Connect to existing browser
let browser = Browser::connect("ws://localhost:9222/...").await?;

// Navigate
browser.navigate("https://example.com").await?;

// Get page state
let state = browser.get_browser_state_summary(true).await?;

// Take screenshot
let screenshot = browser.screenshot().await?;

// Access page actor
let page = browser.page();
```

### Agent

The `Agent` struct provides autonomous browsing capabilities.

```rust
use browsing::{Agent, Browser, Config};

let browser = Browser::launch(config.browser_profile).await?;
let llm = create_your_llm(); // Implement ChatModel trait
let mut agent = Agent::new(browser, llm, config.agent);

// Run autonomous task
let result = agent.run("Find information about Rust").await?;

println!("Completed: {}", result.is_done);
println!("Steps: {}", result.steps);
println!("Output: {:?}", result.output);

// Close browser
agent.close().await?;
```

### Page Actor

Low-level page interactions.

```rust
let page = browser.page();

// Navigate
page.navigate("https://example.com").await?;

// Evaluate JavaScript
let result = page.evaluate("document.title").await?;

// Get element by index
let element = page.get_element_by_index(5).await?;

// Keyboard input
page.keyboard().press_key("Enter").await?;
```

### Element Actor

Interact with specific elements.

```rust
let element = page.get_element_by_index(10).await?;

// Click
element.click().await?;

// Fill input
element.fill("search query").await?;

// Get text
let text = element.text().await?;

// Take screenshot
let screenshot = element.screenshot().await?;

// Get bounding box
let bbox = element.bounding_box().await?;
```

## LLM Integration

Implement the `ChatModel` trait for your LLM provider.

```rust
use browsing::{ChatModel, ChatMessage, ChatInvokeCompletion};
use async_trait::async_trait;
use anyhow::Result;

struct MyLlm {
    // Your LLM client
}

#[async_trait]
impl ChatModel for MyLlm {
    async fn invoke(&self, messages: Vec<ChatMessage>) -> Result<ChatInvokeCompletion> {
        // Call your LLM API
        // Return completion with token usage
    }
    
    async fn invoke_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String>>> {
        // Stream responses from your LLM
    }
}
```

### Example with watsonx

```rust
use watsonx_rs::{WatsonxClient, GenerateStreamRequest};
use browsing::{ChatModel, ChatMessage, ChatInvokeCompletion, ChatInvokeUsage};

struct WatsonxLlm {
    client: WatsonxClient,
    model: String,
}

#[async_trait]
impl ChatModel for WatsonxLlm {
    async fn invoke(&self, messages: Vec<ChatMessage>) -> Result<ChatInvokeCompletion> {
        let prompt = format_messages_for_granite(&messages);
        
        let response = self.client
            .generate(&self.model, &prompt)
            .await?;
        
        Ok(ChatInvokeCompletion {
            content: response.text,
            usage: ChatInvokeUsage {
                prompt_tokens: response.input_tokens,
                completion_tokens: response.output_tokens,
                total_tokens: response.input_tokens + response.output_tokens,
            },
        })
    }
    
    async fn invoke_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String>>> {
        let prompt = format_messages_for_granite(&messages);
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        let mut stream = self.client
            .generate_stream(&self.model, &prompt)
            .await?;
        
        tokio::spawn(async move {
            while let Some(chunk) = stream.next().await {
                let _ = tx.send(chunk).await;
            }
        });
        
        Ok(rx)
    }
}

fn format_messages_for_granite(messages: &[ChatMessage]) -> String {
    // Format according to Granite prompt engineering guide
    // https://www.ibm.com/granite/docs/use-cases/prompt-engineering
    messages.iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}
```

## Custom Actions

Register custom actions with the tools service.

```rust
use browsing::tools::{ActionHandler, ActionResult, ToolsService};
use async_trait::async_trait;

struct CustomAction;

#[async_trait]
impl ActionHandler for CustomAction {
    async fn execute(
        &self,
        browser: &Browser,
        params: Option<serde_json::Value>,
    ) -> Result<ActionResult> {
        // Your custom action logic
        Ok(ActionResult {
            extracted_content: Some("Result".to_string()),
            error: None,
            include_in_memory: true,
        })
    }
}

// Register
let mut tools = ToolsService::new();
tools.register_action("my_action", Box::new(CustomAction));
```

## Configuration

```rust
use browsing::Config;

// From environment variables
let config = Config::from_env();

// From file
let config = Config::load_from_file("config.json")?;

// Manual configuration
let config = Config {
    browser_profile: BrowserProfileConfig {
        headless: Some(true),
        user_data_dir: Some("/path/to/data".into()),
        allowed_domains: Some(vec!["example.com".to_string()]),
        downloads_path: None,
        proxy: None,
    },
    llm: LlmConfig {
        api_key: Some("key".to_string()),
        model: Some("ibm/granite-4-h-small".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2000),
    },
    agent: AgentConfig {
        max_steps: Some(100),
        use_vision: Some(false),
        system_prompt: None,
    },
};
```

## Error Handling

All functions return `Result<T>` with `BrowsingError`.

```rust
use browsing::{BrowsingError, Result};

match browser.navigate("https://example.com").await {
    Ok(_) => println!("Success"),
    Err(BrowsingError::Navigation(msg)) => eprintln!("Navigation error: {}", msg),
    Err(BrowsingError::Cdp(msg)) => eprintln!("CDP error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Examples

See the `examples/` directory:
- `library_usage.rs`: Basic library usage
- `custom_actions.rs`: Custom action registration
- `simple_agent.rs`: Simple agent example

Run examples:

```bash
cargo run --example library_usage
cargo run --example custom_actions
```

## Best Practices

1. **Initialize Logging**: Call `browsing::init()` at startup
2. **Error Handling**: Use `?` operator for error propagation
3. **Resource Cleanup**: Ensure browser is closed with `agent.close()` or drop
4. **Configuration**: Use environment variables or config files
5. **Testing**: Use the `ChatModel` trait for mock LLMs in tests
6. **Async Runtime**: Use tokio runtime with full features
7. **Element Indices**: Use `get_browser_state_summary()` to see element indices
8. **Screenshots**: Enable for visual debugging and verification

## API Documentation

Generate and view full API documentation:

```bash
cargo doc --open
```
