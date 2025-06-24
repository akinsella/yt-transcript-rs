use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// Debug example to investigate transcript fetching issues
///
/// This example provides detailed debugging information to help identify
/// why transcripts might be returning 0 snippets.
#[tokio::main]
async fn main() -> Result<()> {
    let api = YouTubeTranscriptApi::new(None, None, None)?;
    let video_id = "XQJhRDbsDzI";

    println!("=== DEBUG: Investigating transcript issue ===");
    println!("Video ID: {}", video_id);

    // Step 1: List available transcripts
    println!("\n1. Listing available transcripts...");
    match api.list_transcripts(video_id).await {
        Ok(transcript_list) => {
            println!(
                "Found {} manually created transcripts",
                transcript_list.manually_created_transcripts.len()
            );
            println!(
                "Found {} auto-generated transcripts",
                transcript_list.generated_transcripts.len()
            );

            // Print all available transcripts
            for transcript in transcript_list.transcripts() {
                println!(
                    "  - {} ({}) [{}] URL: {}",
                    transcript.language,
                    transcript.language_code,
                    if transcript.is_generated {
                        "auto"
                    } else {
                        "manual"
                    },
                    transcript.url
                );
            }

            // Step 2: Try to fetch the transcript manually
            println!("\n2. Attempting to fetch transcript...");
            if let Ok(transcript) = transcript_list.find_transcript(&["en"]) {
                println!(
                    "Found transcript: {} ({})",
                    transcript.language, transcript.language_code
                );
                println!("Transcript URL: {}", transcript.url);

                // Step 3: Fetch the raw XML content
                println!("\n3. Fetching raw XML content...");
                let client = reqwest::Client::new();
                match client.get(&transcript.url).send().await {
                    Ok(response) => {
                        println!("HTTP Status: {}", response.status());
                        println!("Response Headers:");
                        for (key, value) in response.headers() {
                            println!("  {}: {:?}", key, value);
                        }

                        match response.text().await {
                            Ok(xml_content) => {
                                println!("XML Content Length: {} bytes", xml_content.len());

                                if xml_content.is_empty() {
                                    println!("ERROR: XML content is empty!");
                                } else {
                                    println!("First 500 characters of XML:");
                                    println!("{}", &xml_content[..xml_content.len().min(500)]);

                                    // Check if it looks like valid XML
                                    if xml_content.trim_start().starts_with('<') {
                                        println!("✓ Content appears to be XML");
                                    } else {
                                        println!("✗ Content does not appear to be XML");
                                    }

                                    // Count potential transcript entries
                                    let text_tag_count = xml_content.matches("<text").count();
                                    println!("Found {} <text> tags in XML", text_tag_count);
                                }

                                // Step 4: Try parsing with debug info
                                println!("\n4. Attempting to parse XML...");
                                match transcript.fetch(&client, false).await {
                                    Ok(fetched) => {
                                        println!(
                                            "✓ Parsing successful! Snippets: {}",
                                            fetched.snippets.len()
                                        );
                                        if fetched.snippets.is_empty() {
                                            println!(
                                                "WARNING: No snippets found in parsed transcript!"
                                            );
                                            println!("This suggests the XML format may have changed or the content is empty.");
                                        } else {
                                            println!(
                                                "✓ First snippet: {:?}",
                                                fetched.snippets.first()
                                            );
                                            println!(
                                                "✓ Last snippet: {:?}",
                                                fetched.snippets.last()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        println!("✗ Parsing failed: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("✗ Failed to read response text: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ HTTP request failed: {}", e);
                    }
                }
            } else {
                println!("✗ No English transcript found!");
            }
        }
        Err(e) => {
            println!("✗ Failed to list transcripts: {:?}", e);
        }
    }

    // Step 5: Test with multiple video IDs
    println!("\n5. Testing with multiple video IDs...");
    let test_videos = vec![
        ("dQw4w9WgXcQ", "Rick Roll"),
        ("9bZkp7q19f0", "Gangnam Style"),
        ("XQJhRDbsDzI", "Original failing video"),
        ("arj7oStGLkU", "Another test video"),
    ];

    for (video_id, description) in test_videos {
        println!("\nTesting {}: {}", description, video_id);
        match api.fetch_transcript(video_id, &["en"], false).await {
            Ok(transcript) => {
                println!("  ✓ Success: {} snippets", transcript.snippets.len());
            }
            Err(e) => {
                println!("  ✗ Failed: {:?}", e);
            }
        }
    }

    Ok(())
}
