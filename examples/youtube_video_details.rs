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
