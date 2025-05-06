use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to translate a YouTube transcript to a different language.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Listing available transcripts
/// 3. Finding a translatable transcript
/// 4. Translating the transcript to a different language
/// 5. Displaying the translated transcript
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID (known to have multiple language transcripts)
    let video_id = "arj7oStGLkU";

    // Target translation language
    let target_language = "fr"; // French

    println!(
        "Getting transcript for video ID: {} and translating to {}",
        video_id, target_language
    );

    // Step 1: List available transcripts
    match api.list_transcripts(video_id).await {
        Ok(transcript_list) => {
            println!("Successfully retrieved transcript list!");

            // Step 2: Find a translatable transcript
            let mut found_translatable = false;

            for transcript in &transcript_list {
                if transcript.is_translatable() {
                    found_translatable = true;
                    println!(
                        "Found translatable transcript: {} ({})",
                        transcript.language, transcript.language_code
                    );

                    // Check if the target language is available
                    let target_available = transcript
                        .translation_languages
                        .iter()
                        .any(|lang| lang.language_code == target_language);

                    if target_available {
                        println!(
                            "Target language {} is available for translation",
                            target_language
                        );

                        // Step 3: Translate the transcript
                        match transcript.translate(target_language) {
                            Ok(translated_transcript) => {
                                println!("Successfully created translated transcript object");

                                // Step 4: Fetch the translated transcript content
                                match translated_transcript.fetch(false).await {
                                    Ok(fetched_transcript) => {
                                        println!("Successfully fetched translated transcript!");
                                        println!(
                                            "Original language: {} ({})",
                                            transcript.language, transcript.language_code
                                        );
                                        println!(
                                            "Translated language: {} ({})",
                                            fetched_transcript.language,
                                            fetched_transcript.language_code
                                        );
                                        println!(
                                            "Number of snippets: {}",
                                            fetched_transcript.snippets.len()
                                        );

                                        // Display the first 5 snippets of the translated transcript
                                        println!("\nTranslated transcript content:");
                                        for (i, snippet) in
                                            fetched_transcript.snippets.iter().take(5).enumerate()
                                        {
                                            println!(
                                                "[{:.1}-{:.1}s] {}",
                                                snippet.start,
                                                snippet.start + snippet.duration,
                                                snippet.text
                                            );
                                        }

                                        println!("... (truncated)");
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        println!("Failed to fetch translated transcript: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to translate transcript: {:?}", e);
                            }
                        }
                    } else {
                        println!(
                            "Target language {} is not available for translation",
                            target_language
                        );
                        println!("Available translation languages:");
                        for lang in &transcript.translation_languages {
                            println!("  {} ({})", lang.language, lang.language_code);
                        }
                    }

                    // Only process the first translatable transcript
                    break;
                }
            }

            if !found_translatable {
                println!("No translatable transcripts found for this video");
            }
        }
        Err(e) => {
            println!("Failed to list transcripts: {:?}", e);
        }
    }

    Ok(())
}
