//! Lightweight MCP server: browse, navigate, get links, follow links,
//! list content (links/images), get/save content, screenshot (full or element),
//! generate_sitemap. Lazy browser init. RwLock enables parallel operations.

mod params;
mod service;
mod sitemap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    browsing::init();
    let service = service::BrowsingService::new();
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let running = rmcp::ServiceExt::serve(service, transport).await?;
    running.waiting().await?;
    Ok(())
}
