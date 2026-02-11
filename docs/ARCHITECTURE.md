# Architecture Documentation

This document provides a comprehensive overview of the Browsing project architecture, design patterns, and testing strategy.

## Table of Contents

- [Overview](#overview)
- [Design Principles](#design-principles)
- [Architecture Layers](#architecture-layers)
- [Module Structure](#module-structure)
- [Key Traits](#key-traits)
- [Testing Strategy](#testing-strategy)
- [Data Flow](#data-flow)
- [Error Handling](#error-handling)

## Overview

Browsing is a Rust-based autonomous web browsing library for AI agents. It follows a **trait-based architecture** that enables:

- **Testability**: Mock implementations for unit testing
- **Extensibility**: Custom actions, browsers, and LLM providers
- **Maintainability**: Clear separation of concerns
- **Performance**: Efficient DOM processing and token usage

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         User Layer                          │
│  ┌─────────────┬──────────────┬──────────────┬─────────┐  │
│  │   CLI Tool  │  MCP Server  │   Library    │ Examples│  │
│  └──────┬──────┴──────┬───────┴──────┬───────┴────┬────┘  │
└─────────┼─────────────┼──────────────┼────────────┼───────┘
          │             │              │            │
┌─────────┼─────────────┼──────────────┼────────────┼───────┐
│         │      Agent Layer             │            │       │
│  ┌──────▼──────┬──────────────┬──────────────┬────▼─────┐  │
│  │   Agent     │ DOMProcessor │     LLM      │  Tools  │  │
│  │   Service   │   (trait)    │   (trait)    │ Service │  │
│  └──────┬──────┴──────┬───────┴──────┬───────┴────┬────┘  │
└─────────┼─────────────┼──────────────┼────────────┼───────┘
          │             │              │            │
┌─────────┼─────────────┼──────────────┼────────────┼───────┐
│         │    Browser Layer            │            │       │
│  ┌──────▼──────┬──────────────┬──────────────┬────▼─────┐  │
│  │  Browser    │  TabManager  │  Navigation  │  Actor  │  │
│  │  (trait)    │  Screenshot  │    Manager   │ System  │  │
│  └──────┬──────┴──────┬───────┴──────┬───────┴────┬────┘  │
└─────────┼─────────────┼──────────────┼────────────┼───────┘
          │             │              │            │
┌─────────┼─────────────┼──────────────┼────────────┼───────┐
│         │    Infrastructure Layer     │            │       │
│  ┌──────▼──────┬──────────────┬──────────────┬────▼─────┐  │
│  │  CDP Client│   Config     │   Error      │  Utils  │  │
│  │            │   Logging    │   Handling   │         │  │
│  └────────────┴──────────────┴──────────────┴──────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Trait-Based Design

All major components are defined as traits to enable:

- **Dependency Injection**: Inject mock implementations for testing
- **Multiple Implementations**: Support different browsers, LLMs, etc.
- **Polymorphism**: Use different implementations interchangeably

### 2. Separation of Concerns

Each module has a single, well-defined responsibility:

- **Agent**: Orchestrates the browsing task
- **Browser**: Manages browser lifecycle
- **DOM**: Extracts and processes page content
- **Tools**: Executes specific actions
- **Actor**: Low-level browser interactions

### 3. SOLID Principles

- **S**ingle Responsibility: Each struct/trait has one reason to change
- **O**pen/Closed: Open for extension (custom actions), closed for modification
- **L**iskov Substitution: Mock implementations can replace real ones
- **I**nterface Segregation: Small, focused traits
- **D**ependency Inversion: Depend on abstractions (traits), not concretions

### 4. DRY (Don't Repeat Yourself)

Shared utilities and traits reduce code duplication:

- **ActionParams**: Reusable parameter extraction
- **JSONExtractor**: Centralized JSON parsing with repair
- **SessionGuard**: Unified session access pattern

### 5. KISS (Keep It Simple, Stupid)

Complex functionality is broken into simple, focused components:

- **Managers**: Single-purpose managers (TabManager, NavigationManager, etc.)
- **Handlers**: Individual action handlers
- **Clear naming**: Self-documenting code

## Architecture Layers

### 1. User Interface Layer

Provides multiple ways to use the library:

#### CLI Tool (`src/bin/cli.rs`)
- Command-line interface for autonomous browsing
- Commands: `run`, `launch`, `connect`
- Configuration via environment variables and CLI arguments

#### MCP Server (`src/bin/mcp_server.rs`)
- Model Context Protocol server for AI assistants
- Tools: navigate, get_content, click, input, screenshot
- Prompts: browse_task template
- Resources: browser://current for page content

#### Library Interface (`src/lib.rs`)
- Public API exports
- Re-exports main types: Browser, Agent, Config
- Examples for common use cases

### 2. Agent Layer

Coordinates the autonomous browsing task.

#### Agent Service (`src/agent/service.rs`)
- **Purpose**: Orchestrates browsing tasks
- **Responsibilities**:
  - Task execution loop
  - LLM interaction
  - Action parsing and execution
  - History tracking
  - Usage tracking (tokens, cost)
- **Key Methods**:
  - `run()`: Execute the agent
  - `step()`: Execute one agent step
  - `build_messages()`: Construct LLM messages

#### DOM Processor (`src/dom/processor.rs`)
- **Purpose**: Extract and process page content
- **Trait**: `DOMProcessor`
- **Implementations**:
  - CDP-based DOM extraction
  - LLM-ready serialization
  - Selector map generation

#### LLM Integration (`src/llm/base.rs`)
- **Purpose**: Abstract LLM provider
- **Trait**: `ChatModel`
- **Supports**:
  - Chat completions
  - Streaming responses
  - Token usage tracking
  - Tool/function calling

#### Tools Service (`src/tools/service.rs`)
- **Purpose**: Registry and executor for actions
- **Components**:
  - Action model parsing
  - Handler registry
  - Action execution
  - Custom action support

### 3. Browser Layer

Manages browser lifecycle and interactions.

#### Browser Session (`src/browser/session.rs`)
- **Purpose**: Main browser implementation
- **Trait**: `BrowserClient`
- **Responsibilities**:
  - Browser lifecycle (start, stop)
  - Navigation
  - Tab management
  - Screenshot capture
  - State retrieval

#### Managers
- **TabManager** (`src/browser/tab_manager.rs`): Tab operations
- **NavigationManager** (`src/browser/navigation.rs`): Navigation operations
- **ScreenshotManager** (`src/browser/screenshot.rs`): Screenshot operations

#### Actor System (`src/actor/`)
Low-level browser interactions:
- **Page** (`page.rs`): Page-level operations
- **Element** (`element.rs`): Element operations
- **Mouse** (`mouse.rs`): Mouse interactions
- **Keyboard** (`keyboard.rs`): Keyboard input

### 4. Infrastructure Layer

Provides foundational utilities and services.

#### CDP Client (`src/browser/cdp.rs`)
- **Purpose**: Chrome DevTools Protocol client
- **Features**:
  - WebSocket communication
  - Command execution
  - Session management
  - Event handling

#### Configuration (`src/config.rs`)
- **Purpose**: Configuration management
- **Sources**:
  - Environment variables
  - .env files
  - Configuration structs

#### Error Handling (`src/error.rs`)
- **Purpose**: Centralized error types
- **Error Types**:
  - `BrowsingError`: Main error enum
  - Specific variants: Browser, Dom, Tool, LLM, Config

#### Utilities (`src/utils.rs`)
- **Purpose**: Shared utility functions
- **Features**:
  - URL extraction
  - Domain matching
  - Signal handling

## Module Structure

```
src/
├── agent/              # Agent orchestration
│   ├── service.rs      # Main agent implementation
│   ├── json_extractor.rs # JSON parsing utilities
│   ├── views.rs        # Data types
│   └── mod.rs
├── browser/            # Browser management
│   ├── session.rs      # Browser session (BrowserClient impl)
│   ├── tab_manager.rs  # Tab operations
│   ├── navigation.rs   # Navigation operations
│   ├── screenshot.rs   # Screenshot operations
│   ├── cdp.rs          # CDP WebSocket client
│   ├── launcher.rs     # Browser launcher
│   ├── profile.rs      # Browser configuration
│   ├── views.rs        # Data types
│   └── mod.rs
├── dom/                # DOM processing
│   ├── processor.rs    # DOMProcessor trait impl
│   ├── serializer.rs   # LLM-ready serialization
│   ├── tree_builder.rs # DOM tree construction
│   ├── cdp_client.rs   # CDP wrapper for DOM
│   ├── html_converter.rs # HTML to markdown
│   ├── views.rs        # Data types
│   └── mod.rs
├── tools/              # Action system
│   ├── service.rs      # Tools registry
│   ├── handlers/       # Action handlers
│   │   ├── navigation.rs
│   │   ├── interaction.rs
│   │   ├── tabs.rs
│   │   ├── content.rs
│   │   ├── advanced.rs
│   │   └── mod.rs
│   ├── views.rs        # Data types
│   └── mod.rs
├── traits/             # Core trait abstractions
│   ├── browser_client.rs # BrowserClient trait
│   ├── dom_processor.rs # DOMProcessor trait
│   └── mod.rs
├── llm/                # LLM integration
│   ├── base.rs         # ChatModel trait
│   └── mod.rs
├── actor/              # Low-level interactions
│   ├── page.rs         # Page operations
│   ├── element.rs      # Element operations
│   ├── mouse.rs        # Mouse interactions
│   ├── keyboard.rs     # Keyboard input
│   └── mod.rs
├── config/             # Configuration
│   └── mod.rs
├── error.rs            # Error types
├── logging.rs          # Logging setup
├── utils.rs            # Utilities
├── views.rs            # Shared data types
└── lib.rs              # Public API
```

## Key Traits

### BrowserClient

Abstracts browser operations for testing and alternative backends.

```rust
#[async_trait]
pub trait BrowserClient: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn navigate(&mut self, url: &str) -> Result<()>;
    async fn get_current_url(&self) -> Result<String>;
    async fn create_tab(&mut self, url: Option<&str>) -> Result<String>;
    async fn switch_to_tab(&mut self, target_id: &str) -> Result<()>;
    async fn close_tab(&mut self, target_id: &str) -> Result<()>;
    async fn get_tabs(&self) -> Result<Vec<TabInfo>>;
    fn get_page(&self) -> Result<Page>;
    async fn take_screenshot(&self, path: Option<&str>, full_page: bool) -> Result<Vec<u8>>;
    // ... more methods
}
```

### DOMProcessor

Abstracts DOM processing operations.

```rust
#[async_trait]
pub trait DOMProcessor: Send + Sync {
    async fn get_serialized_dom(&self) -> Result<SerializedDOMState>;
    async fn get_page_state_string(&self) -> Result<String>;
    async fn get_selector_map(&self) -> Result<HashMap<u32, DOMInteractedElement>>;
}
```

### ChatModel

Abstracts LLM provider interactions.

```rust
#[async_trait]
pub trait ChatModel: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatInvokeCompletion>;
    async fn chat_stream(&self, messages: Vec<ChatMessage>) -> Result<BoxStream<ChatInvokeCompletion>>;
}
```

### ActionHandler

Abstracts action implementations.

```rust
#[async_trait]
pub trait ActionHandler: Send + Sync {
    async fn execute(&self, params: &ActionParams<'_>, context: &mut ActionContext<'_>) -> Result<ActionResult>;
}
```

## Testing Strategy

### Test Organization

```
tests/
├── actor_test.rs              # Actor module tests (60+ tests)
├── browser_managers_test.rs   # Browser managers tests (50+ tests)
├── tools_handlers_test.rs     # Tools handlers tests (50+ tests)
├── agent_service_test.rs      # Agent service tests (40+ tests)
├── traits_test.rs             # Traits tests (30+ tests)
├── utils_test.rs              # Utilities tests (50+ tests)
├── browser_test.rs            # Browser integration tests
├── dom_test.rs                # DOM integration tests
├── agent_test.rs              # Agent integration tests
├── tools_test.rs              # Tools integration tests
├── integration_test.rs        # Full workflow integration tests
├── integration_workflow_test.rs # End-to-end workflow tests
└── ... (additional test files)
```

### Test Categories

#### 1. Unit Tests
- **Purpose**: Test individual functions and methods
- **Examples**:
  - Key code mapping
  - URL parsing
  - Domain matching
  - Data structure validation

#### 2. Integration Tests
- **Purpose**: Test multiple components working together
- **Examples**:
  - Browser navigation
  - DOM extraction
  - Agent execution flow
  - Tool execution

#### 3. Trait Tests
- **Purpose**: Test trait implementations and mock objects
- **Examples**:
  - Mock BrowserClient
  - Mock DOMProcessor
  - Trait method validation

#### 4. Property-Based Tests
- **Purpose**: Verify invariants across many inputs
- **Examples**:
  - URL encoding/decoding
  - Domain pattern matching
  - Data structure consistency

### Mock Implementations

The project provides mock implementations for testing:

```rust
struct MockBrowserClient {
    started: bool,
    current_url: String,
    navigation_count: AtomicUsize,
}

#[async_trait]
impl BrowserClient for MockBrowserClient {
    async fn start(&mut self) -> Result<()> {
        self.started = true;
        Ok(())
    }
    // ... other methods
}
```

### Test Coverage Summary

| Module | Tests | Coverage |
|--------|-------|----------|
| Actor | 60+ | Keyboard, Mouse, Page, Element operations |
| Browser | 50+ | Navigation, Screenshot, Tab management |
| Tools | 50+ | All action handlers |
| Agent | 40+ | Execution logic, history, usage tracking |
| Traits | 30+ | BrowserClient, DOMProcessor implementations |
| Utils | 50+ | URL extraction, domain matching, signals |
| Integration | 50+ | End-to-end workflows |
| **Total** | **200+** | |

## Data Flow

### Agent Execution Flow

```
┌─────────────┐
│   Task      │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Agent     │
│  .run()     │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────┐
│ For each step (max_steps):  │
└────────────┬────────────────┘
             │
             ▼
    ┌────────────────┐
    │ Get Page State │
    │  (DOMProcessor)│
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ Build Messages │
    │  (with state)  │
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ Call LLM       │
    │  (ChatModel)   │
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ Parse Action   │
    │  (JSONExtractor)│
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ Execute Action │
    │  (Tools)       │
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ Track History  │
    │  & Usage       │
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ Check Done     │
    │  Condition     │
    └────────────────┘
```

### Action Execution Flow

```
┌─────────────┐
│  Action     │
│  Parameters │
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│ Get Handler      │
│ (from registry)  │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Execute Handler  │
│  - BrowserClient │
│  - DOMProcessor  │
│  - ActionContext │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Return Result    │
│  - Content       │
│  - Memory        │
│  - State         │
└──────────────────┘
```

## Error Handling

### Error Hierarchy

```
BrowsingError
├── Browser(String)      # Browser-related errors
├── Dom(String)          # DOM processing errors
├── Tool(String)         # Tool/action errors
├── LLM(String)          # LLM provider errors
├── Config(String)       # Configuration errors
└── Other(String)        # Other errors
```

### Error Propagation

Errors are propagated using Rust's `Result<T>` type and the `?` operator:

```rust
async fn navigate(&mut self, url: &str) -> Result<()> {
    self.validate_url(url)?;     // Returns early on error
    self.cdp_navigate(url).await?; // Propagates CDP errors
    Ok(())
}
```

### Error Recovery

The agent implements several recovery strategies:

1. **JSON Repair**: Repairs malformed JSON from LLMs
2. **Retry Logic**: Retries failed actions
3. **Graceful Degradation**: Continues on non-critical errors
4. **Error Logging**: Logs errors for debugging

## Design Patterns Used

### 1. Strategy Pattern
- **Traits**: BrowserClient, DOMProcessor, ChatModel
- **Purpose**: Enable different implementations

### 2. Builder Pattern
- **Types**: Browser, Agent, Config
- **Purpose**: Fluent configuration

### 3. Factory Pattern
- **Types**: BrowserLauncher, Handlers
- **Purpose**: Create instances with context

### 4. Repository Pattern
- **Types**: TabManager, NavigationManager, ScreenshotManager
- **Purpose**: Manage domain operations

### 5. Observer Pattern
- **Types**: CDP event handling
- **Purpose**: React to browser events

### 6. Command Pattern
- **Types**: ActionHandler, Tools
- **Purpose**: Encapsulate actions as objects

## Performance Considerations

### Token Optimization
- **Selective Extraction**: Only extract interactive elements
- **Content Pruning**: Limit text content length
- **Tree Pruning**: Remove irrelevant DOM nodes

### Caching
- **CDP Sessions**: Reuse sessions for multiple commands
- **Selector Maps**: Cache element mappings
- **DOM State**: Cache serialized state

### Concurrency
- **Async/Await**: Non-blocking I/O operations
- **Tokio Runtime**: Efficient async runtime
- **Parallel Requests**: Concurrent CDP commands

## Security Considerations

### Input Validation
- **URL Validation**: Validate and sanitize URLs
- **Parameter Validation**: Validate action parameters
- **File Path Validation**: Validate file paths for screenshots

### Sandboxing
- **Browser Flags**: Use Chrome's sandbox flags
- **User Data Dir**: Isolated browser profile
- **No Root**: Don't run as root

### Data Privacy
- **Local Processing**: Process data locally
- **No Telemetry**: No data collection
- **Configurable LLM**: User controls LLM provider

## Future Enhancements

### Planned Features
- [ ] Performance benchmarks
- [ ] More browser backends (Firefox, Safari)
- [ ] Enhanced DOM processing (paint order)
- [ ] Distributed agent execution
- [ ] Advanced error recovery

### Community Contributions
We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## References

- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
