use anyhow::Result;
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Command line arguments for the HTML content extractor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the HTML file to parse
    #[arg(short, long)]
    input: PathBuf,
}

/// This example demonstrates how to extract information from a YouTube HTML page
/// saved to a file. It parses the HTML to find key YouTube data structures
/// like ytInitialPlayerResponse, which contains metadata about the video.
///
/// It shows:
/// 1. Reading an HTML file containing YouTube page content
/// 2. Extracting the ytInitialPlayerResponse JavaScript variable
/// 3. Parsing the JSON data to access video details
/// 4. Displaying the extracted information in a structured format
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("YouTube HTML Content Extractor Example");
    println!("-------------------------------------");

    println!("Reading HTML file: {}", args.input.display());

    // Read the HTML content from the file
    let html_content = fs::read_to_string(&args.input)?;
    // println!("HTML content: {}", html_content);

    // Extract the ytInitialPlayerResponse JavaScript variable
    println!("Extracting player response data...");
    let player_response = extract_yt_initial_player_response(&html_content)?;
    println!("Player response: {}", player_response);

    // Parse the player response to access structured data
    println!("Parsing response data...");
    let video_info = VideoInfo::from_player_response(&player_response)?;
    println!("Video info: {:?}", video_info);

    // Display the extracted information
    display_video_info(&video_info);

    Ok(())
}

/// Extracts the ytInitialPlayerResponse JavaScript variable from YouTube's HTML
fn extract_yt_initial_player_response(html: &str) -> Result<serde_json::Value> {
    // Regular expression to match the ytInitialPlayerResponse variable
    let re = Regex::new(r"var\s+ytInitialPlayerResponse\s*=\s*(\{.+?\});\s*(?:var|if|</script>)")?;

    if let Some(captures) = re.captures(html) {
        if let Some(json_str) = captures.get(1) {
            // Parse the JSON string
            let player_response: serde_json::Value = serde_json::from_str(json_str.as_str())?;
            return Ok(player_response);
        }
    }

    anyhow::bail!("Could not find ytInitialPlayerResponse in the HTML content")
}

/// Structured representation of video information
#[derive(Debug, Serialize, Deserialize)]
struct VideoInfo {
    // Basic video details
    title: String,
    author: String,
    view_count: Option<String>,
    video_id: String,
    length_seconds: Option<i64>,
    keywords: Option<Vec<String>>,
    is_live: Option<bool>,

    // Accessibility details
    has_captions: bool,
    available_caption_languages: Vec<CaptionLanguageInfo>,

    // Category and rating information
    category: Option<String>,
    is_family_safe: Option<bool>,

    // Available formats
    available_formats: Vec<FormatInfo>,
}

/// Information about a caption language
#[derive(Debug, Serialize, Deserialize)]
struct CaptionLanguageInfo {
    language: String,
    language_code: String,
    is_auto_generated: bool,
    is_translatable: bool,
}

/// Information about a video format
#[derive(Debug, Serialize, Deserialize)]
struct FormatInfo {
    itag: i64,
    mime_type: String,
    quality: String,
    width: Option<i64>,
    height: Option<i64>,
    bitrate: Option<i64>,
}

