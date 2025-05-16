use anyhow::Result;
use yt_transcript_rs::transcript_parser::TranscriptParser;

/// Example demonstrating the usage of the YouTube transcript parser
///
/// This example shows how to:
/// 1. Parse YouTube transcript XML data with HTML formatting tags
/// 2. Create parsers that either preserve or strip formatting
/// 3. Display transcript snippets with timing information
/// 4. Calculate total transcript duration
///
/// The example uses a sample XML transcript with various HTML formatting tags
/// (<b>, <i>) to demonstrate the parser's ability to handle formatted text.
fn main() -> Result<()> {
    // Sample XML transcript data with more HTML formatting tags
    // This simulates a real YouTube transcript with timing information and HTML formatting
    let xml_content = r#"<?xml version="1.0" encoding="utf-8" ?>
<transcript>
    <text start="12.645" dur="1.37">So in <b>college</b>,</text>
    <text start="15.349" dur="1.564">I was a <i>government</i> major,</text>
    <text start="16.937" dur="2.462">which means <b>I had to write</b> <i>a lot</i> of <b>papers</b>.</text>
</transcript>"#;

    // Initialize parser with formatting preservation enabled
    // This will keep HTML tags like <b> and <i> in the output
    let parser_with_formatting = TranscriptParser::new(true);
    let formatted_snippets = parser_with_formatting.parse(xml_content)?;

    // Initialize parser with formatting disabled
    // This will strip all HTML tags from the output
    let plain_parser = TranscriptParser::new(false);
    let plain_snippets = plain_parser.parse(xml_content)?;

    // Display formatted transcript snippets with their timing information
    println!("=== Transcript with formatting preserved ===");
    println!("(HTML tags should be visible in this output)");
    for (i, snippet) in formatted_snippets.iter().enumerate() {
        println!(
            "#{}: [{:.1}s-{:.1}s] {}",
            i + 1,
            snippet.start,
            snippet.start + snippet.duration,
            snippet.text
        );
    }

    // Display plain transcript snippets (no HTML tags)
    println!("\n=== Transcript with all formatting removed ===");
    println!("(No HTML tags should be visible in this output)");
    for (i, snippet) in plain_snippets.iter().enumerate() {
        println!(
            "#{}: [{:.1}s-{:.1}s] {}",
            i + 1,
            snippet.start,
            snippet.start + snippet.duration,
            snippet.text
        );
    }

    // Calculate and display the total duration of the transcript
    // by summing up the duration of each snippet
    let total_duration: f64 = formatted_snippets
        .iter()
        .map(|snippet| snippet.duration)
        .sum();

    println!("\nTotal transcript duration: {:.2} seconds", total_duration);

    Ok(())
}
