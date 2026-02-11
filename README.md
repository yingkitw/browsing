# Browsing

**Lightweight MCP/API for browser automation**

A concise MCP server and Rust library: **navigate**, **get_links**, **follow_link**, **list_content** (links+images), **get_content**, **get_image**, **save_content**, **screenshot** (full or element). Lazy browser init. Parallel reads via RwLock.

## 🎯 Usage Modes

1. **🔌 MCP Server** (primary) - `navigate`, `get_links`, `follow_link`, `list_content`, `get_content`, `get_image`, `save_content`, `screenshot`, `generate_sitemap` tools for AI assistants
2. **⌨️ CLI** - Autonomous browsing tasks
3. **📦 Library** - Full agent system with LLM, custom actions

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
- Comprehensive test coverage (200+ tests)

## 📦 Installation

### As a Library

```toml
[dependencies]
browsing = "0.1"
tokio = { version = "1.40", features = ["full"] }
```

### As a CLI Tool

```bash
cargo install --path . --bin browsing
```

### As an MCP Server

```bash
cargo build --release --bin browsing-mcp
```

## 🚀 Quick Start

### 1️⃣ CLI Usage

```bash
# Run an autonomous browsing task
browsing run "Find the latest news about AI" --url https://news.ycombinator.com --headless

# Launch a browser and get CDP URL
browsing launch --headless

# Connect to existing browser
browsing connect ws://localhost:9222/devtools/browser/abc123
```

**📖 [Full CLI Documentation](docs/CLI_USAGE.md)**

### 2️⃣ MCP Server Usage

Configure in Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "browsing": {
      "command": "/path/to/browsing/target/release/browsing-mcp",
      "env": {
        "BROWSER_USE_HEADLESS": "true"
      }
    }
  }
}
```

Then ask Claude:
```
"Navigate to rust-lang.org, get the links, follow the second link, and screenshot the main content area"
```

**📖 [Full MCP Documentation](docs/MCP_USAGE.md)**

### 3️⃣ Library Usage

```rust
use anyhow::Result;
use browsing::{Browser, Config};

#[tokio::main]
async fn main() -> Result<()> {
    browsing::init();
    
    let config = Config::from_env();
    let browser = Browser::launch(config.browser_profile).await?;
    
    browser.navigate("https://example.com").await?;
    
    let state = browser.get_browser_state_summary(true).await?;
    println!("Title: {}", state.title);
    
    Ok(())
}
```

**📖 [Full Library Documentation](docs/LIBRARY_USAGE.md)**

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

### Content Download

```rust
use browsing::{Browser, BrowserProfile};
use browsing::dom::DOMProcessorImpl;
use browsing::traits::DOMProcessor;

#[tokio::main]
async fn main() -> browsing::error::Result<()> {
    let mut browser = Browser::new(BrowserProfile::default());
    browser.start().await?;

    // Navigate to website
    browser.navigate("https://www.ibm.com").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Extract content
    let cdp_client = browser.get_cdp_client()?;
    let session_id = browser.get_session_id()?;
    let target_id = browser.get_current_target_id()?;

    let dom_processor = DOMProcessorImpl::new()
        .with_cdp_client(cdp_client, session_id)
        .with_target_id(target_id);

    let page_content = dom_processor.get_page_state_string().await?;
    println!("Extracted {} bytes of content", page_content.len());

    // Save to file
    std::fs::write("ibm_content.txt", page_content)?;
    Ok(())
}
```

**Run this example:**
```bash
cargo run --example ibm_content_download
```

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
- **317 tests** across all modules (all passing)
- **50+ integration tests** for full workflow
- **150+ unit tests** for individual components
- **Test files**:
  - [actor_test.rs](tests/actor_test.rs) - Page, Element, Mouse, Keyboard operations (23 passed)
  - [browser_managers_test.rs](tests/browser_managers_test.rs) - Navigation, Screenshot, Tab managers
  - [tools_handlers_test.rs](tests/tools_handlers_test.rs) - All action handlers (49 passed)
  - [agent_service_test.rs](tests/agent_service_test.rs) - Agent execution logic (32 passed)
  - [agent_execution_test.rs](tests/agent_execution_test.rs) - Agent workflow tests (11 passed)
  - [traits_test.rs](tests/traits_test.rs) - BrowserClient, DOMProcessor traits (24 passed)
  - [utils_test.rs](tests/utils_test.rs) - URL extraction, signal handling (49 passed)
- **Mock implementations** for deterministic testing
- **Trait-based mocking** for browser/DOM components

## ⚠️ Data Retention Policy

### Browser Data is NEVER Deleted

**IMPORTANT**: The `browsing` library **never deletes browser data** for safety reasons.

#### What This Means:

| Data Type | Behavior |
|-----------|----------|
| **Bookmarks** | Never deleted |
| **History** | Never deleted |
| **Cookies** | Never deleted |
| **Passwords** | Never deleted |
| **Extensions** | Never deleted |
| **Cache** | Never deleted |
| **Temp Directories** | Never deleted (left in `/tmp/`) |

#### Why This Policy Exists:

1. **User Safety**: Users may specify a custom `user_data_dir` pointing to their real browser profile
2. **Catastrophe Prevention**: Accidentally deleting a user's real browser data (bookmarks, history, passwords) would be devastating
3. **Debugging**: Leaving temp directories allows inspection after crashes or failures
4. **User Control**: Users are responsible for managing their own browser data

#### How It Works:

When no `user_data_dir` is specified:
```rust
let profile = BrowserProfile {
    user_data_dir: None,  // Uses temp directory: /tmp/browser-use-1738369200000/
    ..Default::default()
};
```

When `browser.stop()` is called:
- ✅ Browser process is killed
- ✅ In-memory state is cleared
- ❌ User data directory is **NOT** deleted

#### Managing Temporary Data:

Users are responsible for cleanup:

```bash
# List browser temp directories
ls -la /tmp/browser-use-*

# Delete old temp directories (optional, manual cleanup)
rm -rf /tmp/browser-use-1738369200000/
```

#### Using a Custom Data Directory:

```rust
let profile = BrowserProfile {
    user_data_dir: Some("/path/to/custom/profile".into()),
    ..Default::default()
};
```

**Warning**: If you point to your real browser profile, the library will NOT protect it. You're responsible for that directory.



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
