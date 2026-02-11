//! Lightweight MCP server: browse, navigate, get links, follow links,
//! list content (links/images), get/save content, screenshot (full or element).
//! Lazy browser init. RwLock enables parallel operations.

use browsing::{config::Config, Browser};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Content, ErrorData as McpError, *},
    tool, tool_handler, tool_router,
    ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct BrowsingService {
    browser: Arc<RwLock<Option<Browser>>>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NavigateParams {
    #[schemars(description = "URL to navigate to")]
    url: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FollowLinkParams {
    #[schemars(description = "Index of link from get_links (0-based)")]
    index: Option<u32>,
    #[schemars(description = "Or specify URL directly")]
    url: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetContentParams {
    #[schemars(description = "Max characters to return")]
    max_chars: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SaveContentParams {
    #[schemars(description = "Path to save file")]
    path: String,
    #[schemars(description = "Content type: text, or image index from list_content")]
    content_type: String,
    #[schemars(description = "For image: index from list_content.images")]
    image_index: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetImageParams {
    #[schemars(description = "Index from list_content.images (0-based)")]
    index: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ScreenshotParams {
    #[schemars(description = "Capture full scrollable page")]
    full_page: Option<bool>,
    #[schemars(description = "CSS selector for element (e.g. '.sidebar', '#content', 'a')")]
    selector: Option<String>,
    #[schemars(description = "If selector matches multiple elements, use this index")]
    element_index: Option<u32>,
}

#[tool_router]
impl BrowsingService {
    fn new() -> Self {
        Self {
            browser: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    async fn ensure_browser(&self) -> Result<(), McpError> {
        let mut g = self.browser.write().await;
        if g.is_none() {
            let profile = Config::from_env().browser_profile;
            let mut browser = Browser::new(profile);
            browser.start().await.map_err(|e| {
                McpError::internal_error(format!("Browser start failed: {}", e), None)
            })?;
            *g = Some(browser);
        }
        Ok(())
    }

    #[tool(description = "Navigate to a URL")]
    async fn navigate(&self, Parameters(p): Parameters<NavigateParams>) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let mut g = self.browser.write().await;
        let browser = g.as_mut().ok_or_else(|| McpError::internal_error("No browser", None))?;
        browser.navigate(&p.url).await.map_err(|e| {
            McpError::internal_error(format!("Navigate failed: {}", e), None)
        })?;
        Ok(CallToolResult::structured(serde_json::json!({
            "success": true,
            "url": p.url
        })))
    }

    #[tool(description = "Get all links on the current page (index, href, text)")]
    async fn get_links(&self) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let g = self.browser.read().await;
        let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
        let page = browser.get_page().map_err(|e| {
            McpError::internal_error(format!("Get page failed: {}", e), None)
        })?;
        let script = r#"
            (function() {
                const links = Array.from(document.querySelectorAll('a[href]'))
                    .filter(a => a.href && !a.href.startsWith('javascript:'))
                    .map((a, i) => ({
                        index: i,
                        href: a.href,
                        text: (a.textContent || '').trim().slice(0, 150)
                    }));
                return JSON.stringify(links);
            })()
        "#;
        let result = page.evaluate(script).await.unwrap_or_else(|_| "[]".to_string());
        let links: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap_or_default();
        let url = browser.get_current_url().await.unwrap_or_default();
        drop(g);
        Ok(CallToolResult::structured(serde_json::json!({
            "url": url,
            "links": links,
            "count": links.len()
        })))
    }

    #[tool(description = "Follow a link by index (from get_links) or by URL")]
    async fn follow_link(&self, Parameters(p): Parameters<FollowLinkParams>) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let url = if let Some(u) = p.url {
            u
        } else if let Some(idx) = p.index {
            let g = self.browser.read().await;
            let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
            let page = browser.get_page().map_err(|e| {
                McpError::internal_error(format!("Get page failed: {}", e), None)
            })?;
            let script = r#"
                (function() {
                    const links = Array.from(document.querySelectorAll('a[href]'))
                        .filter(a => a.href && !a.href.startsWith('javascript:'));
                    return links.length > 0 ? JSON.stringify(links.map(a => a.href)) : '[]';
                })()
            "#;
            let result = page.evaluate(script).await.unwrap_or_else(|_| "[]".to_string());
            let hrefs: Vec<String> = serde_json::from_str(&result).unwrap_or_default();
            drop(g);
            hrefs.get(idx as usize)
                .cloned()
                .ok_or_else(|| McpError::invalid_params(format!("Link index {} out of range ({} links)", idx, hrefs.len()), None))?
        } else {
            return Err(McpError::invalid_params("Provide 'index' or 'url'", None));
        };

        let mut g = self.browser.write().await;
        let browser = g.as_mut().ok_or_else(|| McpError::internal_error("No browser", None))?;
        browser.navigate(&url).await.map_err(|e| {
            McpError::internal_error(format!("Navigate failed: {}", e), None)
        })?;
        Ok(CallToolResult::structured(serde_json::json!({
            "success": true,
            "url": url
        })))
    }

    #[tool(description = "List available content: links and images with indices")]
    async fn list_content(&self) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let g = self.browser.read().await;
        let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
        let page = browser.get_page().map_err(|e| {
            McpError::internal_error(format!("Get page failed: {}", e), None)
        })?;
        let script = r#"
            (function() {
                const links = Array.from(document.querySelectorAll('a[href]'))
                    .filter(a => a.href && !a.href.startsWith('javascript:'))
                    .map((a, i) => ({ index: i, href: a.href, text: (a.textContent||'').trim().slice(0, 100) }));
                const images = Array.from(document.querySelectorAll('img[src]'))
                    .map((img, i) => ({ index: i, src: img.src, alt: (img.alt||'').slice(0, 80) }));
                return JSON.stringify({ links, images });
            })()
        "#;
        let result = page.evaluate(script).await.unwrap_or_else(|_| "{\"links\":[],\"images\":[]}".to_string());
        let content: serde_json::Value = serde_json::from_str(&result).unwrap_or(serde_json::json!({"links":[],"images":[]}));
        let url = browser.get_current_url().await.unwrap_or_default();
        drop(g);
        Ok(CallToolResult::structured(serde_json::json!({
            "url": url,
            "links": content.get("links").cloned().unwrap_or_default(),
            "images": content.get("images").cloned().unwrap_or_default()
        })))
    }

    #[tool(description = "Get page text content")]
    async fn get_content(&self, Parameters(p): Parameters<GetContentParams>) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let g = self.browser.read().await;
        let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
        let page = browser.get_page().map_err(|e| {
            McpError::internal_error(format!("Get page failed: {}", e), None)
        })?;
        let url = browser.get_current_url().await.unwrap_or_default();
        let max_chars = p.max_chars.unwrap_or(100_000) as usize;
        let expr = format!(
            "(document.body?.innerText||document.body?.textContent||'').slice(0,{})",
            max_chars
        );
        let text = page.evaluate(&expr).await.unwrap_or_default();
        drop(g);
        Ok(CallToolResult::structured(serde_json::json!({
            "url": url,
            "text": text,
            "length": text.len()
        })))
    }

    #[tool(description = "Get or save image by index from list_content.images (captures visible element as screenshot)")]
    async fn get_image(&self, Parameters(p): Parameters<GetImageParams>) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let idx = p.index.unwrap_or(0);
        let g = self.browser.read().await;
        let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
        let page = browser.get_page().map_err(|e| {
            McpError::internal_error(format!("Get page failed: {}", e), None)
        })?;
        let elements = page
            .get_elements_by_css_selector("img[src]")
            .await
            .map_err(|e| McpError::internal_error(format!("Get elements failed: {}", e), None))?;
        let element = elements
            .get(idx as usize)
            .ok_or_else(|| McpError::invalid_params(format!("Image index {} out of range ({} images)", idx, elements.len()), None))?;
        let b64 = element
            .screenshot(Some("png"), None)
            .await
            .map_err(|e| McpError::internal_error(format!("Screenshot failed: {}", e), None))?;
        drop(g);
        Ok(CallToolResult::success(vec![Content::image(b64, "image/png")]))
    }

    #[tool(description = "Save text content or image (by index) to a file")]
    async fn save_content(&self, Parameters(p): Parameters<SaveContentParams>) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let path = p.path;
        match p.content_type.to_lowercase().as_str() {
            "text" => {
                let g = self.browser.read().await;
                let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
                let page = browser.get_page().map_err(|e| {
                    McpError::internal_error(format!("Get page failed: {}", e), None)
                })?;
                let text = page
                    .evaluate("(document.body?.innerText||document.body?.textContent||'')")
                    .await
                    .unwrap_or_default();
                drop(g);
                tokio::fs::write(&path, &text)
                    .await
                    .map_err(|e| McpError::internal_error(format!("Write failed: {}", e), None))?;
            }
            "image" => {
                let idx = p.image_index.unwrap_or(0);
                let g = self.browser.read().await;
                let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
                let page = browser.get_page().map_err(|e| {
                    McpError::internal_error(format!("Get page failed: {}", e), None)
                })?;
                let elements = page
                    .get_elements_by_css_selector("img[src]")
                    .await
                    .map_err(|e| McpError::internal_error(format!("Get elements failed: {}", e), None))?;
                let element = elements
                    .get(idx as usize)
                    .ok_or_else(|| McpError::invalid_params(format!("Image index {} out of range", idx), None))?;
                let b64 = element
                    .screenshot(Some("png"), None)
                    .await
                    .map_err(|e| McpError::internal_error(format!("Screenshot failed: {}", e), None))?;
                drop(g);
                let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64)
                    .map_err(|e| McpError::internal_error(format!("Base64 decode failed: {}", e), None))?;
                tokio::fs::write(&path, &bytes)
                    .await
                    .map_err(|e| McpError::internal_error(format!("Write failed: {}", e), None))?;
            }
            _ => return Err(McpError::invalid_params("content_type must be 'text' or 'image'", None)),
        }
        Ok(CallToolResult::structured(serde_json::json!({
            "success": true,
            "path": path
        })))
    }

    #[tool(description = "Take screenshot: full page, or a specific element by CSS selector")]
    async fn screenshot(&self, Parameters(p): Parameters<ScreenshotParams>) -> Result<CallToolResult, McpError> {
        self.ensure_browser().await?;
        let g = self.browser.read().await;
        let browser = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;

        let bytes = if let Some(selector) = p.selector {
            let page = browser.get_page().map_err(|e| {
                McpError::internal_error(format!("Get page failed: {}", e), None)
            })?;
            let elements = page
                .get_elements_by_css_selector(&selector)
                .await
                .map_err(|e| McpError::internal_error(format!("Selector failed: {}", e), None))?;
            let idx = p.element_index.unwrap_or(0) as usize;
            let element = elements
                .get(idx)
                .ok_or_else(|| McpError::invalid_params(format!("Element index {} out of range ({} matches for '{}')", idx, elements.len(), selector), None))?;
            let b64 = element
                .screenshot(Some("png"), None)
                .await
                .map_err(|e| McpError::internal_error(format!("Element screenshot failed: {}", e), None))?;
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64)
                .map_err(|e| McpError::internal_error(format!("Base64 decode: {}", e), None))?
        } else {
            browser
                .take_screenshot(None, p.full_page.unwrap_or(false), None, None)
                .await
                .map_err(|e| McpError::internal_error(format!("Screenshot failed: {}", e), None))?
        };
        drop(g);
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        Ok(CallToolResult::success(vec![Content::image(b64, "image/png")]))
    }
}

#[tool_handler]
impl ServerHandler for BrowsingService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Browse the web: navigate, get_links, follow_link, list_content (links+images), get_content, get_image, save_content, screenshot (full or by selector).".into()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    browsing::init();
    let service = BrowsingService::new();
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let running = rmcp::ServiceExt::serve(service, transport).await?;
    running.waiting().await?;
    Ok(())
}
