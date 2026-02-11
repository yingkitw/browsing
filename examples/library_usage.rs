//! Example of using browsing as a library
//!
//! This example demonstrates basic browser automation using the browsing library.

use anyhow::Result;
use browsing::Browser;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    browsing::init();

    println!("=== Browsing Library Example ===\n");

    // Create and start browser
    use browsing::browser::profile::BrowserProfile;
    let profile = BrowserProfile::default();
    let mut browser = Browser::new(profile);
    browser.start().await?;
    println!("✓ Browser launched successfully");

    // Navigate to a webpage
    browser.navigate("https://example.com").await?;
    println!("✓ Navigated to example.com");

    // Get current URL
    let url = browser.get_current_url().await?;
    println!("  Current URL: {}", url);

    // Get page title
    let title = browser.get_current_page_title().await?;
    println!("  Page title: {}", title);

    // Get list of tabs
    let tabs = browser.get_tabs().await?;
    println!("  Open tabs: {}", tabs.len());

    // Take a screenshot
    let screenshot = browser.take_screenshot(None, false, Some("png"), None).await?;
    println!("  Screenshot captured: {} bytes", screenshot.len());

    // Navigate to another page
    browser.navigate("https://www.rust-lang.org").await?;
    println!("\n✓ Navigated to rust-lang.org");

    let url = browser.get_current_url().await?;
    println!("  Current URL: {}", url);

    println!("\n=== Example completed successfully ===");

    Ok(())
}
