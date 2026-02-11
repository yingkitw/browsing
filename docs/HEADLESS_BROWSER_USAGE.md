# Browser-Use-RS Usage Examples

This guide demonstrates how to use browser-use-rs with headless browsers for automation.

## Quick Start

### Basic Setup

```rust
use browser_use::agent::service::Agent;
use browser_use::browser::{Browser, BrowserProfile};
use browser_use::agent::views::AgentSettings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create browser with headless configuration
    let profile = BrowserProfile {
        headless: Some(true), // Run in headless mode (important for CI/automation)
        user_data_dir: None, // Use temporary directory
        allowed_domains: None, // Allow all domains (configure as needed)
        downloads_path: Some("/tmp/browser_downloads".into()),
    };
    
    let browser = Browser::new(profile);
    
    // Configure your LLM (example with mock or real implementation)
    let llm = create_llm_client().await?;
    
    // Create agent with task
    let task = "Navigate to https://example.com and extract the main heading text";
    
    let agent = Agent::new(task, browser, llm)
        .with_max_steps(10)
        .with_settings(AgentSettings {
            use_vision: browser_use::agent::views::VisionMode::Auto,
            max_failures: 3,
            use_thinking: true,
            ..Default::default()
        });
    
    // Run agent to completion
    let history = agent.run().await?;
    
    println!("Agent completed successfully!");
    println!("Total steps: {}", history.number_of_steps());
    println!("Duration: {:.2}s", history.total_duration_seconds());
    
    // Browser automatically cleans up when dropped
    // Or explicitly: browser.stop().await?;
    
    Ok(())
}
```

## Advanced Configuration

### Using Existing Browser Instance

```rust
// Connect to an already running browser via CDP
let mut browser = Browser::new(BrowserProfile::default())
    .with_cdp_url("ws://localhost:9222".to_string());

browser.start().await?;
browser.navigate("https://example.com").await?;
```

### Custom Browser Executable

```rust
use std::path::PathBuf;

let profile = BrowserProfile {
    headless: Some(true),
    user_data_dir: Some("/tmp/chrome_profile".into()),
    allowed_domains: None,
    downloads_path: Some("/tmp/downloads".into()),
};

let mut browser = Browser::new(profile);
// browser.with_executable_path(PathBuf::from("/usr/bin/google-chrome-stable"));
browser.start().await?;
```

### Domain Restrictions

```rust
let profile = BrowserProfile {
    headless: Some(true),
    user_data_dir: None,
    allowed_domains: Some(vec![
        "example.com".to_string(),
        "subdomain.example.com".to_string(),
    ]),
    downloads_path: None,
};

let browser = Browser::new(profile);
// Browser will only allow navigation to specified domains
```

## Error Handling

The browser use cases should include proper error handling:

```rust
use browser_use::error::BrowserUseError;

match agent.run().await {
    Ok(history) => {
        println!("Success: {} steps", history.number_of_steps());
    }
    Err(BrowserUseError::Browser(msg)) => {
        eprintln!("Browser error: {}", msg);
        // Try to fallback or retry
    }
    Err(BrowserUseError::Llm(msg)) => {
        eprintln!("LLM error: {}", msg);
        // Check API keys, model availability
    }
    Err(e) => {
        eprintln!("Other error: {:?}", e);
    }
}
```

## Headless Browser Requirements

For headless mode to work, you need:

1. **Chrome/Chromium** installed and accessible in PATH
2. **Chrome DevTools Protocol** (CDP) support (included with Chrome)
3. **Permissions** to launch processes
4. **Sufficient system resources** (RAM for browser instances)

### Installing Chrome

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y google-chrome-stable
# or
sudo apt install -y chromium-browser
```

#### macOS
```bash
# Using Homebrew
brew install --cask google-chrome

# Or download from https://www.google.com/chrome/
```

#### Windows
Download from https://www.google.com/chrome/ and install to default location

### Verifying Installation

```bash
# Check if Chrome is in PATH
google-chrome --version
# or
chromium-browser --version
```

## CI/CD Considerations

### GitHub Actions Example

```yaml
name: Browser Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install dependencies
      run: |
        sudo apt update
        sudo apt install -y google-chrome-stable
    
    - name: Run tests
      run: cargo test --test integration_workflow_test
      env:
        DISPLAY: ':99'
```

### Docker Example

```dockerfile
FROM rust:1.70

# Install Chrome
RUN wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | apt-key add - && \
    echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google-chrome.list && \
    apt-get update && \
    apt-get install -y google-chrome-stable

WORKDIR /app
COPY . .
RUN cargo build --release

CMD ["cargo", "run", "--release"]
```

## Troubleshooting

### "No browser executable found"

**Problem**: Browser cannot find Chrome/Chromium executable.

**Solutions**:
1. Install Chrome/Chromium
2. Add Chrome to system PATH
3. Use `with_executable_path()` to specify location:
   ```rust
   browser.with_executable_path(PathBuf::from("/path/to/chrome"));
   ```

### CDP Connection Issues

**Problem**: Browser starts but CDP connection fails.

**Solutions**:
1. Ensure Chrome wasn't launched with `--disable-dev-shm-usage`
2. Check if another instance is using the same debug port
3. Try different user data directory: `user_data_dir: Some("/tmp/chrome_test".into())`

### Headless Mode Issues

**Problem**: Headless mode doesn't work on some systems.

**Solutions**:
1. On Linux, ensure Xvfb (virtual display) is running:
   ```bash
   Xvfb :99 -screen 0 1024x768x24 &
   export DISPLAY=:99
   ```
2. On macOS, try disabling sandbox flags:
   ```rust
   let launcher = BrowserLauncher::new(profile);
   // Configure with appropriate flags if needed
   ```

### Memory Issues

**Problem**: Browser consumes too much memory.

**Solutions**:
1. Use lighter weight Chromium builds
2. Limit number of concurrent browser instances
3. Explicitly close browsers when done: `browser.stop().await?`

## Security Considerations

### File Uploads

File upload paths are automatically validated for security:

```rust
// These are REJECTED by default
"../../../etc/passwd"        // Path traversal
"~/.ssh/id_rsa"             // Home directory access
"/etc/shadow"                // System files

// This is ACCEPTED
"/tmp/safe_file.txt"         // Temporary directory
"./local_file.txt"           // Relative path
```

### JavaScript Execution

JavaScript code is automatically sanitized:

```rust
// These are BLOCKED
"document.cookie = 'hacked'"
"localStorage.setItem('token', 'stolen')"
"eval('malicious code')"

// These are ALLOWED
"document.title = 'Test'"
"console.log('debug message')"
"return document.body.innerHTML"
```

## Next Steps

1. **Start Simple**: Use the basic setup example above
2. **Configure LLM**: Replace mock LLM with your actual LLM implementation
3. **Customize Browser**: Adjust browser profile settings for your use case
4. **Add Error Handling**: Implement robust error handling for production
5. **Test Thoroughly**: Use the provided tests as examples for your own tests

For more examples, see the `tests/` directory in the repository.