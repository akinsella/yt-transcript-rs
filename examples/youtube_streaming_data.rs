use anyhow::Result;
use yt_transcript_rs::api::YouTubeTranscriptApi;

/// This example demonstrates how to fetch streaming data from a YouTube video.
///
/// It shows:
/// 1. Creating a YouTubeTranscriptApi instance
/// 2. Fetching streaming data for a given video ID
/// 3. Displaying the available video and audio formats with their properties
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

            // Display combined formats (with both video and audio)
            println!(
                "\nCombined Formats (video+audio): {}",
                streaming_data.formats.len()
            );
            println!("--------------------------------------------------------");
            println!(
                "{:<5} {:<15} {:<10} {:<15} {:<10}",
                "itag", "Quality", "Type", "Resolution", "Bitrate"
            );
            println!("--------------------------------------------------------");

            for format in &streaming_data.formats {
                let resolution = match (format.width, format.height) {
                    (Some(w), Some(h)) => format!("{}x{}", w, h),
                    _ => "N/A".to_string(),
                };

                let quality = format.quality_label.as_deref().unwrap_or(&format.quality);
                let mime_parts: Vec<&str> = format.mime_type.split(';').collect();
                let mime_type = mime_parts[0];

                println!(
                    "{:<5} {:<15} {:<10} {:<15} {:<10}",
                    format.itag,
                    quality,
                    mime_type,
                    resolution,
                    format_bitrate(format.bitrate)
                );
            }

            // Display adaptive formats (separate video and audio streams)
            println!(
                "\nAdaptive Formats: {}",
                streaming_data.adaptive_formats.len()
            );
            println!("--------------------------------------------------------");
            println!(
                "{:<5} {:<10} {:<20} {:<15} {:<10} {:<15}",
                "itag", "Type", "Codec", "Resolution", "FPS", "Bitrate"
            );
            println!("--------------------------------------------------------");

            // First display video formats
            let video_formats: Vec<_> = streaming_data
                .adaptive_formats
                .iter()
                .filter(|f| f.mime_type.starts_with("video/"))
                .collect();

            println!("\nVideo Formats: {}", video_formats.len());
            for format in video_formats {
                let resolution = match (format.width, format.height) {
                    (Some(w), Some(h)) => format!("{}x{}", w, h),
                    _ => "N/A".to_string(),
                };

                let fps = format.fps.map_or("N/A".to_string(), |f| f.to_string());
                let codec = extract_codec(&format.mime_type);

                println!(
                    "{:<5} {:<10} {:<20} {:<15} {:<10} {:<15}",
                    format.itag,
                    "Video",
                    codec,
                    resolution,
                    fps,
                    format_bitrate(format.bitrate)
                );
            }

            // Then display audio formats
            let audio_formats: Vec<_> = streaming_data
                .adaptive_formats
                .iter()
                .filter(|f| f.mime_type.starts_with("audio/"))
                .collect();

            println!("\nAudio Formats: {}", audio_formats.len());
            for format in audio_formats {
                let quality = format.audio_quality.as_deref().unwrap_or("Unknown");
                let codec = extract_codec(&format.mime_type);
                let sample_rate = format.audio_sample_rate.as_deref().unwrap_or("N/A");
                let channels = format
                    .audio_channels
                    .map_or("N/A".to_string(), |c| c.to_string());

                println!(
                    "{:<5} {:<10} {:<20} {:<15} {:<10} {:<15}",
                    format.itag,
                    "Audio",
                    codec,
                    format!("{}ch, {}Hz", channels, sample_rate),
                    quality,
                    format_bitrate(format.bitrate)
                );
            }

            // Display additional format details for one high-quality format
            if let Some(hd_format) = streaming_data
                .adaptive_formats
                .iter()
                .find(|f| f.mime_type.starts_with("video/") && f.quality == "hd720")
            {
                println!(
                    "\nDetailed information for HD format (itag {}):",
                    hd_format.itag
                );
                println!("------------------------------------------------");
                println!("Quality: {}", hd_format.quality);
                println!(
                    "Quality label: {}",
                    hd_format.quality_label.as_deref().unwrap_or("N/A")
                );
                println!("MIME type: {}", hd_format.mime_type);
                println!(
                    "Resolution: {}x{}",
                    hd_format.width.unwrap_or(0),
                    hd_format.height.unwrap_or(0)
                );
                println!(
                    "Average bitrate: {} bps",
                    hd_format.average_bitrate.unwrap_or(0)
                );
                println!("Approximate duration: {} ms", hd_format.approx_duration_ms);

                if let Some(init_range) = &hd_format.init_range {
                    println!("Init range: {} - {}", init_range.start, init_range.end);
                }

                if let Some(index_range) = &hd_format.index_range {
                    println!("Index range: {} - {}", index_range.start, index_range.end);
                }

                if let Some(color_info) = &hd_format.color_info {
                    println!("Color info:");
                    println!(
                        "  Primaries: {}",
                        color_info.primaries.as_deref().unwrap_or("N/A")
                    );
                    println!(
                        "  Transfer: {}",
                        color_info
                            .transfer_characteristics
                            .as_deref()
                            .unwrap_or("N/A")
                    );
                    println!(
                        "  Matrix: {}",
                        color_info.matrix_coefficients.as_deref().unwrap_or("N/A")
                    );
                }
            }

            // Display server ABR streaming URL if available
            if let Some(abr_url) = &streaming_data.server_abr_streaming_url {
                println!(
                    "\nServer ABR streaming URL available: {}",
                    !abr_url.is_empty()
                );
            }
        }
        Err(e) => {
            println!("Failed to fetch streaming data: {:?}", e);
        }
    }

    Ok(())
}

/// Formats a bitrate value to a human-readable string
fn format_bitrate(bitrate: u64) -> String {
    if bitrate >= 1_000_000 {
        format!("{:.2} Mbps", bitrate as f64 / 1_000_000.0)
    } else {
        format!("{:.2} Kbps", bitrate as f64 / 1_000.0)
    }
}

/// Extracts the codec information from a MIME type string
fn extract_codec(mime_type: &str) -> String {
    if let Some(pos) = mime_type.find("codecs=") {
        let codec_part = &mime_type[pos + 8..];
        // Remove the quotes and trim
        codec_part
            .trim_start_matches('"')
            .trim_end_matches('"')
            .to_string()
    } else {
        "Unknown".to_string()
    }
}
