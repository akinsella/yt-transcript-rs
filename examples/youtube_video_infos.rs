use anyhow::Result;
use reqwest::Client;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to fetch all information about a YouTube video in a single request.
/// It also shows how to serialize and deserialize the VideoInfos struct for storage or transmission.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Fetching all video information at once using fetch_video_infos
/// 3. Serializing the data to JSON and deserializing it back
/// 4. Displaying comprehensive information from different data categories:
///    - Video details (title, author, etc.)
///    - Microformat data (category, countries, etc.)
///    - Streaming data (available formats)
///    - Transcript list (available captions)
#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Video Infos (All-in-One) Example");
    println!("----------------------------------------");

    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    println!("Fetching all video information for: {}", video_id);
    println!("(This makes only a single request to YouTube)");

    match api.fetch_video_infos(video_id).await {
        Ok(infos) => {
            // ----- BASIC VIDEO DETAILS -----
            println!("\n📝 Video Details:");
            println!("----------------");
            println!("Title: {}", infos.video_details.title);
            println!("Author: {}", infos.video_details.author);
            println!("View Count: {}", infos.video_details.view_count);
            println!("Length: {} seconds", infos.video_details.length_seconds);

            // Print keywords if available
            if let Some(keywords) = &infos.video_details.keywords {
                if !keywords.is_empty() {
                    println!(
                        "Keywords: {}",
                        keywords
                            .iter()
                            .take(5)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    if keywords.len() > 5 {
                        println!("  ... and {} more", keywords.len() - 5);
                    }
                }
            }

            // ----- MICROFORMAT DATA -----
            println!("\n🌐 Microformat Data:");
            println!("-----------------");

            // Print category
            if let Some(category) = &infos.microformat.category {
                println!("Category: {}", category);
            }

            // Print upload and publish dates
            if let Some(upload_date) = &infos.microformat.upload_date {
                println!("Upload Date: {}", upload_date);
            }

            // Print family safe status
            if let Some(is_family_safe) = infos.microformat.is_family_safe {
                println!("Family Safe: {}", is_family_safe);
            }

            // Print available countries
            if let Some(countries) = &infos.microformat.available_countries {
                println!("Available in {} countries", countries.len());
            }

            // ----- STREAMING DATA -----
            println!("\n🎬 Streaming Data:");
            println!("-----------------");
            println!(
                "Expires in: {} seconds",
                infos.streaming_data.expires_in_seconds
            );
            println!("Combined formats: {}", infos.streaming_data.formats.len());
            println!(
                "Adaptive formats: {}",
                infos.streaming_data.adaptive_formats.len()
            );

            // Count video and audio formats
            let video_formats = infos
                .streaming_data
                .adaptive_formats
                .iter()
                .filter(|f| f.mime_type.starts_with("video/"))
                .count();

            let audio_formats = infos
                .streaming_data
                .adaptive_formats
                .iter()
                .filter(|f| f.mime_type.starts_with("audio/"))
                .count();

            println!("Video-only formats: {}", video_formats);
            println!("Audio-only formats: {}", audio_formats);

            // Find highest resolution
            let highest_resolution = infos
                .streaming_data
                .adaptive_formats
                .iter()
                .filter_map(|f| f.height)
                .max()
                .unwrap_or(0);

            println!("Highest resolution: {}p", highest_resolution);

            // ----- TRANSCRIPT DATA -----
            println!("\n📄 Transcript Data:");
            println!("-----------------");

            let transcript_list = &infos.transcript_list;
            let transcript_count = transcript_list.transcripts().count();
            println!("Available transcripts: {}", transcript_count);

            // Print information about each transcript
            for (i, transcript) in transcript_list.transcripts().enumerate().take(5) {
                let translatable = if transcript.is_translatable() {
                    "✓"
                } else {
                    "✗"
                };
                println!(
                    "  {}: {} ({}) [Translatable: {}]",
                    i + 1,
                    transcript.language(),
                    transcript.language_code(),
                    translatable
                );

                // Only show translation languages for the first translatable transcript
                if transcript.is_translatable() && i == 0 {
                    let translation_langs = &transcript.translation_languages;
                    println!(
                        "    Available translations: {} languages",
                        translation_langs.len()
                    );
                    for (j, lang) in translation_langs.iter().take(3).enumerate() {
                        println!(
                            "      {}: {} ({})",
                            j + 1,
                            lang.language,
                            lang.language_code
                        );
                    }

                    if translation_langs.len() > 3 {
                        println!("      ... and {} more", translation_langs.len() - 3);
                    }
                }

                // Limit to showing first 5 transcripts
                if i >= 4 && transcript_count > 5 {
                    println!("  ... and {} more", transcript_count - 5);
                    break;
                }
            }

            println!("\n✨ All data retrieved in a single request!");

            // ----- SERIALIZATION EXAMPLE -----
            println!("\n💾 Serialization and Deserialization:");
            println!("--------------------------------");

            // Serialize to JSON
            println!("Serializing VideoInfos to JSON...");
            let serialized = serde_json::to_string(&infos).expect("Failed to serialize");
            println!("Serialized data size: {} bytes", serialized.len());

            // Deserialize back
            println!("Deserializing back to VideoInfos...");
            let deserialized =
                serde_json::from_str::<yt_transcript_rs::models::VideoInfos>(&serialized)
                    .expect("Failed to deserialize");

            // Verify the data is intact
            println!("Verifying data integrity:");
            println!(
                "  - Title match: {}",
                infos.video_details.title == deserialized.video_details.title
            );
            println!(
                "  - Format count match: {}",
                infos.streaming_data.formats.len() == deserialized.streaming_data.formats.len()
            );
            println!(
                "  - Transcript count match: {}",
                transcript_count == deserialized.transcript_list.transcripts().count()
            );

            // Demonstrate fetching a transcript from the deserialized data
            println!("\nFetching a transcript from the deserialized data...");
            if transcript_count > 0 {
                let transcript = deserialized.transcript_list.find_transcript(&["en", "es"]);
                if let Ok(transcript) = transcript {
                    println!("Found {} transcript", transcript.language());

                    // Create a client to fetch the transcript
                    let client = Client::new();
                    let fetched = transcript.fetch(&client, false).await;

                    if let Ok(fetched) = fetched {
                        let snippet_count = fetched.parts().len();
                        println!(
                            "Successfully fetched transcript with {} text segments",
                            snippet_count
                        );

                        // Show a small sample of the transcript
                        println!("Sample transcript content:");
                        for (i, part) in fetched.parts().iter().take(3).enumerate() {
                            println!(
                                "  [{}] {:.1}s: {}",
                                i + 1,
                                part.start,
                                &part.text[..part.text.len().min(50)]
                            );
                        }
                    } else {
                        println!("Failed to fetch transcript: {:?}", fetched.err());
                    }
                } else {
                    println!("No English or Spanish transcript found");
                }
            }

            println!("\nTry this example with different video IDs to see more information.");
        }
        Err(e) => {
            println!("Failed to fetch video information: {:?}", e);
        }
    }

    Ok(())
}
