use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use yt_transcript_rs::proxies::{
    GenericProxyConfig, ProxyConfig,
};
use std::env;
use std::path::Path;

/// This advanced example demonstrates combining multiple features of the YouTube Transcript API:
///
/// 1. Cookie authentication for age-restricted videos
/// 2. Proxy configuration for bypassing IP blocks
/// 3. Consent cookie handling for EU regions
/// 4. Multiple transcript formats (JSON, SRT, plain text)
/// 5. Detailed error handling for all scenarios
///
/// You can customize this example by setting environment variables:
/// - YOUTUBE_COOKIE_FILE: Path to a cookies.txt file for auth
/// - HTTP_PROXY: HTTP proxy URL
/// - HTTPS_PROXY: HTTPS proxy URL
/// - VIDEO_ID: Specific video ID to use (optional)
#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Transcript API - Advanced Example");
    println!("----------------------------------------");

    // === 1. Set up configuration from environment variables ===

    // Cookie authentication
    let cookie_file = env::var("YOUTUBE_COOKIE_FILE").ok();
    if let Some(ref path) = cookie_file {
        println!("Using cookie file: {}", path);
    } else {
        println!("No cookie file specified (set YOUTUBE_COOKIE_FILE env var to use one)");
    }

    // Proxy settings
    let http_proxy = env::var("HTTP_PROXY").ok();
    let https_proxy = env::var("HTTPS_PROXY").ok();
    let using_proxy = http_proxy.is_some() || https_proxy.is_some();

    if using_proxy {
        println!("Using proxy configuration from environment variables");
    } else {
        println!("No proxy configuration found (set HTTP_PROXY/HTTPS_PROXY env vars to use one)");
    }

    // Video ID
    let video_id = env::var("VIDEO_ID").unwrap_or_else(|_| {
        if cookie_file.is_some() {
            // If we have cookie auth, try a potentially age-restricted video
            "kJQP7kiw5Fk".to_string() // "Despacito" by Luis Fonsi
        } else {
            // Otherwise use a video without restrictions
            "arj7oStGLkU".to_string() // TED Talk
        }
    });

    println!("Using video ID: {}", video_id);

    // === 2. Initialize the API with our configuration ===

    // Set up proxy configuration if available
    let proxy_config = if using_proxy {
        match GenericProxyConfig::new(http_proxy.clone(), https_proxy.clone()) {
            Ok(config) => {
                println!("Successfully created proxy configuration");
                Some(Box::new(config) as Box<dyn ProxyConfig + Send + Sync>)
            }
            Err(e) => {
                println!("Failed to create proxy configuration: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Set up cookie path if available
    let cookie_path = cookie_file.as_ref().map(|path| Path::new(path));

    // Initialize the API
    let api = YouTubeTranscriptApi::new(cookie_path, proxy_config, None)?;

    // === 3. Fetch the transcript ===

    println!("\nFetching transcript for video ID: {}", video_id);
    println!("(The API will handle cookie consent if needed)");

    let language_codes = &["en", "es", "fr"]; // Try English, Spanish, French in order
    let preserve_formatting = false;

    match api
        .fetch_transcript(&video_id, language_codes, preserve_formatting)
        .await
    {
        Ok(transcript) => {
            println!("\n✅ Successfully fetched transcript!");
            println!("Video ID: {}", transcript.video_id);
            println!(
                "Language: {} ({})",
                transcript.language, transcript.language_code
            );
            println!("Is auto-generated: {}", transcript.is_generated);
            println!("Number of snippets: {}", transcript.snippets.len());

            // === 4. Format the transcript in different ways ===

            // JSON format
            let json_output = serde_json::to_string_pretty(&transcript).unwrap();
            println!("\n--- JSON Sample ---");
            println!(
                "{}",
                if json_output.len() > 200 {
                    format!("{}... (truncated)", &json_output[..200])
                } else {
                    json_output
                }
            );
        }
        Err(e) => {
            println!("\n❌ Failed to fetch transcript: {:?}", e);

            // === 5. Provide detailed error handling guidance ===

            let error_str = format!("{:?}", e);

            if error_str.contains("AgeRestricted") {
                println!("\nThis video is age-restricted. To access it, you need to:");
                println!("1. Export cookies from a browser where you're logged into YouTube");
                println!("2. Set the YOUTUBE_COOKIE_FILE environment variable to point to your cookie file");
                println!("3. Run this example again");
            } else if error_str.contains("IpBlocked") || error_str.contains("RequestBlocked") {
                println!("\nYouTube is blocking requests from your IP address.");
                println!("To bypass this restriction, you can:");
                println!("1. Set up an HTTP proxy by setting the HTTP_PROXY and HTTPS_PROXY environment variables");
                println!("2. Wait a while before trying again from your current IP");
                println!("3. Try using a VPN service");
            } else if error_str.contains("FailedToCreateConsentCookie") {
                println!("\nCouldn't automatically handle YouTube's cookie consent requirements.");
                println!("To resolve this:");
                println!("1. Export cookies from a browser where you've already accepted YouTube's consent");
                println!("2. Set the YOUTUBE_COOKIE_FILE environment variable to point to your cookie file");
            } else if error_str.contains("NoTranscriptFound") {
                println!("\nNo transcript was found for this video in the requested languages.");
                println!("You can:");
                println!(
                    "1. Try different language codes (the example tried: {})",
                    language_codes.join(", ")
                );
                println!("2. First check available transcripts using the list() method");
                println!("   (see the youtube_transcript_list example)");
            }
        }
    }

    Ok(())
}
