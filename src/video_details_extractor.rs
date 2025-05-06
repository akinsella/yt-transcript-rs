use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use crate::models::{VideoDetails, VideoThumbnail};

/// # VideoDetailsExtractor
///
/// Specialized component for extracting detailed metadata from YouTube video player responses.
///
/// This utility struct provides functionality to parse YouTube's internal JSON structures
/// and extract comprehensive video details including title, author, thumbnails, and other
/// metadata.
///
/// The extractor works with the JSON data contained in the `ytInitialPlayerResponse` variable
/// that can be found in YouTube's video page HTML. It uses type-safe extraction with fallbacks
/// to ensure robustness against YouTube's format changes.
///
/// ## Usage Flow
///
/// 1. The `VideoDataFetcher` retrieves the raw player response JSON from YouTube
/// 2. This extractor parses the "videoDetails" section of that JSON
/// 3. It handles missing fields gracefully by providing default values
///
/// ## Extracted Data
///
/// The extractor can retrieve the following details:
/// - Basic info (title, author, video ID)
/// - Channel information
/// - Video statistics (view count, length)
/// - Thumbnails in various resolutions
/// - Keywords and description
/// - Live content status
pub struct VideoDetailsExtractor;

impl VideoDetailsExtractor {
    /// Extracts comprehensive video details from YouTube's player response JSON.
    ///
    /// This method parses the `videoDetails` section of YouTube's player response
    /// and constructs a structured `VideoDetails` object containing metadata about the video.
    /// It applies sensible defaults for any missing fields to ensure the returned object
    /// is always complete.
    ///
    /// # Parameters
    ///
    /// * `player_response` - The parsed JSON containing YouTube's player response data
    /// * `video_id` - The YouTube video ID (used as fallback and for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<VideoDetails, CouldNotRetrieveTranscript>` - Structured video details on success,
    ///   or an error if the data cannot be parsed
    ///
    /// # Errors
    ///
    /// This method will return a `CouldNotRetrieveTranscript` error with reason
    /// `YouTubeDataUnparsable` if:
    /// - The `videoDetails` section is missing from the player response
    /// - Critical parsing errors occur during extraction
    ///
    /// # Example (internal usage)
    ///
    /// ```rust,no_run
    /// # use serde_json::json;
    /// # use yt_transcript_rs::video_details_extractor::VideoDetailsExtractor;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Example player response (simplified)
    /// let player_response = json!({
    ///     "videoDetails": {
    ///         "videoId": "dQw4w9WgXcQ",
    ///         "title": "Rick Astley - Never Gonna Give You Up",
    ///         "lengthSeconds": "212",
    ///         "author": "Rick Astley",
    ///         "viewCount": "1234567890",
    ///         "thumbnail": {
    ///             "thumbnails": [
    ///                 {
    ///                     "url": "https://i.ytimg.com/vi/dQw4w9WgXcQ/default.jpg",
    ///                     "width": 120,
    ///                     "height": 90
    ///                 }
    ///             ]
    ///         }
    ///     }
    /// });
    ///
    /// // Extract video details
    /// let video_details = VideoDetailsExtractor::extract_video_details(
    ///     &player_response,
    ///     "dQw4w9WgXcQ"
    /// )?;
    ///
    /// println!("Title: {}", video_details.title);
    /// println!("Author: {}", video_details.author);
    /// # Ok(())
    /// # }
    /// ```
    pub fn extract_video_details(
        player_response: &serde_json::Value,
        video_id: &str,
    ) -> Result<VideoDetails, CouldNotRetrieveTranscript> {
        match player_response.get("videoDetails") {
            Some(details) => {
                // Extract thumbnail data
                let thumbnails = match details
                    .get("thumbnail")
                    .and_then(|t| t.get("thumbnails"))
                    .and_then(|t| t.as_array())
                {
                    Some(thumbs) => thumbs
                        .iter()
                        .filter_map(|thumb| {
                            let url = thumb.get("url")?.as_str()?;
                            let width = thumb.get("width")?.as_u64()? as u32;
                            let height = thumb.get("height")?.as_u64()? as u32;

                            Some(VideoThumbnail {
                                url: url.to_string(),
                                width,
                                height,
                            })
                        })
                        .collect(),
                    None => Vec::new(),
                };

                // Extract keywords
                let keywords = details
                    .get("keywords")
                    .and_then(|k| k.as_array())
                    .map(|kw_array| {
                        kw_array
                            .iter()
                            .filter_map(|k| k.as_str().map(|s| s.to_string()))
                            .collect()
                    });

                // Extract other fields with defaults for missing fields
                Ok(VideoDetails {
                    video_id: details
                        .get("videoId")
                        .and_then(|v| v.as_str())
                        .unwrap_or(video_id)
                        .to_string(),
                    title: details
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown Title")
                        .to_string(),
                    length_seconds: details
                        .get("lengthSeconds")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0),
                    keywords,
                    channel_id: details
                        .get("channelId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    short_description: details
                        .get("shortDescription")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    view_count: details
                        .get("viewCount")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    author: details
                        .get("author")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    thumbnails,
                    is_live_content: details
                        .get("isLiveContent")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                })
            }
            None => Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable),
            }),
        }
    }
}
