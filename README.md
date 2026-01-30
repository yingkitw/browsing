# Browsing

**Autonomous web browsing for AI agents - Rust implementation**

Browsing is a powerful Rust library that enables AI agents to autonomously interact with web pages. It provides a clean, trait-based architecture for browser automation, DOM extraction, and LLM-driven web interactions.

## ✨ Why Browsing?

Building AI agents that can navigate and interact with websites is challenging. You need to:

- **Extract structured data from unstructured HTML** - Parse complex DOM trees and make them LLM-readable
- **Handle browser automation reliably** - Manage browser lifecycle, CDP connections, and process management
- **Coordinate multiple subsystems** - Orchestrate DOM extraction, LLM inference, and action execution
- **Maintain testability** - Mock components for unit testing without real browsers
- **Support extensibility** - Add custom actions, browser backends, and LLM providers

**Browsing solves all of this** with a clean, modular, and well-tested architecture.

## 🎯 Key Features

### 🏗️ Trait-Based Architecture
- **BrowserClient trait** - Abstract browser operations for easy mocking and alternative backends
- **DOMProcessor trait** - Pluggable DOM processing implementations
- **ActionHandler trait** - Extensible action system for custom behaviors

### 🤖 Autonomous Agent System
- Complete agent execution loop with LLM integration
- Robust action parsing with JSON repair
- History tracking with state snapshots
- Graceful error handling and recovery

### 🌐 Full Browser Automation
- Cross-platform support (macOS, Linux, Windows)
- Automatic browser detection
- Chrome DevTools Protocol (CDP) integration
- Tab management (create, switch, close)
- Screenshot capture (page and element-level)

### 📊 Advanced DOM Processing
- Full CDP integration (DOM, AX tree, Snapshot)
- LLM-ready serialization with interactive element indices
- Accessibility tree support for better semantic understanding
- Optimized for token efficiency

### 🔧 Extensible & Maintainable
- Manager-based architecture (TabManager, NavigationManager, ScreenshotManager)
- Custom action registration
- Utility traits for reduced code duplication
- Comprehensive test coverage (74+ tests)

## 📦 Installation

```bash
# Add to Cargo.toml
[dependencies]
browsing = "0.1"
```

```bash
# Or clone from source
git clone <repository>
cd browsing-rs
cargo build
```

## 🚀 Quick Start

### Basic Example

```rust
use browsing::{Agent, Browser, BrowserProfile};
use browsing::dom::DOMProcessorImpl;
use browsing::llm::ChatModel;

// Implement your own LLM by implementing the ChatModel trait
struct MyLLM;

#[async_trait::async_trait]
impl ChatModel for MyLLM {
    fn model(&self) -> &str { "my-model" }
    fn provider(&self) -> &str { "my-provider" }
    
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatInvokeCompletion<String>> {
        // Your LLM implementation here
        todo!()
    }
    
    async fn chat_stream(&self, messages: &[ChatMessage]) 
        -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        // Your streaming implementation here
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create browser profile
    let profile = BrowserProfile::default();
    let browser = Box::new(Browser::new(profile));

    // 2. Create your LLM implementation
    let llm = MyLLM;

    // 3. Create DOM processor
    let dom_processor = Box::new(DOMProcessorImpl::new());

    // 4. Create and run agent
    let mut agent = Agent::new(
        "Find the top post on Hacker News".to_string(),
        browser,
        dom_processor,
        llm,
    );

    let history = agent.run().await?;
    println!("✅ Completed in {} steps", history.history.len());

    Ok(())
}
```

### Browser Launch Options

```rust
use browsing::{Browser, BrowserProfile};

// Option 1: Auto-launch browser (default)
let profile = BrowserProfile::default();
let browser = Browser::new(profile);

// Option 2: Connect to existing browser
let browser = Browser::new(profile)
    .with_cdp_url("http://localhost:9222".to_string());

// Option 3: Custom browser executable
use browsing::browser::launcher::BrowserLauncher;
let launcher = BrowserLauncher::new(profile)
    .with_executable_path(std::path::PathBuf::from("/path/to/chrome"));
```

### Using Traits for Testing

