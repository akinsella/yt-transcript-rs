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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_captions_data_success() {
        // Test successful extraction of captions data
        let video_id = "test_video_id";

        // Create a mock player response with captions data
        let mock_renderer = json!({
            "captionTracks": [
                {
                    "baseUrl": "https://example.com/captions",
                    "name": { "simpleText": "English" },
                    "vssId": ".en",
                    "languageCode": "en",
                    "isTranslatable": true
                }
            ]
        });

        let player_response = json!({
            "captions": {
                "playerCaptionsTracklistRenderer": mock_renderer
            }
        });

        // Extract captions data
        let result = CaptionsExtractor::extract_captions_data(&player_response, video_id);

        // Verify the result
        assert!(result.is_ok());
        let extracted_data = result.unwrap();
        assert_eq!(extracted_data, mock_renderer);

        // Verify content of the extracted data
        assert!(extracted_data.get("captionTracks").is_some());
        let tracks = extracted_data["captionTracks"].as_array().unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0]["languageCode"], "en");
    }

    #[test]
    fn test_extract_captions_data_missing_captions() {
        // Test when the player response has no captions field
        let video_id = "test_video_id";
        let player_response = json!({
            "videoDetails": {
                "videoId": "test_video_id",
                "title": "Test Video"
            }
            // No captions field
        });

        // Extract captions data
        let result = CaptionsExtractor::extract_captions_data(&player_response, video_id);

        // Verify it returns an error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.video_id, video_id);
        assert!(matches!(
            error.reason,
            Some(CouldNotRetrieveTranscriptReason::TranscriptsDisabled)
        ));
    }

    #[test]
    fn test_extract_captions_data_missing_renderer() {
        // Test when the captions field exists but has no playerCaptionsTracklistRenderer
        let video_id = "test_video_id";
        let player_response = json!({
            "captions": {
                // No playerCaptionsTracklistRenderer field
                "otherField": "value"
            }
        });

        // Extract captions data
        let result = CaptionsExtractor::extract_captions_data(&player_response, video_id);

        // Verify it returns an error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.video_id, video_id);
        assert!(matches!(
            error.reason,
            Some(CouldNotRetrieveTranscriptReason::TranscriptsDisabled)
        ));
    }

    #[test]
    fn test_extract_captions_data_empty_renderer() {
        // Test when the renderer exists but has no useful data
        let video_id = "test_video_id";
        let player_response = json!({
            "captions": {
                "playerCaptionsTracklistRenderer": {}
            }
        });

        // Extract captions data
        let result = CaptionsExtractor::extract_captions_data(&player_response, video_id);

        // Should succeed but return empty object
        assert!(result.is_ok());
        let extracted_data = result.unwrap();
        assert!(extracted_data.is_object());
        assert_eq!(extracted_data.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_extract_captions_data_complex_structure() {
        // Test with a more complex data structure similar to real YouTube responses
        let video_id = "test_video_id";

        let player_response = json!({
            "captions": {
                "playerCaptionsTracklistRenderer": {
                    "captionTracks": [
                        {
                            "baseUrl": "https://example.com/captions/en",
                            "name": { "simpleText": "English" },
                            "vssId": ".en",
                            "languageCode": "en",
                            "isTranslatable": true
                        },
                        {
                            "baseUrl": "https://example.com/captions/fr",
                            "name": { "simpleText": "French" },
                            "vssId": ".fr",
                            "languageCode": "fr",
                            "isTranslatable": true
                        }
                    ],
                    "audioTracks": [
                        {
                            "captionTrackIndices": [0, 1],
                            "defaultCaptionTrackIndex": 0
                        }
                    ],
                    "translationLanguages": [
                        {
                            "languageCode": "es",
                            "languageName": { "simpleText": "Spanish" }
                        },
                        {
                            "languageCode": "de",
                            "languageName": { "simpleText": "German" }
                        }
                    ]
                }
            }
        });

        // Extract captions data
        let result = CaptionsExtractor::extract_captions_data(&player_response, video_id);

        // Verify the result
        assert!(result.is_ok());
        let extracted_data = result.unwrap();

        // Verify the structure
        assert!(extracted_data.get("captionTracks").is_some());
        let tracks = extracted_data["captionTracks"].as_array().unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0]["languageCode"], "en");
        assert_eq!(tracks[1]["languageCode"], "fr");

        // Verify translation languages
        let translations = extracted_data["translationLanguages"].as_array().unwrap();
        assert_eq!(translations.len(), 2);
        assert_eq!(translations[0]["languageCode"], "es");
        assert_eq!(translations[1]["languageCode"], "de");
    }
}
