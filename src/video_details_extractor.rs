use crate::errors::{
    CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason,
};
use crate::models::{VideoDetails, VideoThumbnail};

/// Extracts video details from YouTube's player response
pub struct VideoDetailsExtractor;

impl VideoDetailsExtractor {
    /// Extract video details from player response
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
                let keywords = match details.get("keywords").and_then(|k| k.as_array()) {
                    Some(kw_array) => Some(
                        kw_array
                            .iter()
                            .filter_map(|k| k.as_str().map(|s| s.to_string()))
                            .collect(),
                    ),
                    None => None,
                };

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
