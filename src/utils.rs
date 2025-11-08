//! Utility functions for browser-use-rs

use regex::Regex;
use url::Url;

lazy_static::lazy_static! {
    pub static ref URL_PATTERN: Regex = Regex::new(
        r#"https?://[^\s<>"']+|www\.[^\s<>"']+|[^\s<>"']+\.[a-z]{2,}(?:/[^\s<>"']*)?"#
    ).unwrap();
}

/// Check if a string is a URL
pub fn is_url(s: &str) -> bool {
    URL_PATTERN.is_match(s) || Url::parse(s).is_ok()
}

/// Extract URLs from text
pub fn extract_urls(text: &str) -> Vec<String> {
    URL_PATTERN
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Match a URL against a domain pattern (supports wildcards)
pub fn match_url_with_domain_pattern(url: &str, pattern: &str) -> bool {
    use url::Url;
    
    let url = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    
    let host = match url.host_str() {
        Some(h) => h,
        None => return false,
    };
    
    // Simple pattern matching - supports * wildcard
    if pattern.contains('*') {
        let pattern_parts: Vec<&str> = pattern.split('*').collect();
        if pattern_parts.len() == 2 {
            // Pattern like "*.example.com"
            let prefix = pattern_parts[0];
            let suffix = pattern_parts[1];
            if prefix.is_empty() {
                host.ends_with(suffix)
            } else if suffix.is_empty() {
                host.starts_with(prefix)
            } else {
                host.starts_with(prefix) && host.ends_with(suffix)
            }
        } else {
            // More complex patterns - simple contains check
            host.contains(pattern.trim_matches('*'))
        }
    } else {
        // Exact match or suffix match
        host == pattern || host.ends_with(&format!(".{}", pattern))
    }
}

/// Check if URL is a new tab page
pub fn is_new_tab_page(url: &str) -> bool {
    url == "about:blank" || url == "chrome://newtab/" || url.is_empty()
}

