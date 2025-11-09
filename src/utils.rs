//! Utility functions

pub mod signal;

use regex::Regex;
use url::Url;

/// Extract URLs from text
pub fn extract_urls(text: &str) -> Vec<String> {
    // Simplified URL pattern - match http/https URLs
    // Match until whitespace
    let url_pattern = Regex::new(
        r"(?i)\bhttps?://[^\s]+",
    )
    .unwrap();
    
    url_pattern
        .find_iter(text)
        .filter_map(|m| {
            let url_str = m.as_str();
            // Try to parse and normalize
            url_str
                .parse::<Url>()
                .ok()
                .map(|u| u.to_string())
        })
        .collect()
}

/// Match a URL against a domain pattern
/// Supports patterns like:
/// - "example.com" - exact match
/// - "*.example.com" - subdomain match
/// - "http*://example.com" - protocol match
pub fn match_url_with_domain_pattern(url: &str, pattern: &str) -> bool {
    if pattern.is_empty() || url.is_empty() {
        return false;
    }
    
    // Parse URL
    let parsed_url = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    
    let url_host = parsed_url.host_str().unwrap_or("");
    let url_scheme = parsed_url.scheme();
    
    // Handle protocol pattern (http*://example.com)
    if pattern.contains("://") {
        let parts: Vec<&str> = pattern.split("://").collect();
        if parts.len() == 2 {
            let scheme_pattern = parts[0];
            let domain_pattern = parts[1];
            
            // Check scheme match
            if scheme_pattern.ends_with('*') {
                // Pattern like "http*" should match "http" and "https"
                let prefix = &scheme_pattern[..scheme_pattern.len() - 1];
                if !url_scheme.starts_with(prefix) {
                    return false;
                }
            } else if scheme_pattern != url_scheme {
                return false;
            }
            
            // Check domain match
            return match_domain_pattern(url_host, domain_pattern);
        }
    }
    
    // Handle domain pattern only
    match_domain_pattern(url_host, pattern)
}

fn match_domain_pattern(host: &str, pattern: &str) -> bool {
    if pattern == host {
        return true;
    }
    
    // Handle wildcard pattern (*.example.com)
    if pattern.starts_with("*.") {
        let suffix = &pattern[2..];
        // For *.example.com, host should end with .example.com or be exactly example.com
        // But not match example.com itself (only subdomains)
        if host == suffix {
            return false; // *.example.com should not match example.com itself
        }
        return host.ends_with(&format!(".{}", suffix));
    }
    
    // Handle exact match
    pattern == host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_urls() {
        let text = "Visit https://example.com and http://test.org/page";
        let urls = extract_urls(text);
        // URLs may be normalized by Url parser, so check they contain the domains
        assert!(urls.len() >= 1);
        assert!(urls.iter().any(|u| u.contains("example.com")));
        assert!(urls.iter().any(|u| u.contains("test.org")));
    }

    #[test]
    fn test_extract_urls_with_www() {
        let text = "Check https://www.example.com for details";
        let urls = extract_urls(text);
        assert!(!urls.is_empty());
        assert!(urls[0].starts_with("https://"));
    }

    #[test]
    fn test_extract_urls_no_urls() {
        let text = "This is just plain text with no URLs";
        let urls = extract_urls(text);
        assert!(urls.is_empty());
    }

    #[test]
    fn test_match_url_with_domain_pattern() {
        use super::match_url_with_domain_pattern;
        
        // Exact match
        assert!(match_url_with_domain_pattern("https://example.com", "example.com"));
        assert!(match_url_with_domain_pattern("https://example.com/path", "example.com"));
        
        // Wildcard subdomain
        assert!(match_url_with_domain_pattern("https://www.example.com", "*.example.com"));
        assert!(match_url_with_domain_pattern("https://api.example.com", "*.example.com"));
        assert!(!match_url_with_domain_pattern("https://example.com", "*.example.com"));
        
        // Protocol pattern
        assert!(match_url_with_domain_pattern("https://example.com", "http*://example.com"));
        assert!(match_url_with_domain_pattern("http://example.com", "http*://example.com"));
        assert!(!match_url_with_domain_pattern("ftp://example.com", "http*://example.com"));
    }
}
