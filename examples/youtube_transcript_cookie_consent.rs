use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use std::path::Path;

/// This example demonstrates how to handle cookie consent requirements when using
/// the YouTube Transcript API.
///
/// When accessing YouTube from regions with strict data protection laws (like the EU),
/// YouTube may present a cookie consent page. This API automatically handles this
/// by setting the necessary consent cookie.
///
/// This example will:
/// 1. Create a YouTubeTranscriptApi instance
/// 2. Fetch a transcript for a video (the API will handle consent if needed)
/// 3. Display information about the transcript
#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Transcript API - Cookie Consent Example");
    println!("------------------------------------------------");

    // Initialize the YouTubeTranscriptApi
    // The API will automatically handle consent cookies if needed
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    // Language preference (English)
    let languages = &["en"];

    // Don't preserve formatting
    let preserve_formatting = false;

    println!("Fetching transcript for video ID: {}", video_id);
    println!("(If you're in the EU or other regions with cookie consent laws,");
    println!(" the API will automatically handle the consent requirements)");

    match api.fetch_transcript(video_id, languages, preserve_formatting).await {
        Ok(transcript) => {
            println!("\nSuccessfully fetched transcript!");
            println!("Video ID: {}", transcript.video_id);
            println!(
                "Language: {} ({})",
                transcript.language, transcript.language_code
            );
            println!("Is auto-generated: {}", transcript.is_generated);
            println!("Number of snippets: {}", transcript.snippets.len());
            println!("\nTranscript content (first 5 snippets):");

            for (i, snippet) in transcript.snippets.iter().take(5).enumerate() {
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

            // Specific error handling for consent-related errors
            if format!("{:?}", e).contains("FailedToCreateConsentCookie") {
                println!("\nThis error occurs when the API cannot automatically handle");
                println!("the cookie consent requirements. This might happen if YouTube");
                println!("changes its consent form structure.");

                println!("\nPossible solutions:");
                println!("1. Try using a proxy from a region without consent requirements");
                println!("2. Try using a cookie file from a browser where you've already");
                println!("   accepted the consent (see youtube_transcript_cookie_auth.rs example)");
            }
        }
    }

    Ok(())
}
