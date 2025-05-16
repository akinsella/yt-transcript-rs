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
  - [Fetch all video information at once](#fetch-all-video-information-at-once)
  - [Customize transcript formatting](#customize-transcript-formatting)
- [Requirements](#requirements)
- [Advanced Usage](#advanced-usage)
  - [Using Proxies](#using-proxies)
  - [Using Cookie Authentication](#using-cookie-authentication)
  - [Serializing and Deserializing Video Information](#serializing-and-deserializing-video-information)
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
- Fetch all video information in a single request for optimal performance
- Support for proxy configuration and cookie authentication
- Customizable HTML processing with configurable link formatting
- Robust HTML entity handling and whitespace preservation

## Installation

Add `yt-transcript-rs` to your `Cargo.toml`:

```bash
cargo add yt-transcript-rs
```

Or manually add it to your `Cargo.toml`:

```toml
[dependencies]
yt-transcript-rs = "0.1.6"  # Replace with the latest version
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

### Fetch all video information at once

```rust
use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

#[tokio::main]
async fn main() -> Result<()> {
    println!("YouTube Video Infos (All-in-One) Example");
    println!("----------------------------------------");

    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Ted Talk video ID
    let video_id = "arj7oStGLkU";

    println!("Fetching all video information in a single request...");

    match api.fetch_video_infos(video_id).await {
        Ok(infos) => {
            // Access video details
            println!("\nVideo Details:");
            println!("Title: {}", infos.video_details.title);
            println!("Author: {}", infos.video_details.author);
            println!("Length: {} seconds", infos.video_details.length_seconds);
            
            // Access microformat data
            if let Some(category) = &infos.microformat.category {
                println!("Category: {}", category);
            }
            
            if let Some(countries) = &infos.microformat.available_countries {
                println!("Available in {} countries", countries.len());
            }
            
            // Access streaming data
            println!("\nStreaming Options:");
            println!("Video formats: {}", infos.streaming_data.formats.len());
            println!("Adaptive formats: {}", infos.streaming_data.adaptive_formats.len());
            
            // Find highest resolution
            let highest_res = infos.streaming_data.adaptive_formats
                .iter()
                .filter_map(|f| f.height)
                .max()
                .unwrap_or(0);
            println!("Highest resolution: {}p", highest_res);
            
            // Access transcript information
            let transcript_count = infos.transcript_list.transcripts().count();
            println!("\nAvailable transcripts: {}", transcript_count);
            
            println!("\nAll data retrieved in a single network request!");
        }
        Err(e) => {
            println!("Failed to fetch video information: {:?}", e);
        }
    }

    Ok(())
}
```

### Customize transcript formatting

```rust
use anyhow::Result;
use yt_transcript_rs::transcript_parser::TranscriptParser;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a transcript parser with default settings (plain text, standard link format)
    let default_parser = TranscriptParser::new(false);
    
    // Create a parser that preserves HTML formatting tags
    let formatted_parser = TranscriptParser::new(true);
    
    // Create a parser with custom link format (Markdown style)
    let markdown_parser = TranscriptParser::with_config(false, "[{text}]({url})")?;
    
    // Create a parser with custom link format (HTML style)
    let html_parser = TranscriptParser::with_config(false, "<a href=\"{url}\">{text}</a>")?;
    
    // Sample HTML content with a link
    let html = r#"<p>Check out <a href="https://example.com">this link</a> for more info.</p>"#;
    
    // Process the HTML with different parsers
    let default_text = default_parser.html_to_plain_text(html);
    let formatted_text = formatted_parser.process_with_formatting(html);
    let markdown_text = markdown_parser.html_to_plain_text(html);
    let html_text = html_parser.html_to_plain_text(html);
    
    println!("Default format: {}", default_text);
    println!("Preserved HTML: {}", formatted_text);
    println!("Markdown links: {}", markdown_text);
    println!("HTML links: {}", html_text);
    
    Ok(())
}
```

This feature is particularly useful when you need to:
- Format transcript links according to specific output needs
- Create transcripts for different display contexts (web, terminal, documents)
- Preserve certain HTML tags for styling while removing others
- Ensure proper entity decoding for symbols like apostrophes and quotes

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

### Serializing and Deserializing Video Information

You can serialize video information for storage or transmission between systems. The library provides full support for `serde` serialization and deserialization of the `VideoInfos` struct and related types.

```rust
use anyhow::Result;
use reqwest::Client;
use yt_transcript_rs::api::YouTubeTranscriptApi;
use yt_transcript_rs::models::VideoInfos;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the YouTubeTranscriptApi
    let api = YouTubeTranscriptApi::new(None, None, None)?;

    // Fetch video information
    let video_id = "dQw4w9WgXcQ";
    let infos = api.fetch_video_infos(video_id).await?;
    
    // Serialize to JSON
    let json = serde_json::to_string(&infos)?;
    println!("Serialized data size: {} bytes", json.len());
    
    // Save to file, send over network, etc.
    std::fs::write("video_info.json", &json)?;
    
    // Later, deserialize back from JSON
    let json = std::fs::read_to_string("video_info.json")?;
    let deserialized = serde_json::from_str::<VideoInfos>(&json)?;
    
    // The deserialized object has all the same data
    println!("Title: {}", deserialized.video_details.title);
    
    // To fetch a transcript from the deserialized data, you need to provide a client
    let client = Client::new();
    if let Ok(transcript) = deserialized.transcript_list.find_transcript(&["en"]) {
        let fetched = transcript.fetch(&client, false).await?;
        println!("Transcript text: {}", fetched.text());
    }
    
    Ok(())
}
```

The serialization support makes it easy to:
- Cache video information to reduce YouTube API requests
- Send video data between microservices
- Store video information in databases
- Create backup systems for important videos

Note that when deserializing a `Transcript`, you'll need to provide a `Client` when calling `fetch()`, as HTTP clients cannot be serialized.

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

## Release Process

Releases are automated through GitHub Actions workflows. To create a new release:

1. Update the version in `Cargo.toml`
2. Update the changelog in `CHANGELOG.md`
3. Create and push a new tag:

```bash
VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
git tag v$VERSION && git push origin v$VERSION
```

This will automatically trigger the release workflow which will:
- Build the crate
- Run tests
- Create a GitHub release
- Publish to crates.io

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
git clone https://github.com/akinsella/yt-transcript-rs.git
cd yt-transcript-rs

# Build the project
cargo build

# Run tests
cargo test

# Run Clippy with strict settings for code quality
cargo clippy --all-targets --features ci --all-features -- -D warnings

# Run Clippy in fix mode to automatically apply suggested fixes
cargo clippy --all-targets --features ci --all-features --fix -- -D warnings

# Format code according to Rust style guidelines
cargo fmt

# Format and overwrite files with the formatting changes
cargo fmt --all

### Setting up cargo-husky for Git Hooks

Follow these steps to ensure cargo-husky is properly installed and configured:

1. **Install cargo-husky as a dev dependency**:

   ```bash
   cargo add --dev cargo-husky
   ```

2. **Configure cargo-husky in Cargo.toml**:

   Add the following to your `Cargo.toml` file:

   ```toml
   [dev-dependencies]
   cargo-husky = { version = "1", features = ["precommit-hook", "run-cargo-fmt", "run-cargo-clippy", "run-cargo-check"] }
   ```

3. **Verify the installation**:

   After adding cargo-husky, run `cargo build` once to ensure the git hooks are installed:

   ```bash
   cargo build
   ```

4. **Verify the hooks were created**:

   Check if the pre-commit hook file was created:

   ```bash
   ls -la .git/hooks/pre-commit
   ```

   You should see a pre-commit file, and it should be executable.

5. **Test the pre-commit hook**:

   Make a small change to any file, then try to commit it:

   ```bash
   # Make a change
   echo "// Test comment" >> src/lib.rs
   
   # Add the change
   git add src/lib.rs
   
   # Try to commit
   git commit -m "Test commit"
   ```

   If the hook is working correctly, it should run:
   - `cargo fmt` to format the code
   - `cargo check` to verify compilation
   - `cargo clippy` to check for lints

6. **Troubleshooting**:

   If the hooks aren't running:
   
   - Make sure the hook file is executable: `chmod +x .git/hooks/pre-commit`
   - Try rebuilding the project: `cargo clean && cargo build`
   - Check the content of the pre-commit file to ensure it's correct

7. **Customizing hook behavior**:

   You can customize the hook behavior by adding a `.cargo-husky/hooks/pre-commit` file with your custom script. cargo-husky will use this file instead of generating its own.

8. **Skipping hooks when needed**:

   In rare cases when you need to bypass the hooks, you can use:

   ```bash
   git commit -m "Your message" --no-verify
   ```
   
   However, this should be used sparingly and only in exceptional circumstances.

By following these steps, cargo-husky will enforce code quality standards on every commit, helping maintain a clean and consistent codebase.

## Acknowledgments

- [Jonas Depoix](https://github.com/jdepoix) for the original [youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) Python library
- All contributors who have helped improve this library
