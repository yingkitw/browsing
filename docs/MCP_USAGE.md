# MCP Server Usage Guide

The `browsing-mcp` server provides a Model Context Protocol interface for AI assistants to control web browsers.

## What is MCP?

Model Context Protocol (MCP) is a standard for connecting AI assistants to external tools and data sources. The browsing MCP server exposes web browsing capabilities as tools that can be used by Claude, GPT-4, or other AI assistants.

## Installation

```bash
cargo build --release --bin browsing-mcp
```

## Running the Server

```bash
./target/release/browsing-mcp
```

The server communicates via stdio and is designed to be used with MCP clients.

## Configuration with Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "browsing": {
      "command": "/path/to/browsing/target/release/browsing-mcp",
      "env": {
        "BROWSER_USE_HEADLESS": "true",
        "LLM_API_KEY": "your_api_key"
      }
    }
  }
}
```

## Available Tools

### navigate

Navigate to a URL.

**Parameters:**
- `url` (string, required): URL to navigate to

**Example:**
```json
{
  "name": "navigate",
  "arguments": {
    "url": "https://example.com"
  }
}
```

### get_content

Get the current page content, including DOM structure and metadata.

**Parameters:** None

**Returns:**
```json
{
  "success": true,
  "content": "...",
  "url": "https://example.com",
  "title": "Example Domain"
}
```

### click

Click an element by its index.

**Parameters:**
- `index` (number, required): Element index from the DOM

**Example:**
```json
{
  "name": "click",
  "arguments": {
    "index": 42
  }
}
```

### input

Input text into an element.

**Parameters:**
- `index` (number, required): Element index
- `text` (string, required): Text to input

**Example:**
```json
{
  "name": "input",
  "arguments": {
    "index": 15,
    "text": "search query"
  }
}
```

### screenshot

Take a screenshot of the current page.

**Parameters:** None

**Returns:**
```json
{
  "success": true,
  "screenshot": "base64_encoded_image",
  "format": "base64"
}
```

## Available Prompts

### browse_task

Template for autonomous browsing tasks.

**Arguments:**
- `task` (string, required): Task description

**Example:**
```json
{
  "name": "browse_task",
  "arguments": {
    "task": "Find the latest news about AI"
  }
}
```

## Available Resources

### browser://current

Access the current browser page content.

**URI:** `browser://current`  
**MIME Type:** `text/html`

## Environment Variables

```bash
# Browser settings
BROWSER_USE_HEADLESS=true
BROWSER_USE_USER_DATA_DIR=/path/to/user/data

# LLM settings (if needed)
LLM_API_KEY=your_api_key
LLM_MODEL=ibm/granite-4-h-small
```

## Usage Examples

### With Claude Desktop

Once configured, you can ask Claude:

```
"Navigate to example.com and click on the first link"
```

Claude will use the MCP tools to:
1. Call `navigate` with the URL
2. Call `get_content` to see the page
3. Call `click` with the appropriate element index

### With Custom MCP Client

```python
import mcp

client = mcp.Client()
client.connect("browsing-mcp")

# Navigate
result = client.call_tool("navigate", {"url": "https://example.com"})

# Get content
content = client.call_tool("get_content", {})

# Click element
client.call_tool("click", {"index": 5})
```

## Architecture

The MCP server:
1. Launches a browser instance on first tool call
2. Maintains the browser session across tool calls
3. Exposes browser capabilities as MCP tools
4. Returns structured results in JSON format

## Tips

1. **Lazy Loading**: Browser launches only when first tool is called
2. **Session Persistence**: Browser stays open across multiple tool calls
3. **Element Indices**: Use `get_content` to see element indices before clicking
4. **Screenshots**: Use for visual verification of page state
5. **Error Handling**: All tools return structured error information

## Troubleshooting

### Browser Not Launching

Check environment variables and ensure Chrome/Chromium is installed:

```bash
export BROWSER_USE_HEADLESS=true
which google-chrome
```

### Connection Issues

Verify the MCP server is running and stdio communication is working:

```bash
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | ./browsing-mcp
```

### Tool Errors

Enable verbose logging:

```bash
RUST_LOG=browsing=debug ./browsing-mcp
```