```rust
use browsing::traits::{BrowserClient, DOMProcessor};
use browsing::agent::Agent;
use std::sync::Arc;

// Create mock browser for testing
struct MockBrowser {
    navigation_count: std::sync::atomic::AtomicUsize,
}

#[async_trait::async_trait]
impl BrowserClient for MockBrowser {
    async fn start(&mut self) -> Result<(), BrowsingError> {
        Ok(())
    }

    async fn navigate(&mut self, _url: &str) -> Result<(), BrowsingError> {
        self.navigation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    // ... implement other trait methods
}

#[tokio::test]
async fn test_agent_with_mock_browser() {
    let mock_browser = Box::new(MockBrowser {
        navigation_count: std::sync::atomic::AtomicUsize::new(0),
    });

    // Test agent behavior without real browser
    let dom_processor = Box::new(MockDOMProcessor::new());
    let llm = MockLLM::new();

    let mut agent = Agent::new("Test task".to_string(), mock_browser, dom_processor, llm);
    // ... test agent
}
```

## 📚 Usage Examples

### Screenshot Capture

```rust
use browsing::Browser;

let browser = Browser::new(BrowserProfile::default());
browser.start().await?;

// Full page screenshot
let screenshot_data = browser.take_screenshot(
    Some("screenshot.png"),  // path
    true,                      // full_page
).await?;

// Viewport only
let viewport = browser.take_screenshot(
    Some("viewport.png"),
    false,
).await?;
```

### Direct Browser Control

```rust
use browsing::{Browser, BrowserProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut browser = Browser::new(BrowserProfile::default());
    browser.start().await?;

    // Navigate
    browser.navigate("https://example.com").await?;

    // Get current URL
    let url = browser.get_current_url().await?;
    println!("Current URL: {}", url);

    // Tab management
    browser.create_new_tab(Some("https://hackernews.com")).await?;
    let tabs = browser.get_tabs().await?;
    println!("Open tabs: {}", tabs.len());

    // Switch tabs
    browser.switch_to_tab(&tabs[0].target_id).await?;

    // Go back
    browser.go_back().await?;

    Ok(())
}
```

### Custom Actions

```rust
use browsing::tools::views::{ActionHandler, ActionParams, ActionContext, ActionResult};
use browsing::agent::views::ActionModel;
use browsing::error::Result;

struct CustomActionHandler;

#[async_trait::async_trait]
impl ActionHandler for CustomActionHandler {
    async fn execute(
        &self,
        params: &ActionParams<'_>,
        context: &mut ActionContext<'_>,
    ) -> Result<ActionResult> {
        // Custom action logic here
        Ok(ActionResult {
            extracted_content: Some("Custom result".to_string()),
            ..Default::default()
        })
    }
}

// Register custom action
agent.tools.register_custom_action(
    "custom_action".to_string(),
    "Description of custom action".to_string(),
    None,  // domains
    CustomActionHandler,
);
```

## 🏗️ Architecture

Browsing follows **SOLID principles** with a focus on separation of concerns, testability, and maintainability.

```
┌─────────────────────────────────────────────────────────────┐
│                         Agent                               │
│  ┌─────────────┬──────────────┬──────────────┬─────────┐  │
│  │   Browser   │ DOMProcessor │     LLM      │  Tools  │  │
│  │   (trait)   │    (trait)   │  (trait)     │         │  │
│  └──────┬──────┴──────┬───────┴──────┬───────┴────┬────┘  │
│         │             │              │            │       │
└─────────┼─────────────┼──────────────┼────────────┼───────┘
          │             │              │            │
    ┌─────▼──────┐ ┌───▼────┐   ┌────▼───┐  ┌────▼─────┐
    │  Browser   │ │DomSvc  │   │  LLM   │  │ Handlers │
    │            │ │        │   │        │  │          │
    │TabManager  │ │CDP     │   │Chat    │  │Navigation│
    │NavManager  │ │HTML    │   │Model   │  │Interaction│
    │Screenshot  │ │Tree    │   │        │  │Tabs      │
    │            │ │Builder │   │        │  │Content   │
    └────────────┘ └────────┘   └────────┘  └──────────┘
```

### Key Components

