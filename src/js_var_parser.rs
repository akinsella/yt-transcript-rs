/// JavaScript variable parsing from HTML content.
///
/// This module provides functionality to extract and parse JavaScript variables
/// embedded in HTML content, which is essential for extracting transcript data
/// from YouTube pages. YouTube stores various metadata and configuration options
/// in JavaScript variables within the page source.
///
/// The parser supports multiple extraction strategies:
/// 1. Character-by-character parsing (primary method, more robust)
/// 2. Regular expression fallback (used when the primary method fails)
///
/// This module is primarily used internally to extract transcript metadata from
/// the YouTube video page.
use regex::Regex;

use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};

/// Parser for extracting JavaScript variables from HTML content.
///
/// This parser is designed to extract and parse JavaScript object literals
/// assigned to variables in HTML source code, specifically targeting YouTube's
/// page structure. It handles nested objects, escaping, and proper JSON parsing.
///
/// # Features
///
/// * Extracts JavaScript variables by name from HTML content
/// * Handles nested objects with proper brace matching
/// * Supports both character-by-character parsing and regex fallbacks
/// * Converts extracted JavaScript objects to Rust values via serde_json
///
/// # Example
///
/// ```rust,no_run
/// # use yt_transcript_rs::js_var_parser::JsVarParser;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a parser for the "ytInitialPlayerResponse" variable
/// let parser = JsVarParser::new("ytInitialPlayerResponse");
///
/// // HTML content containing the JavaScript variable
/// let html = r#"
///   <script>
///     var ytInitialPlayerResponse = {"captions": {"playerCaptionsTracklistRenderer":
///       {"captionTracks": [{"baseUrl": "https://example.com", "name": {"simpleText": "English"}}]}}};
///   </script>
/// "#;
///
/// // Parse the variable
/// let json = parser.parse(html, "dQw4w9WgXcQ")?;
///
/// // Access extracted data
/// if let Some(captions) = json.get("captions") {
///     println!("Found captions data: {}", captions);
/// }
/// # Ok(())
/// # }
/// ```
pub struct JsVarParser {
    /// The name of the JavaScript variable to extract
    var_name: String,
}

impl JsVarParser {
    /// Creates a new JavaScript variable parser for the specified variable name.
    ///
    /// # Parameters
    ///
    /// * `var_name` - The name of the JavaScript variable to extract (e.g., "ytInitialPlayerResponse")
    ///
    /// # Returns
    ///
    /// A new `JsVarParser` instance configured to extract the specified variable.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use yt_transcript_rs::js_var_parser::JsVarParser;
    /// // Create a parser for YouTube's initial player response
    /// let player_response_parser = JsVarParser::new("ytInitialPlayerResponse");
    ///
    /// // Create a parser for YouTube's initial data
    /// let initial_data_parser = JsVarParser::new("ytInitialData");
    /// ```
    pub fn new(var_name: &str) -> Self {
        Self {
            var_name: var_name.to_string(),
        }
    }

    /// Parses a JavaScript variable from HTML content and converts it to a JSON value.
    ///
    /// This method tries multiple parsing strategies:
    /// 1. First, it attempts a character-by-character approach for precise extraction
    /// 2. If that fails, it falls back to regular expression patterns
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content containing the JavaScript variable
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<serde_json::Value, CouldNotRetrieveTranscript>` - The parsed JSON value or an error
    ///
    /// # Errors
    ///
    /// Returns a `CouldNotRetrieveTranscript` error with `YouTubeDataUnparsable` reason when:
    /// - The variable is not found in the HTML
    /// - The variable value cannot be parsed as valid JSON
    /// - The braces in the JavaScript object are mismatched
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::js_var_parser::JsVarParser;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let parser = JsVarParser::new("ytInitialPlayerResponse");
    /// let html = r#"<script>var ytInitialPlayerResponse = {"captions": {"available": true}};</script>"#;
    ///
    /// match parser.parse(html, "dQw4w9WgXcQ") {
    ///     Ok(json) => {
    ///         println!("Successfully extracted variable: {}", json);
    ///         
    ///         // Access nested properties
    ///         if let Some(captions) = json.get("captions") {
    ///             if let Some(available) = captions.get("available") {
    ///                 println!("Captions available: {}", available);
    ///             }
    ///         }
    ///     },
    ///     Err(e) => {
    ///         println!("Failed to parse: {:?}", e.reason);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Parses a JavaScript variable using a character-by-character approach.
    ///
    /// This method mimics the character-by-character approach used in the Python
    /// implementation. It carefully tracks braces, quotes, and escape sequences
    /// to extract nested JavaScript objects correctly.
    ///
    /// The approach:
    /// 1. Finds the variable name in the HTML
    /// 2. Locates the opening brace of the object
    /// 3. Tracks nested braces to find the matching closing brace
    /// 4. Handles string literals and escape sequences properly
    /// 5. Parses the extracted object as JSON
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content containing the JavaScript variable
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<serde_json::Value, CouldNotRetrieveTranscript>` - The parsed JSON value or an error
    ///
    /// # Errors
    ///
    /// Returns a `CouldNotRetrieveTranscript` error with `YouTubeDataUnparsable` reason when:
    /// - The variable name is not found in the HTML
    /// - No opening brace is found after the variable name
    /// - The HTML ends before finding a matching closing brace
    /// - The extracted text is not valid JSON
    ///
    /// # Note
    ///
    /// This is an internal method used by `parse()` and typically should not
    /// be called directly.
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
                    reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                        format!("JavaScript variable '{}' not found in HTML", self.var_name),
                    )),
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
                        reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                            format!(
                                "Opening brace not found after JavaScript variable '{}'",
                                self.var_name
                            ),
                        )),
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
                        reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                            "Unexpected end of HTML while parsing JavaScript variable".to_string(),
                        )),
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
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                    "Extracted JavaScript variable is not valid JSON".to_string(),
                )),
            }),
        }
    }

    /// Parses a JavaScript variable using regular expressions as a fallback method.
    ///
    /// This method tries multiple regex patterns to extract the variable value when
    /// the character-by-character approach fails. It's less precise but can handle
    /// some edge cases.
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content containing the JavaScript variable
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<serde_json::Value, CouldNotRetrieveTranscript>` - The parsed JSON value or an error
    ///
    /// # Errors
    ///
    /// Returns a `CouldNotRetrieveTranscript` error with `YouTubeDataUnparsable` reason when:
    /// - None of the regex patterns match the HTML content
    /// - The matched content cannot be parsed as valid JSON
    ///
    /// # Note
    ///
    /// This is an internal method used by `parse()` as a fallback when the primary
    /// parsing method fails. It typically should not be called directly.
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
        Err(CouldNotRetrieveTranscript {
            video_id: video_id.to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                format!(
                    "Could not find or parse JavaScript variable '{}' using regex patterns",
                    self.var_name
                ),
            )),
        })
    }
}
