//! Sitemap generation by crawling and capturing navigation + content

use browsing::Browser;
use rmcp::model::ErrorData as McpError;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use super::params::GenerateSitemapParams;

fn extract_domain(url_str: &str) -> Option<String> {
    Url::parse(url_str)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
}

fn is_same_domain(url_str: &str, base: &str) -> bool {
    extract_domain(url_str).as_deref() == Some(base)
}

/// Run sitemap crawl: navigate, capture title + content preview, discover links.
pub async fn run_sitemap_crawl(
    browser: Arc<RwLock<Option<Browser>>>,
    p: GenerateSitemapParams,
) -> Result<serde_json::Value, McpError> {
    let max_pages = p.max_pages.unwrap_or(30) as usize;
    let max_depth = p.max_depth.unwrap_or(3) as u32;
    let same_domain_only = p.same_domain_only.unwrap_or(true);
    let preview_chars = p.content_preview_chars.unwrap_or(500) as usize;
    let delay_ms = p.delay_ms.unwrap_or(800);

    let base_url =
        Url::parse(&p.url).map_err(|e| McpError::invalid_params(format!("Invalid URL: {}", e), None))?;
    let base_domain = base_url.host_str().unwrap_or("").to_string();

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();
    queue.push_back((p.url.clone(), 0));

    let mut pages: Vec<serde_json::Value> = Vec::new();

    while let Some((url, depth)) = queue.pop_front() {
        if pages.len() >= max_pages || depth > max_depth {
            continue;
        }
        let url_norm = url.trim_end_matches('/').to_string();
        if visited.contains(&url_norm) {
            continue;
        }
        visited.insert(url_norm.clone());

        // Navigate
        {
            let mut g = browser.write().await;
            let b = g.as_mut().ok_or_else(|| McpError::internal_error("No browser", None))?;
            if b.navigate(&url).await.is_err() {
                drop(g);
                continue;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        // Capture content
        let (current_url, title, content_preview, links) = {
            let g = browser.read().await;
            let b = g.as_ref().ok_or_else(|| McpError::internal_error("No browser", None))?;
            let page = b.get_page().map_err(|e| {
                McpError::internal_error(format!("Get page failed: {}", e), None)
            })?;

            let current_url = b.get_current_url().await.unwrap_or_else(|_| url.clone());
            let title = page.evaluate("document.title").await.unwrap_or_default();
            let content_script = format!(
                "(document.body?.innerText||document.body?.textContent||'').slice(0,{})",
                preview_chars
            );
            let content_preview = page.evaluate(&content_script).await.unwrap_or_default();

            let links_script = r#"
                (function() {
                    const links = Array.from(document.querySelectorAll('a[href]'))
                        .filter(a => a.href && !a.href.startsWith('javascript:'))
                        .map(a => a.href);
                    return JSON.stringify([...new Set(links)]);
                })()
            "#;
            let links_result = page.evaluate(links_script).await.unwrap_or_else(|_| "[]".to_string());
            let links: Vec<String> = serde_json::from_str(&links_result).unwrap_or_default();

            (current_url, title, content_preview, links)
        };

        let outbound: Vec<String> = links
            .iter()
            .filter(|href| {
                if same_domain_only {
                    is_same_domain(href, &base_domain)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        for href in &outbound {
            let h = href.trim_end_matches('/').to_string();
            if !visited.contains(&h) {
                queue.push_back((href.clone(), depth + 1));
            }
        }

        pages.push(serde_json::json!({
            "url": current_url,
            "title": title,
            "content_preview": content_preview,
            "links": outbound,
            "depth": depth
        }));
    }

    Ok(serde_json::json!({
        "base_url": p.url,
        "total_pages": pages.len(),
        "pages": pages
    }))
}
