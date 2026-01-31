//! Basic navigation and content reading example
//!
//! This example demonstrates:
//! - Launching a browser (visible or headless)
//! - Navigating to a specific URL
//! - Reading page content
//! - Taking screenshots
//! - Working in headless mode
//!
//! Usage:
//!   # Visible mode (default)
//!   cargo run --example basic_navigation
//!
//!   # Headless mode
//!   BROWSER_USE_HEADLESS=true cargo run --example basic_navigation
//!
//! Requirements:
//!   - Chrome/Chromium browser installed

use browsing::browser::{Browser, BrowserProfile};
use browsing::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    browsing::init();

    // Check for headless mode environment variable
    let headless = std::env::var("BROWSER_USE_HEADLESS")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    println!("🚀 Basic Browser Navigation Example\n");
    println!("Mode: {}\n", if headless { "headless (background)" } else { "visible" });

    // Create browser profile
    let profile = BrowserProfile {
        headless: Some(headless),
        ..Default::default()
    };

    // Create and start browser
    println!("📋 Step 1: Starting browser...");
    let mut browser = Browser::new(profile);
    browser.start().await?;
    println!("   ✓ Browser started\n");

    // Navigate to a specific URL
    let target_url = "https://example.com";
    println!("📋 Step 2: Navigating to {}...", target_url);
    browser.navigate(target_url).await?;

    // Wait a moment for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Get current URL to verify navigation
    let current_url = browser.get_current_url().await?;
    println!("   ✓ Current URL: {}\n", current_url);

    // Get page title
    let title = browser.get_current_page_title().await?;
    println!("📋 Step 3: Page Information");
    println!("   Title: {}", title);
    println!("   URL: {}", current_url);
    println!();

    // Get browser state summary with DOM content
    println!("📋 Step 4: Reading page content...");
    let state = browser.get_browser_state_summary(true, None).await?;

    println!("   📄 Page Title: {}", state.title);
    println!("   🔗 URL: {}", state.url);

    // Display DOM content
    let dom_state = &state.dom_state;
    // Display first few interactive elements
    if !dom_state.selector_map.is_empty() {
            println!("\n   🔍 Found {} interactive elements:", dom_state.selector_map.len());

            // Show first 5 elements
            for (i, (index, element)) in dom_state.selector_map.iter().take(5).enumerate() {
                println!("      [{}] {}: {} - {:?}",
                    i + 1,
                    index,
                    element.tag,
                    element.text.as_deref().unwrap_or(&"[no text]".to_string())
                );
            }

            if dom_state.selector_map.len() > 5 {
                println!("      ... and {} more", dom_state.selector_map.len() - 5);
            }
    }

    // Show plain text content if available
    if let Some(content) = &dom_state.text {
        let lines: Vec<&str> = content.lines().take(3).collect();
        if !lines.is_empty() {
            println!("\n   📝 Text content preview:");
            for line in lines {
                println!("      {}", line);
            }
            if content.lines().count() > 3 {
                println!("      ... ({} more characters)", content.len() - content.lines().take(3).map(|l| l.len()).sum::<usize>());
            }
        }
    }
    println!();

    // Take a screenshot
    println!("📋 Step 5: Taking screenshot...");
    let screenshot_path = "screenshot_example.png";
    browser.take_screenshot(Some(screenshot_path), false, None, None).await?;
    println!("   ✓ Screenshot saved to: {}", screenshot_path);
    println!();

    // Additional navigation examples
    println!("📋 Step 6: More navigation examples...");

    // Navigate to another site
    println!("   • Navigating to Wikipedia...");
    browser.navigate("https://www.wikipedia.org").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let wiki_title = browser.get_current_page_title().await?;
    println!("     ✓ Loaded: {}", wiki_title);

    // Go back
    println!("   • Going back...");
    browser.go_back().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let back_url = browser.get_current_url().await?;
    println!("     ✓ Back to: {}", back_url);
    println!();

    println!("✅ Navigation example completed successfully!");
    println!("\n💡 Tips:");
    println!("   • Set BROWSER_USE_HEADLESS=true to run in background mode");
    println!("   • The browser will stay open for a few seconds so you can see the result");
    println!("   • Screenshots are saved to the current directory");
    println!("   • Try modifying the target_url to visit different websites");

    // Keep browser open for a moment so user can see the result
    if !headless {
        println!("\n⏳ Keeping browser open for 5 seconds for inspection...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    Ok(())
}
