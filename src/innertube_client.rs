use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use reqwest::Client;
use serde_json::{json, Value};

/// InnerTube API client for fetching YouTube transcript data
///
/// This client uses YouTube's internal InnerTube API instead of the legacy
/// transcript URLs to fetch caption data. This approach is more reliable
/// and doesn't require cookie authentication for public videos.
pub struct InnerTubeClient {
    client: Client,
}

impl InnerTubeClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Fetch transcript data using YouTube's InnerTube API
    pub async fn get_transcript_data(
        &self,
        video_id: &str,
    ) -> Result<Value, CouldNotRetrieveTranscript> {
        let url = "https://www.youtube.com/youtubei/v1/get_transcript";

        let payload = json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20231219.04.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "params": self.encode_transcript_params(video_id)
        });

        let response = self.client
            .post(url)
            .json(&payload)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36")
            .send()
            .await
            .map_err(|e| CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    format!("InnerTube API request failed: {}", e)
                )),
            })?;

        if !response.status().is_success() {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    format!("InnerTube API returned status: {}", response.status()),
                )),
            });
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                    format!("Failed to parse InnerTube response: {}", e),
                )),
            })?;

        Ok(data)
    }

    /// Encode transcript parameters for InnerTube API
    /// This is a simplified version - the actual encoding may be more complex
    fn encode_transcript_params(&self, video_id: &str) -> String {
        // For now, we'll try a simple approach
        // In the real implementation, this would need proper base64 encoding
        // of the video ID and other parameters
        use base64::{engine::general_purpose, Engine as _};

        let params = json!({
            "videoId": video_id
        });

        general_purpose::STANDARD.encode(params.to_string())
    }

    /// Alternative approach: try to get transcript list first
    pub async fn get_transcript_list(
        &self,
        video_id: &str,
    ) -> Result<Value, CouldNotRetrieveTranscript> {
        // Try the player API first to get available captions
        let url = "https://www.youtube.com/youtubei/v1/player";

        let payload = json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20231219.04.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id
        });

        let response = self.client
            .post(url)
            .json(&payload)
            .header("Content-Type", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36")
            .send()
            .await
            .map_err(|e| CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    format!("InnerTube player API request failed: {}", e)
                )),
            })?;

        if !response.status().is_success() {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    format!(
                        "InnerTube player API returned status: {}",
                        response.status()
                    ),
                )),
            });
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                    format!("Failed to parse InnerTube player response: {}", e),
                )),
            })?;

        Ok(data)
    }
}
