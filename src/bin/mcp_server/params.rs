//! MCP tool parameter types

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NavigateParams {
    #[schemars(description = "URL to navigate to")]
    pub url: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FollowLinkParams {
    #[schemars(description = "Index of link from get_links (0-based)")]
    pub index: Option<u32>,
    #[schemars(description = "Or specify URL directly")]
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetContentParams {
    #[schemars(description = "Max characters to return")]
    pub max_chars: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveContentParams {
    #[schemars(description = "Path to save file")]
    pub path: String,
    #[schemars(description = "Content type: text, or image index from list_content")]
    pub content_type: String,
    #[schemars(description = "For image: index from list_content.images")]
    pub image_index: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetImageParams {
    #[schemars(description = "Index from list_content.images (0-based)")]
    pub index: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GenerateSitemapParams {
    #[schemars(description = "Starting URL to crawl")]
    pub url: String,
    #[schemars(description = "Maximum pages to crawl")]
    pub max_pages: Option<u32>,
    #[schemars(description = "Maximum link depth from start")]
    pub max_depth: Option<u32>,
    #[schemars(description = "Only follow links within same domain")]
    pub same_domain_only: Option<bool>,
    #[schemars(description = "Chars of content preview per page")]
    pub content_preview_chars: Option<u32>,
    #[schemars(description = "Path to save sitemap JSON file")]
    pub save_path: Option<String>,
    #[schemars(description = "Delay in ms between page navigations")]
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScreenshotParams {
    #[schemars(description = "Capture full scrollable page")]
    pub full_page: Option<bool>,
    #[schemars(description = "CSS selector for element (e.g. '.sidebar', '#content', 'a')")]
    pub selector: Option<String>,
    #[schemars(description = "If selector matches multiple elements, use this index")]
    pub element_index: Option<u32>,
}
