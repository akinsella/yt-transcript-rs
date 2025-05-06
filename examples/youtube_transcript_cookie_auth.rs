use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use std::env;
use std::path::Path;

/// This example demonstrates how to use cookie authentication with the YouTube Transcript API.
///
/// Cookie authentication is useful for:
/// 1. Accessing age-restricted videos without providing login credentials
/// 2. Bypassing region restrictions
/// 3. Working with videos that require a logged-in user
/// 4. Having consent cookies already set (for EU regions)
///
/// This example will:
/// 1. Check for a cookie file path from an environment variable
/// 2. Create a YouTubeTranscriptApi instance with cookie authentication
/// 3. Fetch a transcript for a video that might be age-restricted
/// 4. Display information about the transcript
///
/// To use this example, set the YOUTUBE_COOKIE_FILE environment variable:
/// export YOUTUBE_COOKIE_FILE=/path/to/your/cookies.txt
///
/// You can export cookies from your browser using extensions like:
/// - "Get cookies.txt" (Chrome/Firefox)
/// - "Cookie-Editor" (Chrome/Firefox)
/// - "EditThisCookie" (Chrome)
#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Transcript API - Cookie Authentication Example");
    println!("-----------------------------------------------------");

    // Get cookie file path from environment variable
    let cookie_file = env::var("YOUTUBE_COOKIE_FILE").ok();

    let api = if let Some(cookie_path) = &cookie_file {
        println!("Using cookie file: {}", cookie_path);

        // Initialize the YouTubeTranscriptApi with cookie authentication
        let path = Path::new(cookie_path);
        YouTubeTranscriptApi::new(Some(path), None, None)?
    } else {
        println!("No cookie file specified (YOUTUBE_COOKIE_FILE not set)");
        println!("Using without cookie authentication");

        // Initialize without cookie auth
        YouTubeTranscriptApi::new(None, None, None)?
    };

    // Try an age-restricted video ID first (if you have cookie auth)
    // If no cookie auth, try a regular video
    let video_id = if cookie_file.is_some() {
        // A potentially age-restricted video
        "kJQP7kiw5Fk" // "Despacito" by Luis Fonsi - often age-restricted
    } else {
        // A regular video without restrictions
        "arj7oStGLkU" // TED Talk
    };

    // Language preference (English)
    let languages = &["en"];

    // Don't preserve formatting
    let preserve_formatting = false;

    println!("Fetching transcript for video ID: {}", video_id);
    println!("(The API will handle cookie consent if needed)");

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

            for snippet in transcript.snippets.iter().take(5) {
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

            if format!("{:?}", e).contains("AgeRestricted") {
                println!("\nThis video is age-restricted. To access it, you need to:");
                println!("1. Export cookies from a browser where you're logged into YouTube");
                println!("2. Set the YOUTUBE_COOKIE_FILE environment variable to point to your cookie file");
                println!("3. Run this example again");
            } else if format!("{:?}", e).contains("FailedToCreateConsentCookie") {
                println!("\nCouldn't create a consent cookie automatically.");
                println!("To resolve this:");
                println!("1. Export cookies from a browser where you've already accepted YouTube's consent");
                println!("2. Set the YOUTUBE_COOKIE_FILE environment variable to point to your cookie file");
            }
        }
    }

    Ok(())
}
