use anyhow::Result;
use std::env;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use yt_transcript_rs::proxies::{GenericProxyConfig, ProxyConfig, WebshareProxyConfig};

/// This example demonstrates how to use proxies with the YouTube Transcript API.
///
/// It shows:
/// 1. Setting up the YouTubeTranscriptApi with different proxy configurations
///    - GenericProxyConfig for standard HTTP/HTTPS proxies
///    - WebshareProxyConfig for Webshare proxy service
/// 2. Fetching a transcript using a proxy
///
/// Note: You'll need to set the following environment variables to run this example:
/// - HTTP_PROXY: Your HTTP proxy URL (for GenericProxyConfig)
/// - HTTPS_PROXY: Your HTTPS proxy URL (for GenericProxyConfig)
/// - WEBSHARE_USER: Your Webshare username (for WebshareProxyConfig)
/// - WEBSHARE_PASSWORD: Your Webshare password (for WebshareProxyConfig)
#[tokio::main]
async fn main() -> Result<()> {
    // Get proxy configuration from environment variables
    let http_proxy = env::var("HTTP_PROXY").ok();
    let https_proxy = env::var("HTTPS_PROXY").ok();

    let has_http_proxy = http_proxy.is_some() || https_proxy.is_some();

    let webshare_user = env::var("WEBSHARE_USER").ok();
    let webshare_password = env::var("WEBSHARE_PASSWORD").ok();

    let has_webshare = webshare_user.is_some() && webshare_password.is_some();

    // Video ID to fetch a transcript for
    let video_id = "arj7oStGLkU";

    // Example 1: Using a standard HTTP/HTTPS proxy
    if has_http_proxy {
        println!("Example 1: Using GenericProxyConfig with HTTP/HTTPS proxy");

        // Create a GenericProxyConfig
        match GenericProxyConfig::new(http_proxy.clone(), https_proxy.clone()) {
            Ok(proxy_config) => {
                // Create the API instance with the proxy configuration
                let api = YouTubeTranscriptApi::new(
                    None,
                    Some(Box::new(proxy_config) as Box<dyn ProxyConfig + Send + Sync>),
                    None,
                )?;

                // Try to fetch a transcript
                match api.fetch_transcript(video_id, &["en"], false).await {
                    Ok(transcript) => {
                        println!("Successfully fetched transcript using HTTP/HTTPS proxy!");
                        println!("Number of snippets: {}", transcript.snippets.len());

                        // Display the first few snippets
                        for (_i, snippet) in transcript.snippets.iter().take(3).enumerate() {
                            println!(
                                "[{:.1}-{:.1}s] {}",
                                snippet.start,
                                snippet.start + snippet.duration,
                                snippet.text
                            );
                        }
                    }
                    Err(e) => {
                        println!("Failed to fetch transcript with HTTP/HTTPS proxy: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to create HTTP/HTTPS proxy config: {}", e);
            }
        }
    } else {
        println!("Skipping HTTP/HTTPS proxy example (HTTP_PROXY or HTTPS_PROXY not set)");
    }

    // Example 2: Using Webshare proxy service
    if has_webshare {
        println!("\nExample 2: Using WebshareProxyConfig");

        // Create a WebshareProxyConfig with default settings
        let proxy_config = WebshareProxyConfig::new(
            webshare_user.unwrap(),
            webshare_password.unwrap(),
            3,
            None,
            None,
        );

        // Create the API instance with the proxy configuration
        let api = YouTubeTranscriptApi::new(
            None,
            Some(Box::new(proxy_config) as Box<dyn ProxyConfig + Send + Sync>),
            None,
        )?;

        // Try to fetch a transcript
        match api.fetch_transcript(video_id, &["en"], false).await {
            Ok(transcript) => {
                println!("Successfully fetched transcript using Webshare proxy!");
                println!("Number of snippets: {}", transcript.snippets.len());

                // Display the first few snippets
                for (_i, snippet) in transcript.snippets.iter().take(3).enumerate() {
                    println!(
                        "[{:.1}-{:.1}s] {}",
                        snippet.start,
                        snippet.start + snippet.duration,
                        snippet.text
                    );
                }
            }
            Err(e) => {
                println!("Failed to fetch transcript with Webshare proxy: {:?}", e);
            }
        }
    } else {
        println!("Skipping Webshare proxy example (WEBSHARE_USER or WEBSHARE_PASSWORD not set)");
    }

    // If no proxies were configured, show an example with no proxy
    if !has_http_proxy && !has_webshare {
        println!("\nExample 3: Using no proxy");

        // Create the API instance with no proxy
        let api = YouTubeTranscriptApi::new(None, None, None)?;

        // Try to fetch a transcript
        match api.fetch_transcript(video_id, &["en"], false).await {
            Ok(transcript) => {
                println!("Successfully fetched transcript without proxy!");
                println!("Number of snippets: {}", transcript.snippets.len());

                // Display the first few snippets
                for (_i, snippet) in transcript.snippets.iter().take(3).enumerate() {
                    println!(
                        "[{:.1}-{:.1}s] {}",
                        snippet.start,
                        snippet.start + snippet.duration,
                        snippet.text
                    );
                }
            }
            Err(e) => {
                println!("Failed to fetch transcript without proxy: {:?}", e);
            }
        }
    }

    Ok(())
}
