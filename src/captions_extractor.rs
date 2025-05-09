use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};

/// # CaptionsExtractor
///
/// Extracts captions/transcript data from YouTube's player response JSON.
///
/// This utility struct provides functionality to parse YouTube's captions data
/// and extract detailed information about available transcripts.
pub struct CaptionsExtractor;

impl CaptionsExtractor {
    /// Extracts captions data from the player response JSON.
    ///
    /// # Parameters
    ///
    /// * `player_response` - The parsed YouTube player response JSON object
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<serde_json::Value, CouldNotRetrieveTranscript>` - The captions JSON data or an error
    ///
    /// # Errors
    ///
    /// This method will return a specific error if:
    /// - Transcripts are disabled for the video
    /// - The captions data is missing or in an unexpected format
    pub fn extract_captions_data(
        player_response: &serde_json::Value,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        // Extract captions from player response
        match player_response.get("captions") {
            Some(captions) => match captions.get("playerCaptionsTracklistRenderer") {
                Some(renderer) => Ok(renderer.clone()),
                None => Err(CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::TranscriptsDisabled),
                }),
            },
            None => Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::TranscriptsDisabled),
            }),
        }
    }
}