| Component | Responsibility | Trait-Based |
|-----------|---------------|-------------|
| **Agent** | Orchestrates browser, LLM, and DOM processing | Uses `BrowserClient`, `DOMProcessor` |
| **Browser** | Manages browser session and lifecycle | Implements `BrowserClient` |
| **DOMProcessor** | Extracts and serializes DOM | Implements `DOMProcessor` |
| **Tools** | Action registry and execution | Uses `BrowserClient` trait |
| **Handlers** | Specific action implementations | Use `ActionHandler` trait |

## 📁 Project Structure

```
browsing/
├── src/
│   ├── agent/              # Agent orchestration
│   │   ├── service.rs      # Main agent implementation
│   │   └── json_extractor.rs # JSON parsing utilities
│   ├── browser/            # Browser management
│   │   ├── session.rs      # Browser session (BrowserClient impl)
│   │   ├── tab_manager.rs  # Tab operations
│   │   ├── navigation.rs   # Navigation operations
│   │   ├── screenshot.rs   # Screenshot operations
│   │   ├── cdp.rs          # CDP WebSocket client
│   │   ├── launcher.rs     # Browser launcher
│   │   └── profile.rs      # Browser configuration
│   ├── dom/                # DOM processing
│   │   ├── processor.rs    # DOMProcessor trait impl
│   │   ├── serializer.rs   # LLM-ready serialization
│   │   ├── tree_builder.rs # DOM tree construction
│   │   ├── cdp_client.rs   # CDP wrapper for DOM
│   │   └── html_converter.rs # HTML to markdown
│   ├── tools/              # Action system
│   │   ├── service.rs      # Tools registry
│   │   ├── handlers/       # Action handlers
│   │   │   ├── navigation.rs
│   │   │   ├── interaction.rs
│   │   │   ├── tabs.rs
│   │   │   ├── content.rs
│   │   │   └── advanced.rs
│   │   └── params.rs       # Parameter extraction
│   ├── traits/             # Core trait abstractions
│   │   ├── browser_client.rs  # BrowserClient trait
│   │   └── dom_processor.rs   # DOMProcessor trait
│   ├── llm/                # LLM integration
│   │   └── base.rs         # ChatModel trait
│   ├── actor/              # Low-level interactions
│   │   ├── page.rs         # Page operations
│   │   ├── element.rs      # Element operations
│   │   └── mouse.rs        # Mouse interactions
│   ├── config/             # Configuration
│   ├── error/              # Error types
│   └── utils/              # Utilities
└── Cargo.toml
```

## 🎨 Design Principles

### Trait-Facing Design
- **BrowserClient** - Abstract browser operations for testing and alternative backends
- **DOMProcessor** - Pluggable DOM processing implementations
- **ActionHandler** - Extensible action system
- **ChatModel** - LLM provider abstraction

### Separation of Concerns
- **TabManager** - Tab operations (create, switch, close)
- **NavigationManager** - Navigation logic
- **ScreenshotManager** - Screenshot capture
- **Handlers** - Focused action implementations

### DRY (Don't Repeat Yourself)
- **ActionParams** - Reusable parameter extraction
- **JSONExtractor** - Centralized JSON parsing
- **SessionGuard** - Unified session access

### KISS (Keep It Simple, Stupid)
- Split complex methods into focused helpers
- Clear naming and single responsibility
- Minimal dependencies between modules

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_agent_workflow

# Run integration tests only
cargo test --test integration
```

### Test Coverage
- **74+ tests** across all modules
- **24 integration tests** for full workflow
- **50+ unit tests** for individual components
- **Mock LLM** for deterministic testing
- **Trait-based mocking** for browser/DOM components

## 🔧 Configuration

### Browser Profile

```rust
use browsing::BrowserProfile;

let profile = BrowserProfile {
    headless: true,
    browser_type: browsing::BrowserType::Chrome,
    user_data_dir: None,
    disable_gpu: true,
    ..Default::default()
};
```

### Agent Settings

```rust
use browsing::agent::views::AgentSettings;

let agent = Agent::new(...)
    .with_max_steps(50)
    .with_settings(AgentSettings {
        override_system_message: Some("Custom system prompt".to_string()),
        ..Default::default()
    });
```

## 📖 API Documentation

Generate and view API docs:

```bash
cargo doc --open
```
