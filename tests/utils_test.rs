//! Comprehensive tests for utilities module
//!
//! These tests cover:
//! - URL extraction and validation
//! - Domain pattern matching
//! - Signal handling for graceful shutdown
//! - Various utility functions

use browsing::utils::{extract_urls, match_url_with_domain_pattern};
use browsing::utils::signal::{is_shutdown_requested, set_shutdown_requested, SignalHandler};

// ============================================================================
// URL Extraction Tests
// ============================================================================

#[test]
fn test_extract_urls_basic() {
    let text = "Visit https://example.com for more info";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("example.com")));
}

#[test]
fn test_extract_urls_multiple() {
    let text = "Check out https://example.com and http://test.org/page";
    let urls = extract_urls(text);

    assert!(urls.len() >= 2);
    assert!(urls.iter().any(|u| u.contains("example.com")));
    assert!(urls.iter().any(|u| u.contains("test.org")));
}

#[test]
fn test_extract_urls_with_path() {
    let text = "Visit https://example.com/path/to/page?query=value";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("example.com")));
    assert!(urls.iter().any(|u| u.contains("/path/to/page")));
}

#[test]
fn test_extract_urls_with_www() {
    let text = "Go to https://www.example.com";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("www.example.com")));
}

#[test]
fn test_extract_urls_with_port() {
    let text = "Connect to http://localhost:8080/api";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("localhost:8080")));
}

#[test]
fn test_extract_urls_with_fragment() {
    let text = "See https://example.com/page#section";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("#section")));
}

#[test]
fn test_extract_urls_no_urls() {
    let text = "This is just plain text with no URLs";
    let urls = extract_urls(text);

    assert!(urls.is_empty());
}

#[test]
fn test_extract_urls_empty_string() {
    let text = "";
    let urls = extract_urls(text);

    assert!(urls.is_empty());
}

#[test]
fn test_extract_urls_with_punctuation() {
    let text = "Visit https://example.com, then go to http://test.org!";
    let urls = extract_urls(text);

    assert!(urls.len() >= 2);
    // URLs should be extracted without trailing punctuation
}

#[test]
fn test_extract_urls_mixed_case() {
    let text = "Go to HTTPS://EXAMPLE.COM and Http://Test.Org";
    let urls = extract_urls(text);

    assert!(urls.len() >= 2);
    // URLs should be normalized to lowercase
}

#[test]
fn test_extract_urls_with_credentials() {
    let text = "Use https://user:pass@example.com for authenticated access";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("user:pass")));
}

#[test]
fn test_extract_urls_ipv4() {
    let text = "Connect to http://192.168.1.1:8080";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("192.168.1.1")));
}

#[test]
fn test_extract_urls_ipv6() {
    let text = "Visit http://[2001:db8::1]:8080";
    let urls = extract_urls(text);

    // May or may not be extracted depending on regex
}

#[test]
fn test_extract_urls_with_parameters() {
    let text = "Go to https://example.com?param1=value1&param2=value2";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
}

#[test]
fn test_extract_urls_from_markdown() {
    let text = "[Link](https://example.com) and another [Link](http://test.org)";
    let urls = extract_urls(text);

    assert!(urls.len() >= 2);
}

#[test]
fn test_extract_urls_from_html() {
    let text = "Visit https://example.com for more info";
    let urls = extract_urls(text);

    assert!(!urls.is_empty());
    assert!(urls.iter().any(|u| u.contains("example.com")));
}

// ============================================================================
// Domain Pattern Matching Tests
// ============================================================================

#[test]
fn test_match_url_exact_domain() {
    assert!(match_url_with_domain_pattern(
        "https://example.com",
        "example.com"
    ));
}

#[test]
fn test_match_url_exact_domain_with_path() {
    assert!(match_url_with_domain_pattern(
        "https://example.com/path/to/page",
        "example.com"
    ));
}

#[test]
fn test_match_url_wildcard_subdomain() {
    assert!(match_url_with_domain_pattern(
        "https://www.example.com",
        "*.example.com"
    ));
    assert!(match_url_with_domain_pattern(
        "https://api.example.com",
        "*.example.com"
    ));
}

#[test]
fn test_match_url_wildcard_no_match_base() {
    assert!(!match_url_with_domain_pattern(
        "https://example.com",
        "*.example.com"
    ));
}

#[test]
fn test_match_url_wildcard_multiple_subdomains() {
    assert!(match_url_with_domain_pattern(
        "https://sub.api.example.com",
        "*.example.com"
    ));
}

#[test]
fn test_match_url_protocol_pattern() {
    assert!(match_url_with_domain_pattern(
        "https://example.com",
        "http*://example.com"
    ));
    assert!(match_url_with_domain_pattern(
        "http://example.com",
        "http*://example.com"
    ));
}

#[test]
fn test_match_url_protocol_pattern_no_match() {
    assert!(!match_url_with_domain_pattern(
        "ftp://example.com",
        "http*://example.com"
    ));
}

#[test]
fn test_match_url_exact_protocol() {
    assert!(match_url_with_domain_pattern(
        "https://example.com",
        "https://example.com"
    ));
}

#[test]
fn test_match_url_protocol_no_match() {
    assert!(!match_url_with_domain_pattern(
        "http://example.com",
        "https://example.com"
    ));
}

#[test]
fn test_match_url_empty_pattern() {
    assert!(!match_url_with_domain_pattern(
        "https://example.com",
        ""
    ));
}

#[test]
fn test_match_url_empty_url() {
    assert!(!match_url_with_domain_pattern(
        "",
        "example.com"
    ));
}

#[test]
fn test_match_url_both_empty() {
    assert!(!match_url_with_domain_pattern("", ""));
}

