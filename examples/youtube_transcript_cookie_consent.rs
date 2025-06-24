use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// Example demonstrating transcript fetching with consent cookies
///
/// YouTube may require consent cookies for transcript access in certain regions
/// or for certain types of content. This example shows how to handle this.
#[tokio::main]
async fn main() -> Result<()> {
    let video_id = "XQJhRDbsDzI";

    println!("=== YouTube Transcript with Consent Handling ===");
    println!("Video ID: {}", video_id);

    // Test 1: Default configuration
    println!("\n1. Testing default configuration...");
    test_transcript_fetch("Default", None, video_id).await;

    // Test 2: With consent cookies
    println!("\n2. Testing with consent cookies...");
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.youtube.com/"),
    );

    // Common consent cookies that YouTube might require
    let consent_cookies = vec![
        "CONSENT=YES+cb.20210328-17-p0.en+FX+111",
        "SOCS=CAESEwgDEgk0ODE3Nzk3MjQaAmVuIAEaBgiA_LyaBg", // Consent for data processing
    ];

    headers.insert(COOKIE, HeaderValue::from_str(&consent_cookies.join("; "))?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    test_transcript_fetch("With Consent Cookies", Some(client), video_id).await;

    // Test 3: Alternative approach - try to fetch the video page first
    println!("\n3. Testing with video page prefetch...");
    test_with_page_prefetch(video_id).await;

    // Provide guidance to users
    println!("\n=== GUIDANCE FOR USERS ===");
    println!("If transcripts are returning empty (0 snippets), this indicates that");
    println!(
        "YouTube has changed their transcript API requirements. Here are potential solutions:"
    );
    println!();
    println!("1. **Use Cookie Authentication**:");
    println!("   - Export cookies from your browser using a browser extension");
    println!("   - Save them to a file (cookies.txt format)");
    println!("   - Use YouTubeTranscriptApi::new(Some(cookie_path), None, None)");
    println!();
    println!("2. **Try Different Videos**:");
    println!("   - Some videos may have transcripts disabled");
    println!("   - Try popular, public videos with known transcripts");
    println!();
    println!("3. **Check Video Accessibility**:");
    println!("   - Ensure the video is public and not age-restricted");
    println!("   - Verify transcripts are available on the YouTube website");
    println!();
    println!("4. **Use Proxies** (if blocked by region):");
    println!("   - Configure a proxy in a different region");
    println!("   - Some regions may have different access requirements");
    println!();
    println!("This is likely a temporary API change by YouTube. Check for library updates.");

    Ok(())
}

async fn test_transcript_fetch(name: &str, client: Option<reqwest::Client>, video_id: &str) {
    println!("Testing: {}", name);

    let api = match client {
        Some(c) => YouTubeTranscriptApi::new(None, None, Some(c)),
        None => YouTubeTranscriptApi::new(None, None, None),
    };

    let api = match api {
        Ok(api) => api,
        Err(e) => {
            println!("  ✗ Failed to create API: {}", e);
            return;
        }
    };

    match api.fetch_transcript(video_id, &["en"], false).await {
        Ok(transcript) => {
            println!("  ✓ Success: {} snippets", transcript.snippets.len());
            if !transcript.snippets.is_empty() {
                println!("  ✓ WORKING SOLUTION FOUND!");
                println!("    First snippet: {:?}", transcript.snippets.first());
            } else {
                println!("  ⚠ Empty transcript (0 snippets) - API change likely");
            }
        }
        Err(e) => {
            println!("  ✗ Failed: {:?}", e);
        }
    }
}

async fn test_with_page_prefetch(video_id: &str) {
    println!("Testing with video page prefetch...");

    // First, visit the video page to establish session
    let client = reqwest::Client::builder()
        .cookie_store(true) // Enable cookie storage
        .build()
        .unwrap();

    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);

    match client.get(&video_url)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36")
        .send()
        .await
    {
        Ok(response) => {
            println!("  Video page status: {}", response.status());

            // Now try to fetch transcript with the same client (which has cookies)
            let api = YouTubeTranscriptApi::new(None, None, Some(client)).unwrap();
            match api.fetch_transcript(video_id, &["en"], false).await {
                Ok(transcript) => {
                    println!("  ✓ Transcript fetch: {} snippets", transcript.snippets.len());
                    if !transcript.snippets.is_empty() {
                        println!("  ✓ SUCCESS! Page prefetch worked!");
                    }
                }
                Err(e) => {
                    println!("  ✗ Transcript fetch failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("  ✗ Failed to fetch video page: {}", e);
        }
    }
}
