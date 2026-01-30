# AGENTS.md

This file provides guidelines and commands for agentic coding tools working in this repository.

## Project Overview

This is a Rust implementation of Browser-Use, a library for making websites accessible to AI agents. It provides autonomous web automation through browser control, DOM extraction, and LLM integration.

## Build and Test Commands

### Building the Project
```bash
cargo build
cargo build --release  # Optimized release build
```

### Running Tests
```bash
# Run all tests
cargo test

# Run a single test (replace with actual test name)
cargo test test_name_here

# Run tests in a specific file
cargo test --lib browser::session
cargo test --test integration_test

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

### Code Quality Checks
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run lints
cargo clippy

# Check for unused dependencies
cargo machete  # if installed
```

### Documentation
```bash
# Generate documentation
cargo doc --open
```

## Code Style Guidelines

### Rust Edition
- Uses Rust 2024 edition
- Follow idiomatic Rust patterns and conventions

### Module Structure
- Each major component has its own module under `src/`
- Public modules use `pub mod` declarations in `mod.rs` files
- Re-export important types at the module level and in `src/lib.rs`

### Error Handling
- Use the `Result<T>` type alias defined in `error.rs` for all public functions
- Error types are defined using `thiserror` for proper error descriptions
- Use `?` operator for error propagation
- Avoid panic! in library code - handle errors gracefully

### Async Code
- Uses `tokio` as the async runtime
- Async functions use `async fn` syntax
- Use `.await` for async operations
- Prefer returning `Result<T>` from async functions

### Import Organization
- Standard library imports first, grouped by category
- External crates next, grouped by crate
- Internal modules last, grouped by module
- Use `use crate::` for internal imports
- Example:
```rust
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::browser::session::Browser;
use crate::error::Result as BrowserUseResult;
```

### Naming Conventions
- Functions and methods: snake_case
- Types and structs: PascalCase
- Constants: SCREAMING_SNAKE_CASE
- Private fields: snake_case with leading underscore if needed
- Modules: snake_case (directory and file names)

### Documentation
- All public modules, functions, and structs should have doc comments
- Use `//!` for module documentation
- Use `///` for item documentation
- Include examples in documentation where appropriate
- Document error conditions and behavior

### Testing
- Unit tests go in the same module with `#[cfg(test)]`
- Integration tests go in the `tests/` directory
- Test functions should use descriptive names starting with `test_`
- Use `tokio::test` for async tests
- Test fixtures should be minimal and focused

### Structs and Enums
- Use `#[derive(Debug)]` for most types
- Add `Clone`, `Copy`, `PartialEq`, `Eq` as appropriate
- Use `#[derive(Serialize, Deserialize)]` for data structures
- Enum variants should be descriptive
- Use `thiserror` for error enums

### Traits
- Define traits with clear purposes
- Use async traits with `async-trait` crate when needed
- Implement traits for external types when useful
- Document trait contract and invariants

### Dependencies
- Check `Cargo.toml` before adding new dependencies
- Prefer commonly used, well-maintained crates
- Keep dependencies minimal
- Use feature flags appropriately

### Performance
- Avoid unnecessary allocations
- Use references instead of copies when possible
- Consider using `Arc` for shared data in async contexts
- Profile performance-critical code

### Code Organization
- Keep functions focused and small (ideally under 50 lines)
- Extract helper functions when needed
- Group related functionality together
- Use consistent patterns across modules

## Important Notes

1. **CDP Integration**: The project uses Chrome DevTools Protocol (CDP) for browser automation
2. **LLM Integration**: Primary LLM integration is with Watsonx, but the code supports other LLMs
3. **Token Tracking**: The project tracks LLM token usage - maintain this in all changes
4. **Browser Lifecycle**: Ensure proper cleanup of browser resources in all code paths
5. **Error Recovery**: The agent system includes error recovery mechanisms
6. **Serialization**: Many data structures are serialized for LLM communication

## Common Patterns

### Async Resource Management
```rust
pub async fn example_function() -> Result<()> {
    let resource = create_resource().await?;
    
    // Use the resource
    
    Ok(()) // Resource is automatically dropped
}
```

### Error Handling
```rust
pub fn example_function() -> Result<OutputType> {
    let value = some_operation()?;
    let result = another_operation(value)?;
    
    Ok(result)
}
```

### Module Re-exports
```rust
pub mod service;
pub mod views;

pub use service::MainType;
pub use views::*;
```