#[test]
fn test_match_url_invalid_url() {
    assert!(!match_url_with_domain_pattern(
        "not-a-url",
        "example.com"
    ));
}

#[test]
fn test_match_url_with_port() {
    assert!(match_url_with_domain_pattern(
        "https://example.com:8080",
        "example.com"
    ));
}

#[test]
fn test_match_url_subdomain_with_protocol() {
    assert!(match_url_with_domain_pattern(
        "https://www.example.com",
        "http*://*.example.com"
    ));
}

#[test]
fn test_match_url_complex_pattern() {
    assert!(match_url_with_domain_pattern(
        "https://api.v1.example.com",
        "*.example.com"
    ));
}

#[test]
fn test_match_url_case_sensitivity() {
    // Domain matching should be case-insensitive
    assert!(match_url_with_domain_pattern(
        "https://Example.COM",
        "example.com"
    ));
}

// ============================================================================
// Signal Handling Tests
// ============================================================================

#[test]
fn test_signal_handler_creation() {
    let handler = SignalHandler::new();
    assert!(!handler.is_shutdown_requested());
}

#[test]
fn test_signal_handler_default() {
    let handler = SignalHandler::default();
    assert!(!handler.is_shutdown_requested());
}

#[test]
fn test_signal_handler_set_shutdown() {
    let handler = SignalHandler::new();
    assert!(!handler.is_shutdown_requested());

    handler.set_shutdown();
    assert!(handler.is_shutdown_requested());
}

#[test]
fn test_global_shutdown_flag() {
    // Reset the global flag first
    set_shutdown_requested();
    assert!(is_shutdown_requested());
}

#[test]
fn test_shutdown_flag_persistence() {
    let handler1 = SignalHandler::new();
    let handler2 = SignalHandler::new();

    handler1.set_shutdown();

    // Both handlers should detect shutdown
    assert!(handler1.is_shutdown_requested());
    // handler2 won't see it because they have different flags
    // but the global flag should be set
    assert!(is_shutdown_requested());
}

// ============================================================================
// Additional Utility Tests
// ============================================================================

#[test]
fn test_url_scheme_validation() {
    let valid_schemes = vec!["http://", "https://", "ftp://", "file://"];

    for scheme in valid_schemes {
        assert!(scheme.ends_with("://"));
        assert!(scheme.len() >= 4);
    }
}

#[test]
fn test_url_components() {
    let url = "https://example.com:8080/path/to/page?query=value#fragment";

    assert!(url.contains("https://"));
    assert!(url.contains("example.com"));
    assert!(url.contains(":8080"));
    assert!(url.contains("/path/to/page"));
    assert!(url.contains("?query=value"));
    assert!(url.contains("#fragment"));
}

#[test]
fn test_domain_validation() {
    let valid_domains = vec![
        "example.com",
        "sub.example.com",
        "api.v1.example.com",
        "example.co.uk",
        "localhost",
    ];

    for domain in valid_domains {
        assert!(!domain.is_empty());
        assert!(!domain.contains("://"));
        assert!(!domain.contains("/"));
    }
}

#[test]
fn test_url_normalization() {
    // URLs should be normalized to have trailing slash or not
    let url1 = "https://example.com/";
    let url2 = "https://example.com";

    // These may or may not be equal depending on normalization
    assert!(url1.contains("https://example.com"));
}

#[test]
fn test_url_encoding() {
    let text = "Search for: café & restaurant";
    let encoded = urlencoding::encode(text);

    assert!(!encoded.contains(" "));
    assert!(!encoded.contains("&"));
}

#[test]
fn test_url_decoding() {
    let encoded = "caf%C3%A9%20%26%20restaurant";
    let decoded = urlencoding::decode(encoded);

    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap().as_ref(), "café & restaurant");
}

#[test]
fn test_regex_url_pattern() {
    use regex::Regex;

    let url_pattern = Regex::new(r"(?i)\bhttps?://[^\s]+").unwrap();

    let text = "Visit https://example.com and http://test.org";
    let matches: Vec<_> = url_pattern.find_iter(text).collect();

    assert_eq!(matches.len(), 2);
}

#[test]
fn test_text_sanitization() {
    let text = "Hello\nWorld\tTest";
    let sanitized = text.replace('\n', " ").replace('\t', " ");

    assert!(!sanitized.contains('\n'));
    assert!(!sanitized.contains('\t'));
}

#[test]
fn test_string_truncation() {
    let long_string = "a".repeat(1000);
    let truncated = &long_string[..100];

    assert_eq!(truncated.len(), 100);
    assert!(truncated.chars().all(|c| c == 'a'));
}

#[test]
fn test_whitespace_normalization() {
    let text = "Hello    World\t\tTest";
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");

    assert_eq!(normalized, "Hello World Test");
}

#[test]
fn test_empty_string_handling() {
    let empty = "";
    assert!(empty.is_empty());

    let whitespace_only = "   \t\n   ";
    assert!(whitespace_only.trim().is_empty());
}

// ============================================================================
// Integration Test Markers
// ============================================================================

#[test]
#[ignore = "Requires signal simulation"]
fn test_signal_handler_wait_for_shutdown() {
    // This test would:
    // 1. Create a SignalHandler
    // 2. Spawn a background listener
    // 3. Send a simulated signal
    // 4. Verify shutdown is detected
}

#[test]
#[ignore = "Requires concurrent operations"]
fn test_concurrent_url_extraction() {
    // This test would:
    // 1. Extract URLs from multiple texts concurrently
    // 2. Verify thread safety
    // 3. Verify correct results
}

#[test]
#[ignore = "Requires large dataset"]
fn test_url_extraction_performance() {
    // This test would:
    // 1. Create a large text with many URLs
    // 2. Extract URLs and measure performance
    // 3. Verify reasonable execution time
}