impl VideoInfo {
    /// Creates a VideoInfo instance from the ytInitialPlayerResponse JSON
    fn from_player_response(response: &serde_json::Value) -> Result<Self> {
        // Extract video details
        let video_details = &response["videoDetails"];

        // Extract title and author
        let title = video_details["title"]
            .as_str()
            .unwrap_or("Unknown Title")
            .to_string();
        let author = video_details["author"]
            .as_str()
            .unwrap_or("Unknown Author")
            .to_string();

        // Extract view count
        let view_count = video_details["viewCount"].as_str().map(|s| s.to_string());

        // Extract video ID
        let video_id = video_details["videoId"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        // Extract video length
        let length_seconds = video_details["lengthSeconds"]
            .as_str()
            .and_then(|s| s.parse::<i64>().ok());

        // Extract keywords
        let keywords = if video_details["keywords"].is_array() {
            Some(
                video_details["keywords"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|k| k.as_str().map(|s| s.to_string()))
                    .collect(),
            )
        } else {
            None
        };

        // Extract live status
        let is_live = video_details["isLiveContent"].as_bool();

        // Extract caption info
        let captions = &response["captions"]["playerCaptionsTracklistRenderer"];
        let has_captions = captions.is_object() && captions["captionTracks"].is_array();

        // Extract available caption languages
        let mut available_caption_languages = Vec::new();
        if has_captions {
            if let Some(tracks) = captions["captionTracks"].as_array() {
                for track in tracks {
                    let language = track["name"]["simpleText"]
                        .as_str()
                        .unwrap_or_else(|| {
                            track["name"]["runs"][0]["text"]
                                .as_str()
                                .unwrap_or("Unknown")
                        })
                        .to_string();

                    let language_code = track["languageCode"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    let is_auto_generated = track["kind"].as_str() == Some("asr");

                    // Check if this track is translatable
                    let is_translatable = track["isTranslatable"].as_bool().unwrap_or(false);

                    available_caption_languages.push(CaptionLanguageInfo {
                        language,
                        language_code,
                        is_auto_generated,
                        is_translatable,
                    });
                }
            }
        }

        // Extract microformat data
        let microformat = &response["microformat"]["playerMicroformatRenderer"];
        let category = microformat["category"].as_str().map(|s| s.to_string());
        let is_family_safe = microformat["isFamilySafe"].as_bool();

        // Extract streaming data
        let streaming_data = &response["streamingData"];
        let mut available_formats = Vec::new();

        // Process both formats and adaptiveFormats if they exist
        for formats_field in &["formats", "adaptiveFormats"] {
            if let Some(formats) = streaming_data[formats_field].as_array() {
                for format in formats {
                    if let (Some(itag), Some(mime_type), Some(quality)) = (
                        format["itag"].as_i64(),
                        format["mimeType"].as_str(),
                        format["qualityLabel"]
                            .as_str()
                            .or_else(|| format["quality"].as_str()),
                    ) {
                        let width = format["width"].as_i64();
                        let height = format["height"].as_i64();
                        let bitrate = format["bitrate"].as_i64();

                        available_formats.push(FormatInfo {
                            itag,
                            mime_type: mime_type.to_string(),
                            quality: quality.to_string(),
                            width,
                            height,
                            bitrate,
                        });
                    }
                }
            }
        }

        Ok(VideoInfo {
            title,
            author,
            view_count,
            video_id,
            length_seconds,
            keywords,
            is_live,
            has_captions,
            available_caption_languages,
            category,
            is_family_safe,
            available_formats,
        })
    }
}

/// Displays the extracted video information in a formatted way
fn display_video_info(info: &VideoInfo) {
    println!("\n📝 Video Details:");
    println!("----------------");
    println!("Title: {}", info.title);
    println!("Author: {}", info.author);

    if let Some(view_count) = &info.view_count {
        println!("View Count: {}", view_count);
    }

    if let Some(length) = info.length_seconds {
        let hours = length / 3600;
        let minutes = (length % 3600) / 60;
        let seconds = length % 60;

        if hours > 0 {
            println!("Length: {}:{:02}:{:02}", hours, minutes, seconds);
        } else {
            println!("Length: {}:{:02}", minutes, seconds);
        }
    }

    if let Some(is_live) = info.is_live {
        println!("Live Content: {}", if is_live { "Yes" } else { "No" });
    }

    // Print keywords if available
    if let Some(keywords) = &info.keywords {
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

    // Print category and content rating information
    println!("\n🌐 Content Information:");
    println!("---------------------");

    if let Some(category) = &info.category {
        println!("Category: {}", category);
    }

    if let Some(is_family_safe) = info.is_family_safe {
        println!("Family Safe: {}", if is_family_safe { "Yes" } else { "No" });
    }

    // Print caption/transcript information
    println!("\n📄 Caption Information:");
    println!("---------------------");
    println!(
        "Has Captions: {}",
        if info.has_captions { "Yes" } else { "No" }
    );

    if info.has_captions {
        println!(
            "Available Languages: {}",
            info.available_caption_languages.len()
        );

        for (i, caption) in info.available_caption_languages.iter().enumerate().take(5) {
            let auto_gen = if caption.is_auto_generated {
                "Auto-generated"
            } else {
                "Manual"
            };
            let translatable = if caption.is_translatable {
                "✓"
            } else {
                "✗"
            };

            println!(
                "  {}: {} ({}) [Type: {}, Translatable: {}]",
                i + 1,
                caption.language,
                caption.language_code,
                auto_gen,
                translatable
            );
        }

        if info.available_caption_languages.len() > 5 {
            println!(
                "  ... and {} more",
                info.available_caption_languages.len() - 5
            );
        }
    }

    // Print video format information
    println!("\n🎬 Video Formats:");
    println!("----------------");
    println!("Available Formats: {}", info.available_formats.len());

    let mut video_formats = 0;
    let mut audio_formats = 0;
    let mut highest_resolution = 0;

    for format in &info.available_formats {
        if format.mime_type.starts_with("video/") {
            video_formats += 1;
            if let Some(height) = format.height {
                if height > highest_resolution {
                    highest_resolution = height;
                }
            }
        } else if format.mime_type.starts_with("audio/") {
            audio_formats += 1;
        }
    }

    println!("Video-only formats: {}", video_formats);
    println!("Audio-only formats: {}", audio_formats);

    if highest_resolution > 0 {
        println!("Highest resolution: {}p", highest_resolution);
    }

    // Print 5 example formats with details
    println!("\nFormat Examples:");
    for (i, format) in info.available_formats.iter().enumerate().take(5) {
        let resolution = match (format.width, format.height) {
            (Some(w), Some(h)) => format!("{}x{}", w, h),
            _ => "N/A".to_string(),
        };

        let bitrate = format.bitrate.map_or("N/A".to_string(), |b| {
            format!("{:.1} Mbps", b as f64 / 1_000_000.0)
        });

        println!(
            "  {}: {} - {} [{}] ({})",
            i + 1,
            format.quality,
            format
                .mime_type
                .split(';')
                .next()
                .unwrap_or(&format.mime_type),
            resolution,
            bitrate
        );
    }

    if info.available_formats.len() > 5 {
        println!(
            "  ... and {} more formats",
            info.available_formats.len() - 5
        );
    }

    println!("\n✨ Data successfully extracted from HTML file!");
}
