//! Simple navigation example
//!
//! This example demonstrates basic browser automation:
//! - Starting a browser
//! - Navigating to a URL
//! - Extracting page content
//! - Taking screenshots
//!
//! Usage:
//!   cargo run --example simple_navigation

use browsing::browser::{Browser, BrowserProfile};
use browsing::dom::DOMProcessorImpl;
use browsing::error::Result;
use browsing::traits::DOMProcessor;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌐 Simple Navigation Example\n");

    // Create browser with default profile
    println!("1. Creating browser...");
    let mut browser = Browser::new(BrowserProfile {
        headless: Some(false),
        ..Default::default()
    });

    // Start the browser
    println!("2. Starting browser...");
    browser.start().await?;
    println!("   ✓ Browser started\n");

    // Navigate to a website
    let url = "https://example.com";
    println!("3. Navigating to {}...", url);
    browser.navigate(url).await?;
    println!("   ✓ Navigation complete\n");

    // Wait a moment for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Get current URL
    println!("4. Getting current URL...");
    let current_url = browser.get_current_url().await?;
    println!("   Current URL: {}\n", current_url);

    // Extract DOM content
    println!("5. Extracting DOM content...");
    let dom_processor = DOMProcessorImpl::new();
    match dom_processor.get_selector_map().await {
        Ok(selector_map) => {
            println!("   ✓ Found {} clickable elements\n", selector_map.len());
        }
        Err(e) => {
            println!("   ⚠ Could not extract DOM: {}\n", e);
        }
    }

    // Take a screenshot
    println!("6. Taking screenshot...");
    let screenshot_path = "/tmp/example_screenshot.png";
    browser.take_screenshot(Some(screenshot_path), false, None, None).await?;
    println!("   ✓ Screenshot saved to: {}\n", screenshot_path);

    // Create a new tab
    println!("7. Creating new tab...");
    browser.create_new_tab(Some("https://www.rust-lang.org")).await?;
    println!("   ✓ New tab created\n");

    // Get all tabs
    println!("8. Getting all tabs...");
    let tabs = browser.get_tabs().await?;
    println!("   Open tabs: {}", tabs.len());
    for (i, tab) in tabs.iter().enumerate() {
        println!("     Tab {}: {}", i, tab.url);
    }
    println!();

    // Switch back to first tab
    if tabs.len() > 1 {
        println!("9. Switching to first tab...");
        browser.switch_to_tab(&tabs[0].target_id).await?;
        println!("   ✓ Switched to first tab\n");
    }

    // Go back in history
    println!("10. Testing navigation history...");
    browser.go_back().await?;
    println!("    ✓ Went back in history\n");

    println!("✅ Example completed successfully!");
    println!("\n💡 The browser will remain open. Close it manually or press Ctrl+C");
    
    // Keep the program running so browser stays open
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    Ok(())
}
