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
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                    "Missing videoDetails section in player response".to_string(),
                )),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_complete_video_details() {
        // Test case with a complete set of video details
        let video_id = "dQw4w9WgXcQ";

        let player_response = json!({
            "videoDetails": {
                "videoId": video_id,
                "title": "Rick Astley - Never Gonna Give You Up",
                "lengthSeconds": "212",
                "author": "Rick Astley",
                "channelId": "UCuAXFkgsw1L7xaCfnd5JJOw",
                "shortDescription": "Rick Astley's official music video for \"Never Gonna Give You Up\"",
                "viewCount": "1234567890",
                "keywords": ["Rick", "Astley", "Never", "Gonna", "Give", "You", "Up"],
                "isLiveContent": false,
                "thumbnail": {
                    "thumbnails": [
                        {
                            "url": "https://i.ytimg.com/vi/dQw4w9WgXcQ/default.jpg",
                            "width": 120,
                            "height": 90
                        },
                        {
                            "url": "https://i.ytimg.com/vi/dQw4w9WgXcQ/mqdefault.jpg",
                            "width": 320,
                            "height": 180
                        },
                        {
                            "url": "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg",
                            "width": 480,
                            "height": 360
                        }
                    ]
                }
            }
        });

        // Extract video details
        let result = VideoDetailsExtractor::extract_video_details(&player_response, video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify the extracted data
        assert_eq!(details.video_id, video_id);
        assert_eq!(details.title, "Rick Astley - Never Gonna Give You Up");
        assert_eq!(details.length_seconds, 212);
        assert_eq!(details.author, "Rick Astley");
        assert_eq!(details.channel_id, "UCuAXFkgsw1L7xaCfnd5JJOw");
        assert_eq!(
            details.short_description,
            "Rick Astley's official music video for \"Never Gonna Give You Up\""
        );
        assert_eq!(details.view_count, "1234567890");
        assert!(!details.is_live_content);

        // Verify keywords
        assert!(details.keywords.is_some());
        let keywords = details.keywords.unwrap();
        assert_eq!(keywords.len(), 7);
        assert!(keywords.contains(&"Rick".to_string()));
        assert!(keywords.contains(&"Astley".to_string()));

        // Verify thumbnails
        assert_eq!(details.thumbnails.len(), 3);
        assert_eq!(details.thumbnails[0].width, 120);
        assert_eq!(details.thumbnails[0].height, 90);
        assert_eq!(details.thumbnails[1].width, 320);
        assert_eq!(details.thumbnails[1].height, 180);
        assert_eq!(details.thumbnails[2].width, 480);
        assert_eq!(details.thumbnails[2].height, 360);
    }

    #[test]
    fn test_extract_video_details_missing_fields() {
        // Test with partial data to check defaults
        let video_id = "partial_video";

        let player_response = json!({
            "videoDetails": {
                "videoId": video_id,
                "title": "Partial Video",
                // Missing many fields that should use defaults
            }
        });

        // Extract video details
        let result = VideoDetailsExtractor::extract_video_details(&player_response, video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify the extracted data and defaults
        assert_eq!(details.video_id, video_id);
        assert_eq!(details.title, "Partial Video");
        assert_eq!(details.length_seconds, 0); // Default
        assert_eq!(details.author, "Unknown"); // Default
        assert_eq!(details.channel_id, ""); // Default
        assert_eq!(details.short_description, ""); // Default
        assert_eq!(details.view_count, "0"); // Default
        assert!(!details.is_live_content); // Default
        assert!(details.keywords.is_none()); // Default
        assert_eq!(details.thumbnails.len(), 0); // Default empty array
    }

    #[test]
    fn test_extract_video_details_fallback_video_id() {
        // Test that the function uses the provided video_id as fallback
        let actual_video_id = "actual_video_id";
        let fallback_video_id = "fallback_video_id";

        let player_response = json!({
            "videoDetails": {
                // No videoId field, should use fallback
                "title": "Test Video"
            }
        });

        // Extract video details with fallback id
        let result =
            VideoDetailsExtractor::extract_video_details(&player_response, fallback_video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify the video_id falls back to provided argument
        assert_eq!(details.video_id, fallback_video_id);

        // Now test with actual video_id present
        let player_response = json!({
            "videoDetails": {
                "videoId": actual_video_id,
                "title": "Test Video"
            }
        });

        let result =
            VideoDetailsExtractor::extract_video_details(&player_response, fallback_video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify the video_id uses the one from the response
        assert_eq!(details.video_id, actual_video_id);
    }

    #[test]
    fn test_extract_video_details_live_content() {
        // Test with a live video
        let video_id = "live_video";

        let player_response = json!({
            "videoDetails": {
                "videoId": video_id,
                "title": "Live Stream",
                "isLiveContent": true
            }
        });

        // Extract video details
        let result = VideoDetailsExtractor::extract_video_details(&player_response, video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify live content flag
        assert!(details.is_live_content);
    }

    #[test]
    fn test_extract_video_details_missing_section() {
        // Test when videoDetails section is completely missing
        let video_id = "missing_details";

        let player_response = json!({
            "playerConfig": {
                "audioConfig": {
                    "loudnessDb": -18.0,
                    "perceptualLoudnessDb": -14.0
                }
            }
            // No videoDetails section
        });

        // Extract video details
        let result = VideoDetailsExtractor::extract_video_details(&player_response, video_id);

        // Verify it returns an error
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.video_id, video_id);
        assert!(matches!(
            error.reason,
            Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_))
        ));
    }

    #[test]
    fn test_extract_video_details_malformed_thumbnails() {
        // Test with malformed thumbnail data
        let video_id = "malformed_thumbs";

        let player_response = json!({
            "videoDetails": {
                "videoId": video_id,
                "title": "Test Video",
                "thumbnail": {
                    "thumbnails": [
                        {
                            // Missing width/height
                            "url": "https://example.com/thumb.jpg"
                        },
                        {
                            // Missing url
                            "width": 320,
                            "height": 180
                        },
                        {
                            // Corrupt width/height (not numbers)
                            "url": "https://example.com/thumb2.jpg",
                            "width": "invalid",
                            "height": "invalid"
                        },
                        {
                            // This one is valid
                            "url": "https://example.com/thumb3.jpg",
                            "width": 480,
                            "height": 360
                        }
                    ]
                }
            }
        });

        // Extract video details
        let result = VideoDetailsExtractor::extract_video_details(&player_response, video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify only valid thumbnails are kept
        assert_eq!(details.thumbnails.len(), 1);
        assert_eq!(details.thumbnails[0].url, "https://example.com/thumb3.jpg");
        assert_eq!(details.thumbnails[0].width, 480);
        assert_eq!(details.thumbnails[0].height, 360);
    }

    #[test]
    fn test_extract_video_details_malformed_length() {
        // Test with non-parseable length
        let video_id = "bad_length";

        let player_response = json!({
            "videoDetails": {
                "videoId": video_id,
                "title": "Test Video",
                "lengthSeconds": "not_a_number"
            }
        });

        // Extract video details
        let result = VideoDetailsExtractor::extract_video_details(&player_response, video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify default length is used
        assert_eq!(details.length_seconds, 0);
    }

    #[test]
    fn test_extract_video_details_keyword_filtering() {
        // Test keyword extraction with some non-string values
        let video_id = "keyword_test";

        let player_response = json!({
            "videoDetails": {
                "videoId": video_id,
                "title": "Keyword Test",
                "keywords": [
                    "valid",
                    123, // This should be filtered out
                    "another valid",
                    null, // This should be filtered out
                    "final valid"
                ]
            }
        });

        // Extract video details
        let result = VideoDetailsExtractor::extract_video_details(&player_response, video_id);
        assert!(result.is_ok());

        let details = result.unwrap();

        // Verify only valid keywords are kept
        assert!(details.keywords.is_some());
        let keywords = details.keywords.unwrap();
        assert_eq!(keywords.len(), 3);
        assert!(keywords.contains(&"valid".to_string()));
        assert!(keywords.contains(&"another valid".to_string()));
        assert!(keywords.contains(&"final valid".to_string()));
    }
}
