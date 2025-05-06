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
