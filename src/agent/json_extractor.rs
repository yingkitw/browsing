//! JSON extraction utility for parsing LLM responses
//!
//! This module provides utilities for extracting JSON from LLM responses,
//! handling various formats including markdown code blocks and plain JSON.

use regex::Regex;

/// JSON extractor for parsing LLM responses
///
/// This utility extracts JSON from responses that may contain markdown code blocks
/// or other formatting around the JSON content.
pub struct JSONExtractor;

impl JSONExtractor {
    /// Create a new JSON extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract JSON from a response string
    ///
    /// Attempts multiple extraction strategies in order:
    /// 1. JSON in markdown code blocks with "```json" marker
    /// 2. JSON in generic markdown code blocks with "```" marker
    /// 3. Standalone JSON object in the text
    /// 4. Fallback to the original response if no JSON found
    pub fn extract_from_response(&self, response: &str) -> String {
        const PATTERNS: &[(&str, bool)] = &[
            // Pattern: whether to use DOTALL (match across newlines)
            (r#"```json\s*([\s\S]*?)\s*```"#, true),   // ```json ... ```
            (r#"```\s*(\{[\s\S]*?\})\s*```"#, true),    // ```{...}```
            (r#"```\s*(\[[\s\S]*?\])\s*```"#, true),    // ```[...]```
            (r#"(\{[^{}]*\}(?:\s*\{[^{}]*\})*)"#, false), // Simple JSON objects
            (r#"(\[[^\[\]]*\](?:\s*\[[^\[\]]*\])*)"#, false), // Simple JSON arrays
        ];

        for (pattern, dotall) in PATTERNS {
            if let Some(json) = self.try_extract(response, pattern, *dotall) {
                return json;
            }
        }

        // Fallback: return original response
        response.to_string()
    }

    /// Try to extract JSON using a specific pattern
    fn try_extract(&self, text: &str, pattern: &str, dotall: bool) -> Option<String> {
        let regex = if dotall {
            Regex::new(pattern).ok()?
        } else {
            Regex::new(pattern).ok()?
        };

        if let Some(captures) = regex.captures(text) {
            if let Some(matched) = captures.get(1) {
                let extracted = matched.as_str().trim();
                // Validate that it looks like JSON
                if self.looks_like_json(extracted) {
                    return Some(extracted.to_string());
                }
            }
        }

        None
    }

    /// Check if a string looks like JSON
    fn looks_like_json(&self, s: &str) -> bool {
        let trimmed = s.trim();
        trimmed.starts_with('{') && trimmed.ends_with('}')
            || trimmed.starts_with('[') && trimmed.ends_with(']')
    }

    /// Try to find a JSON object in the text
    fn find_json_object(&self, text: &str) -> Option<String> {
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                if end > start {
                    return Some(text[start..=end].to_string());
                }
            }
        }
        None
    }
}

impl Default for JSONExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_code_block() {
        let extractor = JSONExtractor::new();
        let response = r#"Here's the JSON:
        ```json
        {"key": "value"}
        ```
        "#;
        let result = extractor.extract_from_response(response);
        assert!(result.contains("key") && result.contains("value"));
    }

    #[test]
    fn test_extract_plain_json() {
        let extractor = JSONExtractor::new();
        let response = r#"The result is {"key": "value"}"#;
        let result = extractor.extract_from_response(response);
        assert!(result.contains("key"));
    }

    #[test]
    fn test_fallback_to_original() {
        let extractor = JSONExtractor::new();
        let response = "No JSON here, just plain text";
        let result = extractor.extract_from_response(response);
        assert_eq!(result, "No JSON here, just plain text");
    }
}
