`yt-transcript-rs` is heavily inspired from the python module [youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) originally developed by [Jonas Depoix](https://github.com/jdepoix).

## Install

```bash
cargo add yt-transcript-rs
```


## API

### Fetch a transcript

```rust
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
    let video_id = "5MuIMqhT8DM";

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
```


### List available transcripts

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to list all available transcripts for a YouTube video.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Listing all available transcripts
/// 3. Displaying information about each transcript, including whether it's translatable
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID (known to have multiple language transcripts)
    let video_id = "arj7oStGLkU";

    // List available transcripts
    println!("Listing available transcripts for video ID: {}", video_id);

    match api.list_transcripts(video_id).await {
        Ok(transcript_list) => {
            println!("Successfully retrieved transcript list!");
            println!("Video ID: {}", transcript_list.video_id);

            // Count available transcripts
            let mut count = 0;
            let mut translatable_count = 0;

            println!("\nAvailable transcripts:");
            for transcript in &transcript_list {
                count += 1;
                let translatable = if transcript.is_translatable() {
                    translatable_count += 1;
                    "[translatable]"
                } else {
                    ""
                };

                println!(
                    "{}: {} ({}) {}",
                    count, transcript.language, transcript.language_code, translatable
                );

                // If this transcript is translatable, show available translation languages
                if transcript.is_translatable() && count == 1 {
                    // Just show for the first one
                    println!("  Available translations:");
                    for (i, lang) in transcript.translation_languages.iter().take(5).enumerate() {
                        println!("    {}: {} ({})", i + 1, lang.language, lang.language_code);
                    }

                    if transcript.translation_languages.len() > 5 {
                        println!(
                            "    ... and {} more",
                            transcript.translation_languages.len() - 5
                        );
                    }
                }
            }

            println!("\nTotal transcripts: {}", count);
            println!("Translatable transcripts: {}", translatable_count);
        }
        Err(e) => {
            println!("Failed to list transcripts: {:?}", e);
        }
    }

    Ok(())
}
```

### Fetch video details

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to fetch video details using the YouTube Transcript API.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Fetching video details for a given video ID
/// 3. Displaying the video information including title, author, view count, etc.
#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Video Details Example");
    println!("------------------------------");

    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    println!("Fetching video details for: {}", video_id);

    match api.fetch_video_details(video_id).await {
        Ok(details) => {
            println!("\nVideo Details:");
            println!("-------------");
            println!("Video ID: {}", details.video_id);
            println!("Title: {}", details.title);
            println!("Author: {}", details.author);
            println!("Channel ID: {}", details.channel_id);
            println!("View Count: {}", details.view_count);
            println!("Length: {} seconds", details.length_seconds);
            println!("Is Live Content: {}", details.is_live_content);

            // Print keywords if available
            if let Some(keywords) = details.keywords {
                println!("\nKeywords:");
                for (i, keyword) in keywords.iter().enumerate().take(10) {
                    println!("  {}: {}", i + 1, keyword);
                }

                if keywords.len() > 10 {
                    println!("  ... and {} more", keywords.len() - 10);
                }
            }

            // Print thumbnail information
            println!("\nThumbnails: {} available", details.thumbnails.len());
            for (i, thumb) in details.thumbnails.iter().enumerate() {
                println!(
                    "  {}: {}x{} - {}",
                    i + 1,
                    thumb.width,
                    thumb.height,
                    thumb.url
                );
            }

            // Print a truncated description
            println!("\nDescription:");
            let description = if details.short_description.len() > 300 {
                format!("{}...", &details.short_description[..300])
            } else {
                details.short_description.clone()
            };
            println!("{}", description);
        }
        Err(e) => {
            println!("Failed to fetch video details: {:?}", e);
        }
    }

    Ok(())
}
```