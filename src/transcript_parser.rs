use anyhow::Result;
use regex::Regex;

use crate::models::FetchedTranscriptSnippet;

/// # TranscriptParser
///
/// Parses YouTube transcript XML data into structured transcript snippets.
///
/// This parser handles YouTube's XML format for transcripts and can:
/// - Extract text content, timing information, and duration
/// - Optionally preserve specified HTML formatting tags
/// - Remove unwanted HTML tags
///
/// ## Usage Example
///
/// ```rust,no_run
/// use yt_transcript_rs::transcript_parser::TranscriptParser;
///
/// // Create a parser that strips all formatting
/// let parser = TranscriptParser::new(false);
///
/// // Or create a parser that preserves certain formatting tags (bold, italic, etc.)
/// let formatting_parser = TranscriptParser::new(true);
///
/// // Parse XML transcript data
/// let xml = r#"
///     <transcript>
///         <text start="0.0" dur="1.0">This is a transcript</text>
///         <text start="1.0" dur="1.5">With multiple entries</text>
///     </transcript>
/// "#;
///
/// let snippets = parser.parse(xml).unwrap();
/// ```
#[derive(Debug)]
/// Parser for YouTube transcript XML data
pub struct TranscriptParser {
    /// Whether to preserve specified formatting tags in the transcript
    preserve_formatting: bool,
    /// Regex pattern for matching HTML tags
    html_regex: Regex,
}

impl TranscriptParser {
    /// List of HTML formatting tags that can be preserved when `preserve_formatting` is enabled.
    ///
    /// These tags are commonly used for text formatting and can be preserved in the transcript:
    /// - strong, b: Bold text
    /// - em, i: Italic text
    /// - mark: Highlighted text
    /// - small: Smaller text
    /// - del: Deleted/strikethrough text
    /// - ins: Inserted/underlined text
    /// - sub: Subscript
    /// - sup: Superscript
    const FORMATTING_TAGS: [&'static str; 10] = [
        "strong", // important (bold)
        "em",     // emphasized (italic)
        "b",      // bold
        "i",      // italic
        "mark",   // highlighted
        "small",  // smaller
        "del",    // deleted/strikethrough
        "ins",    // inserted/underlined
        "sub",    // subscript
        "sup",    // superscript
    ];

    /// Creates a new transcript parser.
    ///
    /// # Parameters
    ///
    /// * `preserve_formatting` - If `true`, certain HTML formatting tags (like bold, italic) will be
    ///   kept in the transcript. If `false`, all HTML tags will be removed.
    ///
    /// # Returns
    ///
    /// A new `TranscriptParser` instance configured according to the formatting preference.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::transcript_parser::TranscriptParser;
    /// // Create a parser that removes all HTML tags
    /// let plain_parser = TranscriptParser::new(false);
    ///
    /// // Create a parser that preserves formatting tags
    /// let formatted_parser = TranscriptParser::new(true);
    /// ```
    pub fn new(preserve_formatting: bool) -> Self {
        // Use a simple regex that matches all HTML tags - we'll handle the preservation logic separately
        let html_regex = Regex::new(r"<[^>]*>").unwrap();

        Self {
            preserve_formatting,
            html_regex,
        }
    }

