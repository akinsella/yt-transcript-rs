use anyhow::Result;
use url::Url;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// Debug example to show exactly what URLs are being tried
#[tokio::main]
async fn main() -> Result<()> {
    let api = YouTubeTranscriptApi::new(None, None, None)?;
    let video_id = "XQJhRDbsDzI";

    println!("=== URL DEBUG: Investigating URL modifications ===");
    println!("Video ID: {}", video_id);

    // Get transcript list
    let transcript_list = api.list_transcripts(video_id).await?;
    let transcript = transcript_list.find_transcript(&["en"])?;

    println!("\nOriginal URL from YouTube:");
    println!("{}", transcript.url);

    // Test the URL cleaning function manually
    let cleaned_url = clean_transcript_url(&transcript.url);
    println!("\nCleaned URL (IP params removed):");
    println!("{}", cleaned_url);

    // Test both URLs directly
    let client = reqwest::Client::new();

    println!("\n=== Testing Original URL ===");
    test_url(&client, &transcript.url).await;

    println!("\n=== Testing Cleaned URL ===");
    test_url(&client, &cleaned_url).await;

    // Try a minimal URL with just essential params
    let minimal_url = create_minimal_url(&transcript.url);
    println!("\n=== Testing Minimal URL ===");
    println!("Minimal URL: {}", minimal_url);
    test_url(&client, &minimal_url).await;

    Ok(())
}

async fn test_url(client: &reqwest::Client, url: &str) {
    match client.get(url).send().await {
        Ok(response) => {
            println!("Status: {}", response.status());
            println!("Headers:");
            for (key, value) in response.headers() {
                println!("  {}: {:?}", key, value);
            }

            match response.text().await {
                Ok(text) => {
                    println!("Response length: {} bytes", text.len());
                    if !text.is_empty() {
                        println!("First 300 chars: {}", &text[..text.len().min(300)]);
                        if text.trim_start().starts_with('<') {
                            println!("✓ Looks like XML!");
                        } else {
                            println!("✗ Does not look like XML");
                        }
                    } else {
                        println!("✗ Empty response");
                    }
                }
                Err(e) => {
                    println!("✗ Failed to read response: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Request failed: {}", e);
        }
    }
}

// Copy of the clean function from transcript.rs for testing
fn clean_transcript_url(url: &str) -> String {
    if let Ok(parsed_url) = Url::parse(url) {
        let mut cleaned_url = format!(
            "{}://{}{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap_or("www.youtube.com"),
            parsed_url.path()
        );

        let mut params = Vec::new();
        for (key, value) in parsed_url.query_pairs() {
            match key.as_ref() {
                "ip" | "ipbits" => continue,
                _ => {
                    params.push(format!("{}={}", key, value));
                }
            }
        }

        if !params.is_empty() {
            cleaned_url.push('?');
            cleaned_url.push_str(&params.join("&"));
        }

        cleaned_url
    } else {
        url.to_string()
    }
}

// Create a minimal URL with only essential parameters
fn create_minimal_url(url: &str) -> String {
    if let Ok(parsed_url) = Url::parse(url) {
        let mut minimal_url = format!(
            "{}://{}{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap_or("www.youtube.com"),
            parsed_url.path()
        );

        let mut params = Vec::new();
        for (key, value) in parsed_url.query_pairs() {
            match key.as_ref() {
                // Only keep absolutely essential parameters
                "v" | "lang" | "tlang" | "kind" => {
                    params.push(format!("{}={}", key, value));
                }
                _ => continue,
            }
        }

        if !params.is_empty() {
            minimal_url.push('?');
            minimal_url.push_str(&params.join("&"));
        }

        minimal_url
    } else {
        url.to_string()
    }
}
