//! Comprehensive example: browse, navigate, get text, get image
//!
//! Demonstrates the full flow:
//! 1. Start browser and navigate to a page
//! 2. Extract text content (innerText)
//! 3. Take screenshot (save image)
//! 4. Navigate to rust-lang.org
//! 5. Follow first link
//! 6. Extract text and screenshot from the followed page
//!
//! Usage:
//!   cargo run --example browse_navigate_extract
//!
//!   # Headless mode
//!   BROWSER_USE_HEADLESS=true cargo run --example browse_navigate_extract

use browsing::browser::{Browser, BrowserProfile};
use browsing::error::Result;
use std::env;
use std::path::Path;

/// Extract visible text from the current page (up to max_chars)
async fn get_page_text(browser: &Browser, max_chars: usize) -> Result<String> {
    let page = browser.get_page()?;
    let expr = format!(
        "(document.body?.innerText || document.body?.textContent || '').slice(0, {})",
        max_chars
    );
    page.evaluate(&expr).await
}

/// Extract page title
async fn get_page_title(browser: &Browser) -> Result<String> {
    let page = browser.get_page()?;
    page.evaluate("document.title").await
}

/// Get first link href that differs from current URL (skip same-page links)
async fn get_first_different_link_href(browser: &Browser, current_url: &str) -> Result<Option<String>> {
    let page = browser.get_page()?;
    let current_escaped = serde_json::to_string(current_url).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        r#"
        (function() {{
            const current = {};
            const links = Array.from(document.querySelectorAll('a[href]'))
                .filter(a => a.href && !a.href.startsWith('javascript:'))
                .map(a => a.href);
            const found = links.find(href => href !== current);
            return found ? JSON.stringify(found) : 'null';
        }})()
        "#,
        current_escaped
    );
    let result = page.evaluate(&script).await?;
    Ok(serde_json::from_str::<Option<String>>(&result).unwrap_or(None))
}

#[tokio::main]
async fn main() -> Result<()> {
    browsing::init();

    let headless = env::var("BROWSER_USE_HEADLESS")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    println!("═══ Comprehensive Browse Example ═══\n");
    println!("  Browse → Navigate → Get Text → Get Image\n");

    let profile = BrowserProfile {
        headless: Some(headless),
        ..Default::default()
    };

    // ─── 1. Start browser ─────────────────────────────────────────────
    println!("1️⃣  Starting browser...");
    let mut browser = Browser::new(profile);
    browser.start().await?;
    println!("    ✓ Browser ready\n");

    // ─── 2. Navigate to first page ────────────────────────────────────
    let url1 = "https://example.com";
    println!("2️⃣  Navigating to {}...", url1);
    browser.navigate(url1).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let url = browser.get_current_url().await?;
    let title = get_page_title(&browser).await.unwrap_or_else(|_| "".into());
    println!("    ✓ Loaded: {} ({})\n", title, url);

    // ─── 3. Get text content ──────────────────────────────────────────
    println!("3️⃣  Extracting text content...");
    let text = get_page_text(&browser, 2000).await?;
    let preview: String = text.chars().take(300).collect();
    println!("    📄 Text preview ({} chars total):", text.len());
    println!("    ┌─────────────────────────────────────────");
    for line in preview.lines().take(8) {
        println!("    │ {}", line);
    }
    if text.len() > 300 {
        println!("    │ ...");
    }
    println!("    └─────────────────────────────────────────\n");

    // ─── 4. Take screenshot (image) ───────────────────────────────────
    let screenshot1 = "example_com_screenshot.png";
    println!("4️⃣  Taking screenshot...");
    browser
        .take_screenshot(Some(screenshot1), false, None, None)
        .await?;
    println!("    ✓ Image saved: {}\n", screenshot1);

    // ─── 5. Navigate to rust-lang.org ────────────────────────────────────
    let url2 = "https://www.rust-lang.org";
    println!("5️⃣  Navigating to {}...", url2);
    browser.navigate(url2).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let url = browser.get_current_url().await?;
    let title2 = get_page_title(&browser).await.unwrap_or_else(|_| "".into());
    println!("    ✓ Loaded: {} ({})\n", title2, url);

    // ─── 5a. Follow first different link ─────────────────────────────────────
    println!("5a️⃣ Following first link (skipping same-URL links)...");
    if let Some(href) = get_first_different_link_href(&browser, &url).await? {
        browser.navigate(&href).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let followed_url = browser.get_current_url().await?;
        println!("    ✓ Followed to: {}\n", followed_url);
    } else {
        println!("    ⚠ No links found, staying on page\n");
    }

    // ─── 6. Get text from current page ─────────────────────────────────
    println!("6️⃣  Extracting text from Rust page...");
    let text2 = get_page_text(&browser, 2000).await?;
    let preview2: String = text2.chars().take(250).collect();
    println!("    📄 Text preview ({} chars total):", text2.len());
    println!("    ┌─────────────────────────────────────────");
    for line in preview2.lines().take(6) {
        println!("    │ {}", line);
    }
    if text2.len() > 250 {
        println!("    │ ...");
    }
    println!("    └─────────────────────────────────────────\n");

    // ─── 7. Take second screenshot ───────────────────────────────────
    let screenshot2 = "rust_lang_screenshot.png";
    println!("7️⃣  Taking screenshot...");
    browser
        .take_screenshot(Some(screenshot2), false, None, None)
        .await?;
    println!("    ✓ Image saved: {}\n", screenshot2);

    // ─── Summary ──────────────────────────────────────────────────────
    println!("═══ Done ═══");
    println!("  Screenshots: {}, {}", screenshot1, screenshot2);
    println!("  Text extracted from 2 pages");
    if Path::new(screenshot1).exists() {
        println!("  💡 Open {} to view the captured image", screenshot1);
    }

    if !headless {
        println!("\n  Keeping browser open 3 seconds...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    Ok(())
}
