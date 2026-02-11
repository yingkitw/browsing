# Browser-Use-RS Examples

This directory contains comprehensive examples demonstrating the capabilities of browser-use-rs.

## Available Examples

### 1. Comprehensive Showcase (`comprehensive_showcase.rs`)

A full-featured demonstration of browser automation capabilities including:

- **Browser Automation**: Starting and managing browser instances
- **Multi-tab Management**: Creating, switching, and managing multiple tabs
- **DOM Extraction**: Extracting structured content from web pages
- **LLM-Driven Navigation**: Autonomous agent-based web browsing
- **Search Operations**: Performing searches on websites
- **Scrolling**: Navigating through long pages
- **Navigation History**: Using browser back/forward functionality
- **Token Tracking**: Monitoring LLM token usage
- **Error Handling**: Graceful error recovery

**Run it:**
```bash
cargo run --example comprehensive_showcase
```

**Features Demonstrated:**
- ✅ Browser profile configuration
- ✅ DOM processor setup
- ✅ Mock LLM implementation (for testing without API keys)
- ✅ Agent creation and execution
- ✅ Multi-step workflow with 9 predefined actions
- ✅ Execution summary and token usage reporting

**Note:** This example uses a mock LLM with predefined responses, so it will work without any API keys. The browser will open in non-headless mode so you can see the automation in action.

### 2. Browse, Navigate, Extract (`browse_navigate_extract.rs`)

A focused example for the browse → navigate → get text → get image flow:

- Start browser and navigate to example.com
- Extract page text (innerText)
- Take screenshot (save as PNG)
- Navigate to rust-lang.org
- Extract text and screenshot again
- Go back in history

**Run it:**
```bash
cargo run --example browse_navigate_extract
BROWSER_USE_HEADLESS=true cargo run --example browse_navigate_extract  # headless
```

**Features Demonstrated:**
- ✅ Full browse/navigate/text/image flow
- ✅ Multi-page navigation
- ✅ Text extraction (lightweight innerText)
- ✅ Screenshot capture (2 images)
- ✅ History navigation (go back)

### 3. Simple Navigation (`simple_navigation.rs`)

A straightforward example demonstrating basic browser operations:

- Starting a browser
- Navigating to URLs
- Extracting DOM content
- Taking screenshots
- Managing tabs
- Using navigation history

**Run it:**
```bash
cargo run --example simple_navigation
```

**Features Demonstrated:**
- ✅ Basic browser lifecycle management
- ✅ URL navigation
- ✅ DOM content extraction
- ✅ Screenshot capture
- ✅ Tab creation and switching
- ✅ Browser history navigation

**Note:** This example doesn't require an LLM or API keys. It demonstrates direct browser control using the BrowserClient trait.

## Example Structure

Each example follows this pattern:

1. **Setup**: Create browser profile and configuration
2. **Initialization**: Start browser and create necessary components
3. **Execution**: Perform the demonstration tasks
4. **Reporting**: Display results and statistics
5. **Cleanup**: Graceful shutdown (or keep browser open for inspection)

## Building Examples

Build all examples:
```bash
cargo build --examples
```

Build a specific example:
```bash
cargo build --example comprehensive_showcase
```

## Running Examples

Run with default settings:
```bash
cargo run --example comprehensive_showcase
```

Run with release optimizations:
```bash
cargo run --release --example comprehensive_showcase
```

## Customizing Examples

### Using Real LLM

To use a real LLM instead of the mock, implement the `ChatModel` trait:

```rust
use browsing::llm::ChatModel;
use async_trait::async_trait;

struct MyLLM {
    api_key: String,
}

#[async_trait]
impl ChatModel for MyLLM {
    fn model(&self) -> &str { "my-model" }
    fn provider(&self) -> &str { "my-provider" }
    
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatInvokeCompletion<String>> {
        // Your LLM API implementation here
        todo!()
    }
    
    async fn chat_stream(&self, messages: &[ChatMessage]) 
        -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        // Your streaming implementation here
        todo!()
    }
}

let llm = MyLLM {
    api_key: std::env::var("YOUR_API_KEY")?,
};
```

### Headless Mode

To run in headless mode (no visible browser window):

```rust
let profile = BrowserProfile {
    headless: Some(true),
    ..Default::default()
};
```

### Custom Browser Path

To use a specific Chrome/Chromium installation:

```rust
use browsing::browser::launcher::BrowserLauncher;

let launcher = BrowserLauncher::new(profile)
    .with_executable_path(std::path::PathBuf::from("/path/to/chrome"));
```

## Requirements

- **Chrome or Chromium**: Must be installed and accessible
- **Rust**: Edition 2024 or later
- **Network**: Internet connection for navigating to websites
- **API Keys** (optional): Only needed if using real LLM providers

## Troubleshooting

### Browser Not Found

If you get an error about Chrome not being found:

1. Install Chrome or Chromium
2. Or specify the path explicitly using `BrowserLauncher`

### Connection Errors

If you see CDP connection errors:

1. Ensure no other browser instances are using the same port
2. Try closing all Chrome instances and running again
3. Check firewall settings

### LLM Errors

If using a real LLM and getting errors:

1. Verify your API key is set correctly
2. Check your network connection
3. Ensure the model name is correct
4. Review rate limits for your API

## Next Steps

After running these examples:

1. **Explore the API**: Run `cargo doc --open` to view full documentation
2. **Customize Actions**: Modify the predefined actions in the examples
3. **Implement Custom LLM**: Create your own `ChatModel` implementation
4. **Build Your Agent**: Use these examples as templates for your own projects
5. **Add Custom Actions**: Extend the action system with domain-specific behaviors

## Contributing

Found a bug or want to add a new example? Contributions are welcome!

1. Fork the repository
2. Create your example
3. Test it thoroughly
4. Submit a pull request

## License

These examples are part of the browser-use-rs project and follow the same MIT license.