    /// Parses YouTube transcript XML into a collection of transcript snippets.
    ///
    /// This method takes raw XML data from YouTube transcripts and processes it into
    /// structured `FetchedTranscriptSnippet` objects that contain:
    /// - Text content (with optional formatting)
    /// - Start time in seconds
    /// - Duration in seconds
    ///
    /// # Parameters
    ///
    /// * `raw_data` - The raw XML string containing transcript data from YouTube
    ///
    /// # Returns
    ///
    /// * `Result<Vec<FetchedTranscriptSnippet>, anyhow::Error>` - A vector of transcript snippets on success,
    ///   or an error if parsing fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The XML data is malformed and cannot be parsed
    /// - Required attributes are missing or invalid
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::transcript_parser::TranscriptParser;
    /// # let xml = "<transcript><text start=\"0.0\" dur=\"1.0\">Hello</text></transcript>";
    /// let parser = TranscriptParser::new(false);
    /// let snippets = parser.parse(xml).unwrap();
    ///
    /// for snippet in snippets {
    ///     println!("[{:.1}-{:.1}s] {}",
    ///         snippet.start,
    ///         snippet.start + snippet.duration,
    ///         snippet.text);
    /// }
    /// ```
    pub fn parse(&self, raw_data: &str) -> Result<Vec<FetchedTranscriptSnippet>, anyhow::Error> {
        let mut snippets = Vec::new();

        // Parse XML using roxmltree
        let document = roxmltree::Document::parse(raw_data)?;
        let transcript_elem = document.root_element();

        // Process each text element in the transcript
        for text_elem in transcript_elem
            .children()
            .filter(|n| n.has_tag_name("text"))
        {
            // Extract start time (defaults to 0.0 if missing or invalid)
            let start = text_elem
                .attribute("start")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            // Extract duration (defaults to 0.0 if missing or invalid)
            let duration = text_elem
                .attribute("dur")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            // Get text directly from the node
            let text = if let Some(text) = text_elem.text() {
                text.to_string()
            } else {
                String::new()
            };

            // Process the text based on formatting preferences
            let text = if self.preserve_formatting {
                // Keep specified formatting tags, remove others
                self.process_with_formatting(&text)
            } else {
                // Remove all HTML tags
                self.html_regex.replace_all(&text, "").to_string()
            };

            // Create and add the snippet to our collection
            snippets.push(FetchedTranscriptSnippet {
                text,
                start,
                duration,
            });
        }

        Ok(snippets)
    }

    /// Processes text to preserve only specific allowed HTML formatting tags.
    ///
    /// This method:
    /// 1. Identifies all HTML tags in the text
    /// 2. Keeps only the tags listed in `FORMATTING_TAGS`
    /// 3. Removes all other HTML tags
    ///
    /// # Parameters
    ///
    /// * `text` - The text containing HTML tags to process
    ///
    /// # Returns
    ///
    /// A string with only the allowed formatting tags preserved and all others removed.
    ///
    /// # Example (internal usage)
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::transcript_parser::TranscriptParser;
    /// # let parser = TranscriptParser::new(true);
    /// # let input = "<b>Bold</b> and <span>span</span> and <i>italic</i>";
    /// // Only <b> and <i> tags would be preserved, <span> would be removed
    /// let result = parser.process_with_formatting(input);
    /// // Result would be "<b>Bold</b> and span and <i>italic</i>"
    /// ```
    pub fn process_with_formatting(&self, text: &str) -> String {
        let mut result = text.to_string();

        // First pass: collect all HTML tags
        let tag_matches: Vec<(usize, usize, String)> = self
            .html_regex
            .find_iter(text)
            .map(|m| {
                let tag_content = &text[m.start()..m.end()];
                (m.start(), m.end(), tag_content.to_string())
            })
            .collect();

        // Second pass: only keep allowed formatting tags
        let mut offset = 0;
        for (start, end, tag) in tag_matches {
            let adjusted_start = start - offset;
            let adjusted_end = end - offset;

            // Check if this tag should be preserved based on our allowed list
            let keep_tag = Self::FORMATTING_TAGS.iter().any(|&allowed_tag| {
                let open_tag = format!("<{}", allowed_tag);
                let close_tag = format!("</{}", allowed_tag);
                tag.starts_with(&open_tag) || tag.starts_with(&close_tag)
            });

            if !keep_tag {
                // Remove tag that's not in the allowed list
                result.replace_range(adjusted_start..adjusted_end, "");
                offset += adjusted_end - adjusted_start;
            }
        }

        result
    }
}
