# yt-transcript-rs

`yt-transcript-rs` is a Rust library for fetching and working with YouTube video transcripts. It allows you to retrieve transcripts in various languages, list available transcripts for a video, and fetch video details.

[![Crates.io](https://img.shields.io/crates/v/yt-transcript-rs.svg)](https://crates.io/crates/yt-transcript-rs)
[![Documentation](https://docs.rs/yt-transcript-rs/badge.svg)](https://docs.rs/yt-transcript-rs)
[![codecov](https://codecov.io/gh/akinsella/yt-transcript-rs/graph/badge.svg?token=08S6M9DDM9)](https://codecov.io/gh/akinsella/yt-transcript-rs)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

This project is heavily inspired by the Python module [youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) originally developed by [Jonas Depoix](https://github.com/jdepoix).

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Fetch a transcript](#fetch-a-transcript)
  - [List available transcripts](#list-available-transcripts)
  - [Fetch video details](#fetch-video-details)
  - [Fetch microformat data](#fetch-microformat-data)
  - [Fetch streaming data](#fetch-streaming-data)
- [Requirements](#requirements)
- [Advanced Usage](#advanced-usage)
  - [Using Proxies](#using-proxies)
  - [Using Cookie Authentication](#using-cookie-authentication)
- [Error Handling](#error-handling)
- [License](#license)
- [Contributing](#contributing)
- [Acknowledgments](#acknowledgments)

## Features

- Fetch transcripts from YouTube videos in various languages
- List all available transcripts for a video
- Retrieve translations of transcripts
- Get detailed information about YouTube videos
- Access video microformat data including available countries and embed information
- Retrieve streaming formats and quality options for videos
- Support for proxy configuration and cookie authentication

## Installation

Add `yt-transcript-rs` to your `Cargo.toml`:

```bash
cargo add yt-transcript-rs
```

Or manually add it to your `Cargo.toml`:

```toml
[dependencies]
yt-transcript-rs = "0.1.0"  # Replace with the latest version
```

## Usage

### Fetch a transcript

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to fetch a transcript from a YouTube video.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Fetching a transcript for a video in a specific language
/// 3. Displaying the transcript content
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the YouTubeTranscriptApi
    // This creates a new instance without proxy or cookie authentication
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "5MuIMqhT8DM";

    // Language preference (English)
    let languages = &["en"];

    // Don't preserve formatting (remove line breaks, etc.)
    let preserve_formatting = false;

    // Fetch the transcript
    println!("Fetching transcript for video ID: {}", video_id);

    match api.fetch_transcript(video_id, languages, preserve_formatting).await {
        Ok(transcript) => {
            println!("Successfully fetched transcript!");
            println!("Video ID: {}", transcript.video_id);
            println!(
                "Language: {} ({})",
                transcript.language, transcript.language_code
            );
            println!("Is auto-generated: {}", transcript.is_generated);
            println!("Number of snippets: {}", transcript.snippets.len());
            println!("\nTranscript content:");

            // Display the first 5 snippets
            for (_i, snippet) in transcript.snippets.iter().take(5).enumerate() {
                println!(
                    "[{:.1}-{:.1}s] {}",
                    snippet.start,
                    snippet.start + snippet.duration,
                    snippet.text
                );
            }

            println!("... (truncated)");
        }
        Err(e) => {
            println!("Failed to fetch transcript: {:?}", e);
        }
    }

    Ok(())
}
```


### List available transcripts

```rust
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
```

### Fetch video details

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to fetch video details using the YouTube Transcript API.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Fetching video details for a given video ID
/// 3. Displaying the video information including title, author, view count, etc.
#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Video Details Example");
    println!("------------------------------");

    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    println!("Fetching video details for: {}", video_id);

    match api.fetch_video_details(video_id).await {
        Ok(details) => {
            println!("\nVideo Details:");
            println!("-------------");
            println!("Video ID: {}", details.video_id);
            println!("Title: {}", details.title);
            println!("Author: {}", details.author);
            println!("Channel ID: {}", details.channel_id);
            println!("View Count: {}", details.view_count);
            println!("Length: {} seconds", details.length_seconds);
            println!("Is Live Content: {}", details.is_live_content);

            // Print keywords if available
            if let Some(keywords) = details.keywords {
                println!("\nKeywords:");
                for (i, keyword) in keywords.iter().enumerate().take(10) {
                    println!("  {}: {}", i + 1, keyword);
                }

                if keywords.len() > 10 {
                    println!("  ... and {} more", keywords.len() - 10);
                }
            }

            // Print thumbnail information
            println!("\nThumbnails: {} available", details.thumbnails.len());
            for (i, thumb) in details.thumbnails.iter().enumerate() {
                println!(
                    "  {}: {}x{} - {}",
                    i + 1,
                    thumb.width,
                    thumb.height,
                    thumb.url
                );
            }

            // Print a truncated description
            println!("\nDescription:");
            let description = if details.short_description.len() > 300 {
                format!("{}...", &details.short_description[..300])
            } else {
                details.short_description.clone()
            };
            println!("{}", description);
        }
        Err(e) => {
            println!("Failed to fetch video details: {:?}", e);
        }
    }

    Ok(())
}

### Fetch microformat data

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    println!("Fetching microformat data for: {}", video_id);

    match api.fetch_microformat(video_id).await {
        Ok(microformat) => {
            println!("\nMicroformat Data:");
            println!("-----------------");

            // Print video title and channel info
            if let Some(title) = &microformat.title {
                println!("Title: {}", title);
            }
            if let Some(channel) = &microformat.owner_channel_name {
                println!("Channel: {}", channel);
            }

            // Print video stats
            if let Some(views) = &microformat.view_count {
                println!("View Count: {}", views);
            }
            if let Some(likes) = &microformat.like_count {
                println!("Like Count: {}", likes);
            }

            // Print video status and category
            if let Some(category) = &microformat.category {
                println!("Category: {}", category);
            }
            if let Some(is_unlisted) = microformat.is_unlisted {
                println!("Is Unlisted: {}", is_unlisted);
            }
            if let Some(is_family_safe) = microformat.is_family_safe {
                println!("Is Family Safe: {}", is_family_safe);
            }

            // Print countries where video is available
            if let Some(countries) = &microformat.available_countries {
                println!("Available in {} countries", countries.len());
            }

            // Print embed information
            if let Some(embed) = &microformat.embed {
                if let Some(iframe_url) = &embed.iframe_url {
                    println!("Embed URL: {}", iframe_url);
                }
            }
        }
        Err(e) => {
            println!("Failed to fetch microformat data: {:?}", e);
        }
    }

    Ok(())
}
```

### Fetch streaming data

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Streaming Data Example");
    println!("------------------------------");

    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    println!("Fetching streaming data for: {}", video_id);

    match api.fetch_streaming_data(video_id).await {
        Ok(streaming_data) => {
            println!("\nStreaming Data:");
            println!("--------------");
            println!("Expires in: {} seconds", streaming_data.expires_in_seconds);
            
            // Display basic format counts
            println!("\nCombined Formats (video+audio): {}", streaming_data.formats.len());
            println!("Adaptive Formats: {}", streaming_data.adaptive_formats.len());
            
            // Example of accessing video format information
            if let Some(format) = streaming_data.formats.first() {
                println!("\nSample format information:");
                println!("  ITAG: {}", format.itag);
                if let (Some(w), Some(h)) = (format.width, format.height) {
                    println!("  Resolution: {}x{}", w, h);
                }
                println!("  Bitrate: {} bps", format.bitrate);
                println!("  MIME type: {}", format.mime_type);
            }
            
            // Count video and audio format types
            let video_count = streaming_data.adaptive_formats
                .iter()
                .filter(|f| f.mime_type.starts_with("video/"))
                .count();
                
            let audio_count = streaming_data.adaptive_formats
                .iter()
                .filter(|f| f.mime_type.starts_with("audio/"))
                .count();
                
            println!("\nAdaptive format breakdown:");
            println!("  Video formats: {}", video_count);
            println!("  Audio formats: {}", audio_count);
        }
        Err(e) => {
            println!("Failed to fetch streaming data: {:?}", e);
        }
    }

    Ok(())
}
```

## Requirements

- Rust 1.56 or higher
- `tokio` for async execution

## Advanced Usage

### Using Proxies

You can configure the API to use a proxy server:

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use yt_transcript_rs::proxies::ProxyConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a proxy configuration
    let proxy = Box::new(ProxyConfig {
        url: "http://your-proxy-server:8080".to_string(),
        username: Some("username".to_string()),
        password: Some("password".to_string()),
    });

    // Initialize the API with proxy
    let api = YouTubeTranscriptApi::new(Some(proxy), None, None)?;
    
    // Use the API as normal
    let video_id = "5MuIMqhT8DM";
    let languages = &["en"];
    let transcript = api.fetch_transcript(video_id, languages, false).await?;
    
    println!("Fetched transcript via proxy!");
    
    Ok(())
}
```

### Using Cookie Authentication

For videos that require authentication:

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Provide path to cookies file exported from browser
    let cookie_path = Path::new("path/to/cookies.txt");
    
    // Initialize the API with cookies
    let api = YouTubeTranscriptApi::new(Some(cookie_path.as_ref()), None, None)?;
    
    // Fetch transcript for a video that requires authentication
    let video_id = "private_video_id";
    let languages = &["en"];
    let transcript = api.fetch_transcript(video_id, languages, false).await?;
    
    println!("Successfully authenticated and fetched transcript!");
    
    Ok(())
}
```

## Error Handling

The library provides specific error types for handling different failure scenarios:

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use yt_transcript_rs::errors::CouldNotRetrieveTranscriptReason;

#[tokio::main]
async fn main() -> Result<()> {
    let api = YouTubeTranscriptApi::new(None, None, None)?;
    let video_id = "5MuIMqhT8DM";
    
    match api.fetch_transcript(video_id, &["en"], false).await {
        Ok(transcript) => {
            println!("Successfully fetched transcript with {} snippets", transcript.snippets.len());
            Ok(())
        },
        Err(e) => {
            match e.reason {
                Some(CouldNotRetrieveTranscriptReason::NoTranscriptFound) => {
                    println!("No transcript found for this video");
                },
                Some(CouldNotRetrieveTranscriptReason::TranslationLanguageNotAvailable) => {
                    println!("The requested translation language is not available");
                },
                Some(CouldNotRetrieveTranscriptReason::VideoUnavailable) => {
                    println!("The video is unavailable or does not exist");
                },
                // Handle other specific errors
                _ => println!("Other error: {:?}", e),
            }
            Err(e.into())
        }
    }
}
```

## License

This project is licensed under the [MIT License](LICENSE) - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Here's how you can contribute:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/yt-transcript-rs.git
cd yt-transcript-rs

# Build the project
cargo build

# Run tests
cargo test
```

## Acknowledgments

- [Jonas Depoix](https://github.com/jdepoix) for the original [youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) Python library
- All contributors who have helped improve this library

