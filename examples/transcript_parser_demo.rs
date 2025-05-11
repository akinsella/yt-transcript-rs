use anyhow::Result;
use yt_transcript_rs::transcript_parser::TranscriptParser;

fn main() -> Result<()> {
    // Sample XML transcript data with more HTML formatting tags
    let xml_content = r#"<?xml version="1.0" encoding="utf-8" ?>
<transcript>
    <text start="12.645" dur="1.37">So in <b>college</b>,</text>
    <text start="15.349" dur="1.564">I was a <i>government</i> major,</text>
    <text start="16.937" dur="2.462">which means <b>I had to write</b> <i>a lot</i> of <b>papers</b>.</text>
</transcript>"#;

    // Create a parser that preserves formatting tags
    let parser_with_formatting = TranscriptParser::new(true);
    let formatted_snippets = parser_with_formatting.parse(xml_content)?;

    // Create a parser that strips all formatting
    let plain_parser = TranscriptParser::new(false);
    let plain_snippets = plain_parser.parse(xml_content)?;

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

    // Calculate total duration
    let total_duration: f64 = formatted_snippets
        .iter()
        .map(|snippet| snippet.duration)
        .sum();

    println!("\nTotal transcript duration: {:.2} seconds", total_duration);

    Ok(())
}
