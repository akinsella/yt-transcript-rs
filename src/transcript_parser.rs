use anyhow::Result;
use html_escape::decode_html_entities;
use regex::Regex;

use crate::models::FetchedTranscriptSnippet;

#[derive(Debug)]
/// Parser for YouTube transcript XML data
pub struct TranscriptParser {
    preserve_formatting: bool,
    html_regex: Regex,
}

impl TranscriptParser {
    /// Formatting tags preserved when preserve_formatting is true
    const FORMATTING_TAGS: [&'static str; 10] = [
        "strong", // important
        "em",     // emphasized
        "b",      // bold
        "i",      // italic
        "mark",   // marked
        "small",  // smaller
        "del",    // deleted
        "ins",    // inserted
        "sub",    // subscript
        "sup",    // superscript
    ];

    /// Create a new transcript parser
    pub fn new(preserve_formatting: bool) -> Self {
        let html_regex = if preserve_formatting {
            // Preserve specified formatting tags but remove all others
            let formats_regex = Self::FORMATTING_TAGS.join("|");
            let pattern = format!(r"<\/?(?!\/?({}))\b.*?\b>", formats_regex);
            Regex::new(&pattern).unwrap()
        } else {
            // Remove all HTML tags
            Regex::new(r"<[^>]*>").unwrap()
        };

        Self {
            preserve_formatting,
            html_regex,
        }
    }

    /// Parse transcript XML into snippets
    pub fn parse(&self, raw_data: &str) -> Result<Vec<FetchedTranscriptSnippet>, anyhow::Error> {
        let mut snippets = Vec::new();

        // Parse XML
        let document = roxmltree::Document::parse(raw_data)?;
        let transcript_elem = document.root_element();

        for text_elem in transcript_elem
            .children()
            .filter(|n| n.has_tag_name("text"))
        {
            let start = text_elem
                .attribute("start")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let duration = text_elem
                .attribute("dur")
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            // Get text and remove unwanted HTML tags
            let text = text_elem.text().unwrap_or("").to_string();
            let text = decode_html_entities(&text).to_string();

            // Apply regex to remove HTML tags (or preserve specific ones)
            let text = self.html_regex.replace_all(&text, "").to_string();

            snippets.push(FetchedTranscriptSnippet {
                text,
                start,
                duration,
            });
        }

        Ok(snippets)
    }
}
