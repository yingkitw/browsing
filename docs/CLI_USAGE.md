# CLI Usage Guide

The `browsing` CLI provides a command-line interface for autonomous web browsing.

## Installation

```bash
cargo install --path .
```

## Commands

### Run an Autonomous Task

Execute an autonomous browsing task with an AI agent:

```bash
browsing run "Find the latest news about AI" --url https://news.ycombinator.com
```

**Options:**
- `--url <URL>`: Starting URL (optional)
- `--max-steps <N>`: Maximum number of steps (default: 100)
- `--headless`: Run browser in headless mode
- `--vision`: Enable vision capabilities
- `--config <PATH>`: Path to configuration file
- `--verbose`: Enable verbose logging

**Examples:**

```bash
# Run with headless mode
browsing run "Search for Rust tutorials" --headless

# Run with custom max steps
browsing run "Find product prices" --url https://example.com --max-steps 50

# Run with vision enabled
browsing run "Analyze this webpage" --vision --url https://example.com
```

### Launch a Browser

Launch a browser instance and get the CDP URL:

```bash
browsing launch
```

**Options:**
- `--headless`: Run browser in headless mode
- `--user-data-dir <PATH>`: User data directory

**Example:**

```bash
browsing launch --headless
# Output: Browser launched successfully!
#         CDP URL: ws://localhost:9222/devtools/browser/...
```

### Connect to Existing Browser

Connect to an already running browser via CDP:

```bash
browsing connect ws://localhost:9222/devtools/browser/abc123
```

## Configuration

### Environment Variables

Create a `.env` file or set environment variables:

```bash
# Browser settings
BROWSER_USE_HEADLESS=true
BROWSER_USE_USER_DATA_DIR=/path/to/user/data
BROWSER_USE_ALLOWED_DOMAINS=example.com,test.com
BROWSER_USE_DOWNLOADS_PATH=/path/to/downloads

# LLM settings
LLM_API_KEY=your_api_key
LLM_MODEL=ibm/granite-4-h-small
LLM_TEMPERATURE=0.7
LLM_MAX_TOKENS=2000

# Agent settings
BROWSER_USE_MAX_STEPS=100
BROWSER_USE_VISION=false
```

### Configuration File

Create a JSON configuration file:

```json
{
  "browser_profile": {
    "headless": true,
    "user_data_dir": "/path/to/user/data",
    "allowed_domains": ["example.com"],
    "downloads_path": "/path/to/downloads"
  },
  "llm": {
    "api_key": "your_api_key",
    "model": "ibm/granite-4-h-small",
    "temperature": 0.7,
    "max_tokens": 2000
  },
  "agent": {
    "max_steps": 100,
    "use_vision": false
  }
}
```

Use with `--config`:

```bash
browsing run "Task description" --config config.json
```

## LLM Integration

The CLI requires an LLM implementation. Implement the `ChatModel` trait for your LLM provider.

**Example with watsonx:**

```rust
use watsonx_rs::WatsonxClient;
use browsing::ChatModel;

// Implement ChatModel for your LLM
// See examples/library_usage.rs for details
```

Set environment variables:

```bash
export LLM_API_KEY=your_watsonx_api_key
export LLM_MODEL=ibm/granite-4-h-small
```

## Tips

1. **Headless Mode**: Use `--headless` for server environments
2. **Vision**: Enable `--vision` for tasks requiring visual understanding
3. **Max Steps**: Adjust `--max-steps` based on task complexity
4. **Logging**: Use `--verbose` for debugging
5. **Configuration**: Use config files for consistent settings across runs
