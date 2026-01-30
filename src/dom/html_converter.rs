//! HTML to markdown conversion
//!
//! This module handles conversion of HTML content to markdown format.

use regex::Regex;

/// HTML to markdown converter
pub struct HTMLConverter;

impl HTMLConverter {
    /// Convert HTML to markdown
    pub fn html_to_markdown(html: &str) -> crate::error::Result<String> {
        let cleaned_html = Self::remove_script_style_tags(html);
        let text = Self::extract_text(&cleaned_html);

        // Basic markdown formatting
        let mut markdown = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                markdown.push_str(trimmed);
                markdown.push_str("\n\n");
            }
        }

        Ok(markdown.trim().to_string())
    }

    /// Remove script and style tags from HTML
    fn remove_script_style_tags(html: &str) -> String {
        let script_re = Regex::new(r"(?s)<script[^>]*>.*?</script>").unwrap();
        let style_re = Regex::new(r"(?s)<style[^>]*>.*?</style>").unwrap();

        let cleaned = script_re.replace_all(html, "");
        let cleaned = style_re.replace_all(&cleaned, "");
        cleaned.to_string()
    }

    /// Extract text content from HTML
    pub fn extract_text(html: &str) -> String {
        let tag_re = Regex::new(r"<[^>]+>").unwrap();
        let text = tag_re.replace_all(html, " ");

        // Clean up whitespace
        let whitespace_re = Regex::new(r"\s+").unwrap();
        let cleaned = whitespace_re.replace_all(&text, " ");
        cleaned.trim().to_string()
    }

    /// Extract page content from HTML
    pub fn extract_page_content(html: &str) -> crate::error::Result<String> {
        Self::html_to_markdown(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_script_tags() {
        let html = r#"<html><script>alert('test');</script><p>Hello</p></html>"#;
        let cleaned = HTMLConverter::remove_script_style_tags(html);
        assert!(!cleaned.contains("<script>"));
        assert!(cleaned.contains("<p>"));
    }

    #[test]
    fn test_extract_text() {
        let html = r#"<p>Hello <strong>world</strong></p>"#;
        let text = HTMLConverter::extract_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }
}
