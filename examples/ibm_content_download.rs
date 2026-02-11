//! Demo: Download content from ibm.com
//!
//! This example demonstrates:
//! - Navigating to a website
//! - Extracting and downloading page content
//! - Saving content to a file
//!
//! Usage:
//!   cargo run --example ibm_content_download

use browsing::browser::{Browser, BrowserProfile};
use browsing::dom::DOMProcessorImpl;
use browsing::error::Result;
use browsing::traits::DOMProcessor;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌐 IBM Content Download Demo\n");
    println!("This demo will:");
    println!("  1. Open a browser");
    println!("  2. Navigate to ibm.com");
    println!("  3. Extract page content");
    println!("  4. Save content to a file\n");

    // 1. Create browser profile
    // Check for BROWSER_USE_HEADLESS environment variable, default to false (visible mode)
    let headless = std::env::var("BROWSER_USE_HEADLESS")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let profile = BrowserProfile {
        headless: Some(headless),
        proxy: None,
        ..Default::default()
    };

    println!("📋 Step 1: Creating browser...");
    println!("   Mode: {}", if headless { "headless" } else { "visible" });

    let mut browser = Box::new(Browser::new(profile));

    // Start the browser
    println!("   Starting browser...\n");
    browser.start().await?;
    println!("   ✓ Browser started\n");

    // 2. Navigate to IBM website
    println!("📋 Step 2: Navigating to ibm.com...");
    browser.navigate("https://www.ibm.com").await?;
    println!("   ✓ Navigated to IBM website\n");

    // Wait a moment for the page to fully load
    println!("   Waiting for page to load...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    println!("   ✓ Page loaded\n");

    // 3. Get the CDP client and session ID
    let cdp_client = browser.get_cdp_client()?;
    let session_id = browser.get_session_id()?;
    let target_id = browser.get_current_target_id()?;

    // 4. Create DOM processor and extract content
    println!("📋 Step 3: Extracting page content...");
    let dom_processor = DOMProcessorImpl::new()
        .with_cdp_client(cdp_client, session_id)
        .with_target_id(target_id);

    // Get the page state as a string
    let page_content = dom_processor.get_page_state_string().await?;
    println!("   ✓ Content extracted ({} characters)\n", page_content.len());

    // 5. Save content to file
    println!("📋 Step 4: Saving content to file...");
    let output_path = Path::new("ibm_content.txt");

    let mut file = File::create(output_path)?;
    writeln!(file, "IBM.com Content Download")?;
    writeln!(file, "{}", "=".repeat(60))?;
    writeln!(file, "URL: https://www.ibm.com")?;
    writeln!(file, "Downloaded at: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(file, "{}\n", "=".repeat(60))?;
    writeln!(file, "{}", page_content)?;

    println!("   ✓ Content saved to: {}\n", output_path.display());

    // 6. Display summary
    println!("📊 Download Summary:");
    println!("   URL: https://www.ibm.com");
    println!("   Output file: {}", output_path.display());
    println!("   Content size: {} bytes", page_content.len());

    // Show a preview of the content
    let preview_len = 500.min(page_content.len());
    let preview = &page_content[..preview_len];
    println!("\n📝 Content Preview (first {} characters):", preview_len);
    println!("---");
    println!("{}", preview);
    if page_content.len() > 500 {
        println!("...");
    }
    println!("---");

    println!("\n🎉 Demo completed successfully!");
    println!("\n💡 Next steps:");
    println!("   - Open ibm_content.txt to view the full content");
    println!("   - Modify this example to download from other websites");
    println!("   - Add content filtering to extract specific information");

    println!("\n  Closing browser...");
    browser.stop().await?;
    println!("  ✓ Browser closed gracefully");

    Ok(())
}
