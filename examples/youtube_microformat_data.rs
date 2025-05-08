use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to fetch microformat data from a YouTube video.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Fetching microformat data for a given video ID
/// 3. Displaying the microformat information including available countries, category, etc.
#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Microformat Data Example");
    println!("--------------------------------");

    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    println!("Fetching microformat data for: {}", video_id);

    match api.fetch_microformat(video_id).await {
        Ok(microformat) => {
            println!("\nMicroformat Data:");
            println!("-----------------");

            // Print video title
            if let Some(title) = &microformat.title {
                println!("Title: {}", title);
            }

            // Print channel information
            if let Some(channel_name) = &microformat.owner_channel_name {
                println!("Channel: {}", channel_name);

                if let Some(profile_url) = &microformat.owner_profile_url {
                    println!("Channel URL: {}", profile_url);
                }

                if let Some(channel_id) = &microformat.external_channel_id {
                    println!("Channel ID: {}", channel_id);
                }
            }

            // Print category
            if let Some(category) = &microformat.category {
                println!("Category: {}", category);
            }

            // Print video stats
            if let Some(views) = &microformat.view_count {
                println!("View Count: {}", views);
            }

            if let Some(likes) = &microformat.like_count {
                println!("Like Count: {}", likes);
            }

            if let Some(length) = &microformat.length_seconds {
                println!("Length: {} seconds", length);
            }

            // Print video status flags
            if let Some(is_unlisted) = microformat.is_unlisted {
                println!("Is Unlisted: {}", is_unlisted);
            }

            if let Some(is_family_safe) = microformat.is_family_safe {
                println!("Is Family Safe: {}", is_family_safe);
            }

            if let Some(is_shorts) = microformat.is_shorts_eligible {
                println!("Is Shorts Eligible: {}", is_shorts);
            }

            // Print upload and publish dates
            if let Some(upload_date) = &microformat.upload_date {
                println!("Upload Date: {}", upload_date);
            }

            if let Some(publish_date) = &microformat.publish_date {
                println!("Publish Date: {}", publish_date);
            }

            // Print available countries information
            if let Some(countries) = &microformat.available_countries {
                println!("\nAvailable in {} countries, including:", countries.len());
                // Display first 10 countries
                for country in countries.iter().take(10) {
                    print!("{} ", country);
                }
                println!("...");
            }

            // Print embed information
            if let Some(embed) = &microformat.embed {
                println!("\nEmbed Information:");
                if let Some(iframe_url) = &embed.iframe_url {
                    println!("iFrame URL: {}", iframe_url);
                }

                if let Some(width) = embed.width {
                    if let Some(height) = embed.height {
                        println!("Dimensions: {}x{}", width, height);
                    }
                }
            }

            // Print thumbnail information
            if let Some(thumbnail) = &microformat.thumbnail {
                if let Some(thumbnails) = &thumbnail.thumbnails {
                    println!("\nThumbnails: {} available", thumbnails.len());
                    for (i, thumb) in thumbnails.iter().enumerate() {
                        println!(
                            "  {}: {}x{} - {}",
                            i + 1,
                            thumb.width,
                            thumb.height,
                            thumb.url
                        );
                    }
                }
            }

            // Print a truncated description
            if let Some(description) = &microformat.description {
                println!("\nDescription:");
                let description_text = if description.len() > 300 {
                    format!("{}...", &description[..300])
                } else {
                    description.clone()
                };
                println!("{}", description_text);
            }
        }
        Err(e) => {
            println!("Failed to fetch microformat data: {:?}", e);
        }
    }

    Ok(())
}
