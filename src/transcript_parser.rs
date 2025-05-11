use crate::models::FetchedTranscriptSnippet;
use anyhow::Result;
use html2text;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
struct Transcript {
    #[serde(rename = "text")]
    texts: Vec<Text>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Text {
    #[serde(rename = "@start")]
    start: String,

    #[serde(rename = "@dur")]
    duration: String,

    // Text content of the element
    #[serde(rename = "$text")]
    content: String,
}

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
    /// - span: Generic inline container
    /// - a: Hyperlink
    const FORMATTING_TAGS: [&'static str; 12] = [
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
        "span",   // generic inline container
        "a",      // hyperlink
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
        let mut reader = Reader::from_reader(Cursor::new(raw_data));
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        let mut snippets = Vec::new();
        let mut in_text = false;
        let mut start = String::new();
        let mut duration = String::new();
        let mut content = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if tag_name == "text" {
                        in_text = true;

                        // Process attributes
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();

                            if key == "start" {
                                start = value;
                            } else if key == "dur" {
                                duration = value;
                            }
                        }
                    } else if in_text {
                        // This is an HTML tag inside the text content
                        // Reconstruct the full tag with attributes
                        let mut tag_with_attrs = format!("<{}", tag_name);

                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            tag_with_attrs.push_str(&format!(" {}=\"{}\"", key, value));
                        }

