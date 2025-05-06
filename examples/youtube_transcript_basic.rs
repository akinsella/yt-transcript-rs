use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to fetch a transcript from a YouTube video.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Fetching a transcript for a video in a specific language
/// 3. Displaying the transcript content
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the YouTubeTranscriptApi
    // This creates a new instance without proxy or cookie authentication
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "nQSDgS8BWx8";

    // Language preference (English)
    let languages = &["en"];

    // Don't preserve formatting (remove line breaks, etc.)
    let preserve_formatting = false;

    // Fetch the transcript
    println!("Fetching transcript for video ID: {}", video_id);

    match api.fetch_transcript(video_id, languages, preserve_formatting).await {
        Ok(transcript) => {
            println!("Successfully fetched transcript!");
            println!("Video ID: {}", transcript.video_id);
            println!(
                "Language: {} ({})",
                transcript.language, transcript.language_code
            );
            println!("Is auto-generated: {}", transcript.is_generated);
            println!("Number of snippets: {}", transcript.snippets.len());
            println!("\nTranscript content:");

            // Display the first 5 snippets
            for (_i, snippet) in transcript.snippets.iter().take(5).enumerate() {
                println!(
                    "[{:.1}-{:.1}s] {}",
                    snippet.start,
                    snippet.start + snippet.duration,
                    snippet.text
                );
            }

            println!("... (truncated)");
        }
        Err(e) => {
            println!("Failed to fetch transcript: {:?}", e);
        }
    }

    Ok(())
}
