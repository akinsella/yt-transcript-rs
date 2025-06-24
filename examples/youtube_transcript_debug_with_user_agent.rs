use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// Enhanced debug example to test different user agents and headers
///
/// This example tests if YouTube is blocking requests based on user agent
/// or missing headers that a real browser would send.
#[tokio::main]
async fn main() -> Result<()> {
    let video_id = "XQJhRDbsDzI";

    println!("=== ENHANCED DEBUG: Testing different request configurations ===");
    println!("Video ID: {}", video_id);

    // Test 1: Default configuration (current failing one)
    println!("\n1. Testing with default configuration...");
    test_configuration("Default", None, video_id).await;

    // Test 2: Enhanced browser-like headers
    println!("\n2. Testing with enhanced browser headers...");
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.youtube.com/"),
    );
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"macOS\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    test_configuration("Enhanced Browser", Some(client), video_id).await;

    // Test 3: Firefox user agent
    println!("\n3. Testing with Firefox user agent...");
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:132.0) Gecko/20100101 Firefox/132.0",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.youtube.com/"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    test_configuration("Firefox", Some(client), video_id).await;

    // Test 4: Test direct URL access with different approaches
    println!("\n4. Testing direct transcript URL access...");
    test_direct_url_access(video_id).await;

    Ok(())
}

async fn test_configuration(name: &str, client: Option<reqwest::Client>, video_id: &str) {
    println!("Testing configuration: {}", name);

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

    match api.list_transcripts(video_id).await {
        Ok(transcript_list) => {
            println!(
                "  ✓ Found {} total transcripts",
                transcript_list.transcripts().count()
            );

            if let Ok(transcript) = transcript_list.find_transcript(&["en"]) {
                match api.fetch_transcript(video_id, &["en"], false).await {
                    Ok(fetched) => {
                        println!("  ✓ Fetch successful: {} snippets", fetched.snippets.len());
                        if !fetched.snippets.is_empty() {
                            println!("  ✓ SUCCESS! Found working configuration!");
                            println!("    First snippet: {:?}", fetched.snippets.first());
                        }
                    }
                    Err(e) => {
                        println!("  ✗ Fetch failed: {:?}", e);
                    }
                }
            } else {
                println!("  ✗ No English transcript found");
            }
        }
        Err(e) => {
            println!("  ✗ Failed to list transcripts: {:?}", e);
        }
    }
}

async fn test_direct_url_access(video_id: &str) {
    println!("Testing direct URL access with various methods...");

    // First get the transcript URL
    let api = YouTubeTranscriptApi::new(None, None, None).unwrap();
    let transcript_list = match api.list_transcripts(video_id).await {
        Ok(list) => list,
        Err(e) => {
            println!("  ✗ Failed to get transcript list: {}", e);
            return;
        }
    };

    let transcript = match transcript_list.find_transcript(&["en"]) {
        Ok(t) => t,
        Err(e) => {
            println!("  ✗ Failed to find transcript: {}", e);
            return;
        }
    };

    let url = &transcript.url;
    println!("  Testing URL: {}", url);

    // Test different client configurations
    let test_configs = vec![
        ("Basic client", reqwest::Client::new()),
        ("Browser-like client", {
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36"));
            headers.insert(
                REFERER,
                HeaderValue::from_static("https://www.youtube.com/"),
            );
            headers.insert(
                ACCEPT,
                HeaderValue::from_static("application/xml,text/xml,*/*"),
            );
            reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap()
        }),
    ];

    for (config_name, client) in test_configs {
        println!("    Testing {}: ", config_name);
        match client.get(url).send().await {
            Ok(response) => {
                println!("      Status: {}", response.status());
                println!(
                    "      Content-Type: {:?}",
                    response.headers().get("content-type")
                );
                println!(
                    "      Content-Length: {:?}",
                    response.headers().get("content-length")
                );

                match response.text().await {
                    Ok(text) => {
                        println!("      Response length: {} bytes", text.len());
                        if !text.is_empty() {
                            println!("      First 200 chars: {}", &text[..text.len().min(200)]);
                            if text.trim_start().starts_with('<') {
                                println!("      ✓ Looks like XML!");
                            } else {
                                println!("      ✗ Does not look like XML");
                            }
                        } else {
                            println!("      ✗ Empty response");
                        }
                    }
                    Err(e) => {
                        println!("      ✗ Failed to read response: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("      ✗ Request failed: {}", e);
            }
        }
    }
}
