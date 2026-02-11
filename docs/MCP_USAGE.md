# MCP Server Usage Guide

The `browsing-mcp` server is a **concise, lightweight** MCP interface: navigate, get links, follow links, list content (links + images), get/save content, screenshot (full or element). Lazy browser init. Parallel reads via RwLock.

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

## Available Tools (9)

### navigate
Navigate to a URL. **Parameters:** `url` (string, required)

### get_links
Get all links on the current page. **Parameters:** None  
**Returns:** `{ url, links: [{ index, href, text }], count }`

### follow_link
Follow a link by index (from get_links) or by URL. **Parameters:** `index` (number) or `url` (string)

### list_content
List available links and images with indices. **Parameters:** None  
**Returns:** `{ url, links: [...], images: [{ index, src, alt }] }`

### get_content
Get page text content. **Parameters:** `max_chars` (number, optional, default 100000)  
**Returns:** `{ url, text, length }`

### get_image
Capture image element by index (from list_content.images) as screenshot. **Parameters:** `index` (number, optional, default 0)  
**Returns:** Image content (base64 PNG)

### save_content
Save text or image to file. **Parameters:** `path` (string), `content_type` ("text" or "image"), `image_index` (number, for images)  
**Returns:** `{ success, path }`

### screenshot
Take screenshot: full page, or element by CSS selector. **Parameters:** `full_page` (bool), `selector` (string, e.g. ".sidebar", "#content"), `element_index` (number, when selector matches multiple)  
**Returns:** Image content (base64 PNG)

### generate_sitemap
Crawl from a URL, capture title and content preview per page, discover links. **Parameters:** `url` (required), `max_pages` (default 30), `max_depth` (default 3), `same_domain_only` (default true), `content_preview_chars` (default 500), `save_path` (optional file path), `delay_ms` (default 800)  
**Returns:** `{ success, total_pages, sitemap: { base_url, pages: [{ url, title, content_preview, links, depth }] }, saved_to }`

## Architecture

- **Lazy init**: Browser starts on first tool call
- **Parallelism**: `get_content` and `screenshot` can run concurrently (RwLock read); `navigate` holds write lock

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

### Typical workflow: rust-lang.org

1. `navigate` to https://www.rust-lang.org
2. `get_links` to list all links
3. `follow_link` with `index` to go to a specific link, or `url` for direct navigation
4. `list_content` to see links and images with indices
5. `get_content` for page text
6. `get_image` with `index` to capture an image, or `save_content` with `content_type: "image"` and `image_index` to save
7. `screenshot` with `full_page: true` for full page, or `selector: ".hero"` for a specific component
8. `generate_sitemap` with `url` to crawl and build sitemap; use `save_path` to write JSON to file

### With Custom MCP Client

```python
import mcp

client = mcp.Client()
client.connect("browsing-mcp")

# Navigate
client.call_tool("navigate", {"url": "https://www.rust-lang.org"})

# Get links
links = client.call_tool("get_links", {})

# Follow second link
client.call_tool("follow_link", {"index": 1})

# List content (links + images)
content = client.call_tool("list_content", {})

# Save page text
client.call_tool("save_content", {"path": "page.txt", "content_type": "text"})

# Screenshot specific element
client.call_tool("screenshot", {"selector": "main"})

# Generate sitemap from URL (crawl, capture content, save)
client.call_tool("generate_sitemap", {
    "url": "https://example.com",
    "max_pages": 20,
    "save_path": "sitemap.json"
})
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
3. **Links & Content**: Use `get_links` and `list_content` to discover links/images; use `follow_link` to navigate by index or URL
4. **Screenshots**: Full page or element by CSS selector (e.g. `selector: ".sidebar"`)
5. **Save Content**: Use `save_content` to save text or images (by index) to files

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
