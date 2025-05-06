use regex::Regex;
use std::fs;

use crate::errors::{
    CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason,
};

/// Parser for JavaScript variables in HTML
pub struct JsVarParser {
    var_name: String,
}

impl JsVarParser {
    pub fn new(var_name: &str) -> Self {
        Self {
            var_name: var_name.to_string(),
        }
    }

    /// Parse a JavaScript variable from HTML
    pub fn parse(
        &self,
        html: &str,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        // First try to find the variable using a character-by-character approach
        // (similar to the Python implementation)
        if let Ok(json_value) = self.parse_char_by_char(html, video_id) {
            return Ok(json_value);
        }

        // Fall back to regex as a backup strategy
        self.parse_with_regex(html, video_id)
    }

    /// Parse JavaScript variable using a character-by-character approach
    /// This closely follows the Python implementation's character-by-character approach
    fn parse_char_by_char(
        &self,
        html: &str,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        // Step 1: Split by "var {var_name}"
        let var_marker = format!("var {}", self.var_name);
        let parts: Vec<&str> = html.split(&var_marker).collect();

        if parts.len() <= 1 {
            // Try with just the var name (without 'var' prefix)
            let parts: Vec<&str> = html.split(&self.var_name).collect();
            if parts.len() <= 1 {
                return Err(CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable),
                });
            }
        }

        // Take the part after the variable name
        let after_var = if parts.len() > 1 { parts[1] } else { "" };

        // Step 2: Create iterator over the characters after the variable name
        let mut chars = after_var.chars();

        // Step 3: Find the opening brace
        loop {
            match chars.next() {
                Some('{') => break,
                Some(_) => continue,
                None => {
                    return Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable),
                    });
                }
            }
        }

        // Step 4: Find the matching closing brace
        let mut json_chars = vec!['{'];
        let mut depth = 1;
        let mut escaped = false;
        let mut in_quotes = false;

        while depth > 0 {
            match chars.next() {
                Some(c) => {
                    json_chars.push(c);

                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        in_quotes = !in_quotes;
                    } else if !in_quotes {
                        if c == '{' {
                            depth += 1;
                        } else if c == '}' {
                            depth -= 1;
                        }
                    }
                }
                None => {
                    // Unexpected end of string
                    return Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable),
                    });
                }
            }
        }

        // Step 5: Parse the extracted JSON string
        let json_str: String = json_chars.into_iter().collect();

        match serde_json::from_str(&json_str) {
            Ok(json) => Ok(json),
            Err(_) => Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable),
            }),
        }
    }

    /// Parse JavaScript variable using regex as a fallback
    fn parse_with_regex(
        &self,
        html: &str,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        // Common patterns for finding JavaScript variables
        let patterns = [
            format!(r"{}\ =\ (.*?);</script>", regex::escape(&self.var_name)),
            format!(r"{}=(.*?);</script>", regex::escape(&self.var_name)),
            format!(r#"{} = (.*?);"#, regex::escape(&self.var_name)),
            format!(r#"{}=(.*?);"#, regex::escape(&self.var_name)),
        ];

        for pattern in &patterns {
            let re = match Regex::new(pattern) {
                Ok(re) => re,
                Err(_) => continue,
            };

            if let Some(cap) = re.captures(html) {
                if let Some(json_str) = cap.get(1) {
                    match serde_json::from_str(json_str.as_str()) {
                        Ok(json) => return Ok(json),
                        Err(_) => continue,
                    }
                }
            }
        }

        // If we get here, we couldn't find or parse the variable
        fs::write("/tmp/html_debug.txt", html).unwrap_or_default();

        Err(CouldNotRetrieveTranscript {
            video_id: video_id.to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable),
        })
    }
}
