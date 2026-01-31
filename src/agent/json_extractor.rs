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
    /// 3. Complete JSON object in the text using brace counting
    /// 4. Fallback to the original response if no JSON found
    pub fn extract_from_response(&self, response: &str) -> String {
        const PATTERNS: &[(&str, bool)] = &[
            // Pattern: whether to use DOTALL (match across newlines)
            (r#"```json\s*([\s\S]*?)\s*```"#, true),   // ```json ... ```
            (r#"```\s*(\{[\s\S]*?\})\s*```"#, true),    // ```{...}```
            (r#"```\s*(\[[\s\S]*?\])\s*```"#, true),    // ```[...]```
        ];

        for (pattern, dotall) in PATTERNS {
            if let Some(json) = self.try_extract(response, pattern, *dotall) {
                return json;
            }
        }

        // Try to extract a complete JSON object using brace counting
        if let Some(json) = self.find_complete_json_object(response) {
            return json;
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

    /// Try to find a complete JSON object in the text using brace counting
    /// This handles nested objects and arrays correctly
    fn find_complete_json_object(&self, text: &str) -> Option<String> {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        // Find the first opening brace
        let mut start = None;
        for (i, &c) in chars.iter().enumerate() {
            if c == '{' {
                start = Some(i);
                break;
            }
        }

        let start = start?;

        // Count braces to find the matching closing brace
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for i in start..len {
            let c = chars[i];

            if escape_next {
                escape_next = false;
                continue;
            }

            if c == '\\' {
                escape_next = true;
                continue;
            }

            if c == '"' {
                in_string = !in_string;
                continue;
            }

            if !in_string {
                if c == '{' {
                    brace_count += 1;
                } else if c == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        // Found the matching closing brace
                        let json_str: String = chars[start..=i].iter().collect();
                        return Some(json_str);
                    }
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
    fn test_extract_nested_json() {
        let extractor = JSONExtractor::new();
        let response = r#"Here's the result: {"action": [{"type": "navigate", "params": {"url": "https://example.com"}}]}"#;
        let result = extractor.extract_from_response(response);
        assert!(result.contains("action"));
        assert!(result.contains("navigate"));
    }

    #[test]
    fn test_fallback_to_original() {
        let extractor = JSONExtractor::new();
        let response = "No JSON here, just plain text";
        let result = extractor.extract_from_response(response);
        assert_eq!(result, "No JSON here, just plain text");
    }
}
