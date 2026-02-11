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
    let browser_guard = std::sync::Arc::clone(&service.browser);
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let running = rmcp::ServiceExt::serve(service, transport).await?;
    let run_result = running.waiting().await;

    // Gracefully close browser when server shuts down (always, regardless of run result)
    let mut g = browser_guard.write().await;
    if let Some(ref mut browser) = *g {
        let _ = browser.stop().await;
        tracing::info!("Browser instance closed gracefully");
    }
    *g = None;
    drop(g);

    run_result.map(|_| ()).map_err(anyhow::Error::from)
}