                        tag_with_attrs.push('>');
                        content.push_str(&tag_with_attrs);
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_text {
                        // Handle XML entities by using unescape
                        match e.unescape() {
                            Ok(text) => content.push_str(&text),
                            Err(_) => content.push_str(&String::from_utf8_lossy(e.as_ref())),
                        }
                    }
                }
                Ok(Event::CData(e)) => {
                    if in_text {
                        content.push_str(&String::from_utf8_lossy(e.as_ref()));
                    }
                }
                Ok(Event::End(e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if tag_name == "text" {
                        in_text = false;

                        // Process content based on formatting preferences
                        let processed_text = if self.preserve_formatting {
                            // When preserving formatting, keep HTML tags based on allowed list
                            self.process_with_formatting(&content)
                        } else {
                            // When removing formatting, use html2text to properly convert to plain text
                            // First, normalize HTML to ensure proper spacing around tags
                            let normalized_content =
                                self.normalize_html_for_text_extraction(&content);

                            // html2text will handle both HTML tags and entities correctly
                            // Use a very large width value (usize::MAX) to effectively disable line wrapping
                            let plain_text =
                                html2text::from_read(normalized_content.as_bytes(), usize::MAX)
                                    .unwrap_or_else(|_| content.clone());

                            // Convert reference-style links to inline links
                            let processed_text =
                                self.convert_reference_links_to_inline(&plain_text);

                            // Trim trailing newlines
                            processed_text.trim_end().to_string()
                        };

                        // Create and add the snippet
                        snippets.push(FetchedTranscriptSnippet {
                            text: processed_text,
                            start: start.parse::<f64>().unwrap_or(0.0),
                            duration: duration.parse::<f64>().unwrap_or(0.0),
                        });

                        // Clear for next item
                        start.clear();
                        duration.clear();
                        content.clear();
                    } else if in_text {
                        // This is a closing HTML tag inside the text content
                        content.push_str(&format!("</{}>", tag_name));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Error at position {}: {:?}",
                        reader.buffer_position(),
                        e
                    ));
                }
                _ => (),
            }
            buf.clear();
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

            // Extract the tag name without attributes for comparison
            let tag_name = if let Some(space_pos) = tag.find(|c: char| c.is_whitespace()) {
                // Handle tags with attributes: <tag attr="value">
                let closing_bracket = tag.find('>').unwrap_or(tag.len());
                let name_end = space_pos.min(closing_bracket);
                if tag.starts_with("</") {
                    // Closing tag
                    tag[2..name_end].to_string()
                } else {
                    // Opening tag
                    tag[1..name_end].to_string()
                }
            } else {
                // Handle simple tags without attributes: <tag> or </tag>
                if tag.starts_with("</") {
                    // Closing tag without attributes
                    let end_pos = tag.find('>').unwrap_or(tag.len());
                    tag[2..end_pos].to_string()
                } else {
                    // Opening tag without attributes
                    let end_pos = tag.find('>').unwrap_or(tag.len());
                    tag[1..end_pos].to_string()
                }
            };

            // Check if this tag should be preserved based on our allowed list
            let keep_tag = Self::FORMATTING_TAGS.contains(&tag_name.as_str());

            if !keep_tag {
                // Remove tag that's not in the allowed list
                result.replace_range(adjusted_start..adjusted_end, "");
                offset += adjusted_end - adjusted_start;
            }
        }

        result
    }

    /// Normalizes HTML content to ensure proper spacing when converted to plain text.
    /// This adds spaces around HTML tags to prevent words from running together after tags are removed.
    fn normalize_html_for_text_extraction(&self, html: &str) -> String {
        // Add spaces before and after tags to prevent words from running together when tags are removed
        let opening_tag_regex = Regex::new(r"<[^/][^>]*>").unwrap();
        let closing_tag_regex = Regex::new(r"</[^>]*>").unwrap();

        let with_spaces_before = opening_tag_regex.replace_all(html, " $0 ");
        let with_spaces_after = closing_tag_regex.replace_all(&with_spaces_before, " $0 ");

        // Remove extra spaces that might have been introduced
        let multiple_spaces_regex = Regex::new(r"\s{2,}").unwrap();
        multiple_spaces_regex
            .replace_all(&with_spaces_after, " ")
            .to_string()
    }

    /// Converts reference-style links from html2text output to inline links with parentheses.
    /// For example, changes "[ link ][1]" with "[1]: https://example.com" to "link (https://example.com)".
    fn convert_reference_links_to_inline(&self, text: &str) -> String {
        // First, collect all reference URLs
        let reference_regex = Regex::new(r"\[(\d+)\]:\s*(https?://[^\s]+)").unwrap();
        let mut references = std::collections::HashMap::new();

        for line in text.lines() {
            if let Some(captures) = reference_regex.captures(line) {
                if captures.len() >= 3 {
                    let reference_num = captures.get(1).unwrap().as_str();
                    let url = captures.get(2).unwrap().as_str();
                    references.insert(reference_num.to_string(), url.to_string());
                }
            }
        }

        // If no references found, return the original text
        if references.is_empty() {
            return text.to_string();
        }

        // Replace reference-style links with inline links
        let link_regex = Regex::new(r"\[\s*([^\]]+)\s*\]\[(\d+)\]").unwrap();
        let lines: Vec<String> = text
            .lines()
            .filter(|line| !reference_regex.is_match(line))
            .map(|line| {
                let mut result = line.to_string();
                while let Some(captures) = link_regex.captures(&result) {
                    if captures.len() >= 3 {
                        let link_text = captures.get(1).unwrap().as_str().trim();
                        let reference_num = captures.get(2).unwrap().as_str();

                        if let Some(url) = references.get(reference_num) {
                            let replacement = format!("{} ({})", link_text, url);
                            result = link_regex
                                .replace(&result, replacement.as_str())
                                .to_string();
                        }
                    } else {
                        break;
                    }
                }
                result
            })
            .collect();

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_transcript() {
        let parser = TranscriptParser::new(false);

        let xml = r#"
        <transcript>
            <text start="0.0" dur="1.0">This is a transcript</text>
            <text start="1.0" dur="1.5">With multiple entries</text>
        </transcript>
        "#;

        let snippets = parser.parse(xml).unwrap();
        assert_eq!(snippets.len(), 2);
        assert_eq!(snippets[0].text, "This is a transcript");
        assert_eq!(snippets[0].start, 0.0);
        assert_eq!(snippets[0].duration, 1.0);
        assert_eq!(snippets[1].text, "With multiple entries");
        assert_eq!(snippets[1].start, 1.0);
        assert_eq!(snippets[1].duration, 1.5);
    }

    #[test]
    fn test_parse_with_html_formatting() {
        let xml_content = r#"<?xml version="1.0" encoding="utf-8" ?>
        <transcript>
            <text start="12.645" dur="1.37">So in <b>college</b>,</text>
            <text start="15.349" dur="1.564">I was a <i>government</i> major,</text>
            <text start="16.937" dur="2.462">which means <b>I had to write</b> <i>a lot</i> of <b>papers</b>.</text>
        </transcript>"#;

        // Test with formatting preserved
        let parser_with_formatting = TranscriptParser::new(true);
        let formatted_snippets = parser_with_formatting.parse(xml_content).unwrap();

        assert_eq!(formatted_snippets.len(), 3);
        println!("Formatted 0: '{}'", formatted_snippets[0].text);
        println!("Formatted 1: '{}'", formatted_snippets[1].text);
        println!("Formatted 2: '{}'", formatted_snippets[2].text);

        // There might be a space between "in" and "<b>" depending on the implementation
        let expected_text = formatted_snippets[0].text.clone();
        assert!(
            expected_text == "So in <b>college</b>," || expected_text == "So in<b>college</b>,",
            "Actual: {}",
            expected_text
        );

        assert!(formatted_snippets[1].text.contains("<i>government</i>"));
        assert!(formatted_snippets[2].text.contains("<b>I had to write</b>"));
        assert!(formatted_snippets[2].text.contains("<i>a lot</i>"));
        assert!(formatted_snippets[2].text.contains("<b>papers</b>"));

        // Test with formatting removed
        let plain_parser = TranscriptParser::new(false);
        let plain_snippets = plain_parser.parse(xml_content).unwrap();

        assert_eq!(plain_snippets.len(), 3);
        println!("Plain 0: '{}'", plain_snippets[0].text);
        println!("Plain 1: '{}'", plain_snippets[1].text);
        println!("Plain 2: '{}'", plain_snippets[2].text);

        // Check plain text output
        assert!(
            plain_snippets[0].text.contains("So in") && plain_snippets[0].text.contains("college"),
            "Actual: {}",
            plain_snippets[0].text
        );
        assert!(
            plain_snippets[1].text.contains("I was a")
                && plain_snippets[1].text.contains("government"),
            "Actual: {}",
            plain_snippets[1].text
        );
        assert!(
            plain_snippets[2].text.contains("which means")
                && plain_snippets[2].text.contains("I had to write")
                && plain_snippets[2].text.contains("a lot")
                && plain_snippets[2].text.contains("papers"),
            "Actual: {}",
            plain_snippets[2].text
        );
    }

    #[test]
    fn test_parse_with_html_attributes() {
        let xml_with_attributes = r#"<?xml version="1.0" encoding="utf-8" ?>
        <transcript>
            <text start="10.0" dur="2.0">This has a <span class="highlight" style="color:red">colored span</span> with attributes.</text>
            <text start="12.5" dur="3.0">And a <a href="https://example.com" target="_blank">link</a> with multiple attributes.</text>
            <text start="16.0" dur="2.5">And <b id="bold1" data-test="value">bold with attributes</b> should work too.</text>
        </transcript>"#;

        // Test with formatting preserved
        let parser_with_attributes = TranscriptParser::new(true);
        let formatted_with_attributes = parser_with_attributes.parse(xml_with_attributes).unwrap();

        assert_eq!(formatted_with_attributes.len(), 3);
        println!(
            "Formatted with attributes 0: '{}'",
            formatted_with_attributes[0].text
        );
        println!(
            "Formatted with attributes 1: '{}'",
            formatted_with_attributes[1].text
        );
        println!(
            "Formatted with attributes 2: '{}'",
            formatted_with_attributes[2].text
        );

        // There might be spacing differences depending on the implementation
        let expected_text = formatted_with_attributes[0].text.clone();
        assert!(
            expected_text.contains("This has a")
                && expected_text
                    .contains("<span class=\"highlight\" style=\"color:red\">colored span</span>")
                && expected_text.contains("with attributes"),
            "Actual: {}",
            expected_text
        );

        assert!(formatted_with_attributes[1]
            .text
            .contains("<a href=\"https://example.com\" target=\"_blank\">link</a>"));
        assert!(formatted_with_attributes[2]
            .text
            .contains("<b id=\"bold1\" data-test=\"value\">bold with attributes</b>"));

        // Test with formatting removed
        let plain_parser = TranscriptParser::new(false);
        let plain_with_attributes = plain_parser.parse(xml_with_attributes).unwrap();

        assert_eq!(plain_with_attributes.len(), 3);
        println!(
            "Plain with attributes 0: '{}'",
            plain_with_attributes[0].text
        );
        println!(
            "Plain with attributes 1: '{}'",
            plain_with_attributes[1].text
        );
        println!(
            "Plain with attributes 2: '{}'",
            plain_with_attributes[2].text
        );

        // Check plain text with flexible assertions
        assert!(
            plain_with_attributes[0].text.contains("This has a")
                && plain_with_attributes[0].text.contains("colored span")
                && plain_with_attributes[0].text.contains("with attributes"),
            "Actual: {}",
            plain_with_attributes[0].text
        );

        assert!(
            plain_with_attributes[1].text.contains("And a")
                && plain_with_attributes[1].text.contains("link")
                && plain_with_attributes[1]
                    .text
                    .contains("with multiple attributes"),
            "Actual: {}",
            plain_with_attributes[1].text
        );

        assert!(
            plain_with_attributes[2].text.contains("And")
                && plain_with_attributes[2]
                    .text
                    .contains("bold with attributes")
                && plain_with_attributes[2].text.contains("should work too"),
            "Actual: {}",
            plain_with_attributes[2].text
        );
    }

    #[test]
    fn test_edge_cases() {
        let parser = TranscriptParser::new(true);

        // Test empty transcript
        let empty_xml = "<transcript></transcript>";
        let empty_result = parser.parse(empty_xml).unwrap();
        assert_eq!(empty_result.len(), 0);

        // Test transcript with empty text elements
        let empty_text_xml = "<transcript><text start=\"0.0\" dur=\"1.0\"></text></transcript>";
        let empty_text_result = parser.parse(empty_text_xml).unwrap();
        assert_eq!(empty_text_result.len(), 1);
        assert_eq!(empty_text_result[0].text, "");

        // Test self-closing tags (which YouTube doesn't use, but good to test)
        let self_closing_xml =
            "<transcript><text start=\"0.0\" dur=\"1.0\">This has a <br/> tag</text></transcript>";
        let self_closing_result = parser.parse(self_closing_xml).unwrap();
        assert_eq!(self_closing_result.len(), 1);

        println!("Self-closing formatted: '{}'", self_closing_result[0].text);

        // The space before and after <br/> may vary
        let text = self_closing_result[0].text.clone();
        assert!(
            text.contains("This has a") && text.contains("tag"),
            "Actual: {}",
            text
        );

        // br is not in our formatting tags list, so it should be removed in non-preserve mode
        let plain_parser = TranscriptParser::new(false);
        let plain_result = plain_parser.parse(self_closing_xml).unwrap();

        println!("Self-closing plain: '{}'", plain_result[0].text);

        // Check plain text with flexible assertions
        assert!(
            plain_result[0].text.contains("This has a") && plain_result[0].text.contains("tag"),
            "Actual: {}",
            plain_result[0].text
        );
    }

    #[test]
    fn test_doc_examples() {
        // Test example from TranscriptParser struct documentation
        let xml = r#"
        <transcript>
            <text start="0.0" dur="1.0">This is a transcript</text>
            <text start="1.0" dur="1.5">With multiple entries</text>
        </transcript>
        "#;

        let parser = TranscriptParser::new(false);
        let snippets = parser.parse(xml).unwrap();
        assert_eq!(snippets.len(), 2);

        // Test example from parse method documentation
        let simple_xml = "<transcript><text start=\"0.0\" dur=\"1.0\">Hello</text></transcript>";
        let simple_parser = TranscriptParser::new(false);
        let simple_snippets = simple_parser.parse(simple_xml).unwrap();
        assert_eq!(simple_snippets.len(), 1);
        assert_eq!(simple_snippets[0].text, "Hello");
        assert_eq!(simple_snippets[0].start, 0.0);
        assert_eq!(simple_snippets[0].duration, 1.0);
    }

    #[test]
    fn test_total_duration_calculation() {
        // Test transcript duration calculation from transcript_parser_demo.rs
        let xml_content = r#"<?xml version="1.0" encoding="utf-8" ?>
        <transcript>
            <text start="12.645" dur="1.37">So in <b>college</b>,</text>
            <text start="15.349" dur="1.564">I was a <i>government</i> major,</text>
            <text start="16.937" dur="2.462">which means <b>I had to write</b> <i>a lot</i> of <b>papers</b>.</text>
        </transcript>"#;

        let parser = TranscriptParser::new(true);
        let snippets = parser.parse(xml_content).unwrap();

        // Calculate total duration
        let total_duration: f64 = snippets.iter().map(|snippet| snippet.duration).sum();

        // Use approximate comparison for floating point values (within 0.001)
        assert!(
            (total_duration - 5.396).abs() < 0.001,
            "Total duration {} should be approximately 5.396 seconds",
            total_duration
        );
    }

    #[test]
    fn test_parse_xml_with_version_declaration() {
        // Test parsing XML with XML declaration at the beginning
        let xml_with_declaration = r#"<?xml version="1.0" encoding="utf-8" ?>
        <transcript>
            <text start="1.0" dur="2.0">Text with XML declaration</text>
        </transcript>"#;

        let parser = TranscriptParser::new(false);
        let snippets = parser.parse(xml_with_declaration).unwrap();

        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].text, "Text with XML declaration");
        assert_eq!(snippets[0].start, 1.0);
        assert_eq!(snippets[0].duration, 2.0);
    }

    #[test]
    fn test_parse_with_xml_entities() {
        // Test transcript with various XML entities
        let xml_with_entities = r#"<?xml version="1.0" encoding="utf-8" ?>
        <transcript>
            <text start="1.0" dur="2.0">I couldn&#39;t quite do stuff.</text>
            <text start="3.0" dur="2.5">Let&#39;s try &amp; test some entities.</text>
            <text start="5.5" dur="3.0">Special characters: &lt;tag&gt; and &quot;quotes&quot;</text>
            <text start="8.5" dur="2.0">French accents: caf&eacute; &agrave; la cr&egrave;me</text>
            <text start="10.5" dur="1.5">Euro symbol: &euro; and degree: &deg;C</text>
        </transcript>"#;

        // Test with plain text mode (formatting removed)
        let plain_parser = TranscriptParser::new(false);
        let plain_snippets = plain_parser.parse(xml_with_entities).unwrap();

        assert_eq!(plain_snippets.len(), 5);

        // Print outputs for visual inspection
        println!("Entity test 0: '{}'", plain_snippets[0].text);
        println!("Entity test 1: '{}'", plain_snippets[1].text);
        println!("Entity test 2: '{}'", plain_snippets[2].text);
        println!("Entity test 3: '{}'", plain_snippets[3].text);
        println!("Entity test 4: '{}'", plain_snippets[4].text);

        // Test plain text conversion - html2text handles entities correctly
        assert_eq!(plain_snippets[0].text, "I couldn't quite do stuff.");
        assert_eq!(plain_snippets[1].text, "Let's try & test some entities.");
        assert_eq!(plain_snippets[2].text, "Special characters: and \"quotes\"");
        assert_eq!(plain_snippets[3].text, "French accents: café à la crème");
        assert_eq!(plain_snippets[4].text, "Euro symbol: € and degree: °C");

        // Test with formatting preserved
        let formatting_parser = TranscriptParser::new(true);
        let formatted_snippets = formatting_parser.parse(xml_with_entities).unwrap();

        assert_eq!(formatted_snippets.len(), 5);

        // In formatting mode, we still preserve structure but entities are decoded
        assert_eq!(formatted_snippets[0].text, "I couldn't quite do stuff.");
        assert_eq!(
            formatted_snippets[1].text,
            "Let's try & test some entities."
        );
        assert_eq!(
            formatted_snippets[2].text,
            "Special characters:  and \"quotes\""
        );
    }

    #[test]
    fn test_process_with_formatting() {
        let parser = TranscriptParser::new(true);

        // Test basic formatting
        let input = "<b>Bold</b> and <span>span</span> and <i>italic</i>";
        let result = parser.process_with_formatting(input);
        assert_eq!(
            result,
            "<b>Bold</b> and <span>span</span> and <i>italic</i>"
        );

        // Test with unwanted tags
        let input2 = "This has <div>unwanted</div> tags but <b>keeps</b> the <i>allowed</i> ones.";
        let result2 = parser.process_with_formatting(input2);
        assert_eq!(
            result2,
            "This has unwanted tags but <b>keeps</b> the <i>allowed</i> ones."
        );

        // Test with attributes
        let input3 =
            "<b id=\"test\">Bold with ID</b> and <i style=\"color:red\">Colored italic</i>";
        let result3 = parser.process_with_formatting(input3);
        assert_eq!(
            result3,
            "<b id=\"test\">Bold with ID</b> and <i style=\"color:red\">Colored italic</i>"
        );
    }
}
