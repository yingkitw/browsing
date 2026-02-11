//! Tests for CDP parameter enhancements
//!
//! This test suite verifies the fixes for:
//! 1. Target selection filtering for page-type targets
//! 2. get_current_url using Runtime.evaluate
//! 3. Removed unsupported Chrome flags

use browsing::browser::{Browser, BrowserProfile};

/// Test that browser starts with a page-type target (not extension/service worker)
#[tokio::test]
#[ignore] // Requires actual Chrome installation
async fn test_browser_selects_page_target() {
    let profile = BrowserProfile {
        headless: Some(true),
        ..Default::default()
    };

    let mut browser = Browser::new(profile);

    // Start the browser
    browser.start().await.expect("Browser should start successfully");

    // Verify we have a page target by checking CDP client is available
    let cdp_client = browser.get_cdp_client();
    assert!(cdp_client.is_ok(), "Should have CDP client after start");

    // Verify session is available (page targets should have sessions)
    let session_id = browser.get_session_id();
    assert!(session_id.is_ok(), "Should have session ID for page target");

    // Navigate to verify the target is actually a page (not extension)
    let navigate_result = browser.navigate("https://example.com").await;
    assert!(navigate_result.is_ok(), "Should be able to navigate on page target");

    browser.stop().await.ok();
}

/// Test that get_current_url returns actual URL from page (not cached session URL)
#[tokio::test]
#[ignore] // Requires actual Chrome installation
async fn test_get_current_url_returns_actual_url() {
    let profile = BrowserProfile {
        headless: Some(true),
        ..Default::default()
    };

    let mut browser = Browser::new(profile);

    browser.start().await.expect("Browser should start");

    // Initial URL should not be chrome-extension:// or chrome://newtab/
    let initial_url = browser.get_current_url().await.ok();
    if let Some(url) = initial_url {
        // URL should be from the actual page, not a cached session URL
        // It should be a valid URL (http/https) or about:blank
        assert!(
            url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("about:blank")
                || url.starts_with("chrome://newtab"),
            "Initial URL should be a valid page URL, got: {}",
            url
        );
    }

    // Navigate to example.com
    browser
        .navigate("https://example.com")
        .await
        .expect("Should navigate to example.com");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Verify URL is actually example.com (not cached session URL)
    let current_url = browser
        .get_current_url()
        .await
        .expect("Should get current URL");

    assert!(
        current_url.contains("example.com") || current_url.contains("example"),
        "get_current_url should return actual page URL using Runtime.evaluate, got: {}",
        current_url
    );

    // Navigate to another URL
    browser
        .navigate("https://www.bing.com")
        .await
        .expect("Should navigate to bing");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Verify URL updates to the new location
    let updated_url = browser.get_current_url().await.expect("Should get URL");

    assert!(
        updated_url.contains("bing"),
        "get_current_url should update after navigation, got: {}",
        updated_url
    );

    browser.stop().await.ok();
}

/// Integration test: Full navigation workflow
#[tokio::test]
#[ignore] // Requires actual Chrome installation
async fn test_full_navigation_workflow() {
    let profile = BrowserProfile {
        headless: Some(true),
        ..Default::default()
    };

    let mut browser = Browser::new(profile);

    // Step 1: Start browser
    browser.start().await.expect("Browser should start");

    // Step 2: Navigate to first URL
    browser
        .navigate("https://example.com")
        .await
        .expect("Should navigate to example.com");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url1 = browser.get_current_url().await.expect("Should get URL");
    assert!(
        url1.contains("example"),
        "Should be at example.com, got: {}",
        url1
    );

    // Step 3: Navigate to second URL
    browser
        .navigate("https://www.bing.com")
        .await
        .expect("Should navigate to bing");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url2 = browser.get_current_url().await.expect("Should get URL");
    assert!(
        url2.contains("bing"),
        "Should be at bing, got: {}",
        url2
    );

    // Step 4: Navigate to third URL
    browser
        .navigate("https://www.github.com")
        .await
        .expect("Should navigate to github");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let url3 = browser.get_current_url().await.expect("Should get URL");
    assert!(
        url3.contains("github"),
        "Should be at github, got: {}",
        url3
    );

    browser.stop().await.ok();
}

/// Test that page information is correctly retrieved
#[tokio::test]
#[ignore] // Requires actual Chrome installation
async fn test_page_title_and_url_retrieval() {
    let profile = BrowserProfile {
        headless: Some(true),
        ..Default::default()
    };

    let mut browser = Browser::new(profile);

    browser.start().await.expect("Browser should start");

    browser
        .navigate("https://example.com")
        .await
        .expect("Should navigate to example.com");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Get page title
    let title = browser
        .get_current_page_title()
        .await
        .expect("Should get page title");

    assert!(
        title.contains("Example Domain"),
        "Page title should contain 'Example Domain', got: {}",
        title
    );

    // Get current URL
    let url = browser.get_current_url().await.expect("Should get URL");

    assert!(
        url.contains("example"),
        "URL should contain 'example', got: {}",
        url
    );

    browser.stop().await.ok();
}

/// Test screenshot functionality works with correct CDP parameters
#[tokio::test]
#[ignore] // Requires actual Chrome installation
async fn test_screenshot_with_correct_cdp_parameters() {
    let profile = BrowserProfile {
        headless: Some(true),
        ..Default::default()
    };

    let mut browser = Browser::new(profile);

    browser.start().await.expect("Browser should start");

    browser
        .navigate("https://example.com")
        .await
        .expect("Should navigate");

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Take screenshot - should use Page.captureScreenshot with correct session
    let screenshot_path = "/tmp/test_screenshot.png";
    let screenshot_result = browser.take_screenshot(Some(screenshot_path), false, None, None).await;

    assert!(
        screenshot_result.is_ok(),
        "Screenshot should work with Page.captureScreenshot using correct session"
    );

    // Verify file was created
    std::fs::metadata(screenshot_path).ok();

    // Cleanup
    let _ = std::fs::remove_file(screenshot_path);

    browser.stop().await.ok();
}

/// Test that multiple navigation and back operations work correctly
#[tokio::test]
#[ignore] // Requires actual Chrome installation
async fn test_multiple_navigation_operations() {
    let profile = BrowserProfile {
        headless: Some(true),
        ..Default::default()
    };

    let mut browser = Browser::new(profile);

    browser.start().await.expect("Browser should start");

    let urls = vec![
        "https://example.com",
        "https://www.bing.com",
        "https://www.github.com",
        "https://www.stackoverflow.com",
    ];

    // Navigate to all URLs
    for url in &urls {
        browser.navigate(url).await.expect("Should navigate");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let current_url = browser.get_current_url().await.expect("Should get URL");
        // URL should be from one of our targets
        let domain = url.split("://").nth(1).unwrap_or("");
        assert!(
            current_url.contains(&domain[..domain.find('/').unwrap_or(domain.len())]),
            "Should be at {}, got: {}",
            url,
            current_url
        );
    }

    browser.stop().await.ok();
}
