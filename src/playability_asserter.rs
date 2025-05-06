/// YouTube video playability status checking.
///
/// This module provides functionality to check if a YouTube video is playable
/// and handles various error conditions such as age restrictions, unavailable videos,
/// region restrictions, and more.
///
/// YouTube returns a playability status for each video which determines whether
/// a video can be viewed. This module parses that status and converts it into
/// appropriate error types that callers can handle.
///
/// Playability status is essential for the transcript API to determine whether
/// it should attempt to fetch transcripts, as many error conditions that affect
/// video playability also affect transcript availability.
use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};

/// Responsible for checking if a YouTube video is playable
/// and handling various error conditions (age restriction, unavailable, etc.)
///
/// The `PlayabilityAsserter` examines the player data returned by YouTube
/// and determines if the video can be played. If not, it provides detailed
/// error information about why the video cannot be played.
///
/// # Features
///
/// * Detects age-restricted videos
/// * Identifies unavailable videos (removed, private, etc.)
/// * Extracts detailed error messages from YouTube's response
/// * Converts YouTube playability status to library-specific error types
///
/// # Example
///
/// ```rust,no_run
/// # use yt_transcript_rs::playability_asserter::PlayabilityAsserter;
/// # use serde_json::json;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // JSON data from YouTube player response
/// let player_data = json!({
///     "playabilityStatus": {
///         "status": "OK"
///     }
/// });
///
/// // Check if the video is playable
/// let video_id = "dQw4w9WgXcQ";
/// match PlayabilityAsserter::assert_playability(&player_data, video_id) {
///     Ok(()) => {
///         println!("Video is playable, can fetch transcripts");
///         // Proceed with transcript fetching...
///     },
///     Err(e) => {
///         println!("Video is not playable: {:?}", e.reason);
///         // Handle the error appropriately...
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct PlayabilityAsserter;

impl PlayabilityAsserter {
    /// Checks if a video is playable and handles various error conditions.
    ///
    /// This method examines the playability status in the YouTube player data
    /// and returns an appropriate error if the video cannot be played.
    ///
    /// # Parameters
    ///
    /// * `player_data` - JSON data from YouTube's player response
    /// * `video_id` - The YouTube video ID being checked
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the video is playable
    /// * `Err(CouldNotRetrieveTranscript)` - With a specific reason if the video is not playable
    ///
    /// # Error Conditions
    ///
    /// This method returns different error types based on the playability status:
    ///
    /// * `AgeRestricted` - The video is age-restricted and requires login
    /// * `VideoUnavailable` - The video doesn't exist or has been removed
    /// * `VideoUnplayable` - Other reasons with detailed information from YouTube
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::playability_asserter::PlayabilityAsserter;
    /// # use yt_transcript_rs::errors::CouldNotRetrieveTranscriptReason;
    /// # use serde_json::json;
    /// # fn example() {
    /// // Example of an age-restricted video
    /// let age_restricted_data = json!({
    ///     "playabilityStatus": {
    ///         "status": "LOGIN_REQUIRED",
    ///         "reason": "This video may be inappropriate for some users. Sign in to confirm your age."
    ///     }
    /// });
    ///
    /// let result = PlayabilityAsserter::assert_playability(&age_restricted_data, "restricted_video_id");
    /// assert!(matches!(result,
    ///     Err(e) if matches!(e.reason, Some(CouldNotRetrieveTranscriptReason::AgeRestricted))
    /// ));
    ///
    /// // Example of a normal playable video
    /// let playable_data = json!({
    ///     "playabilityStatus": {
    ///         "status": "OK"
    ///     }
    /// });
    ///
    /// let result = PlayabilityAsserter::assert_playability(&playable_data, "playable_video_id");
    /// assert!(result.is_ok());
    /// # }
    /// ```
    pub fn assert_playability(
        player_data: &serde_json::Value,
        video_id: &str,
    ) -> Result<(), CouldNotRetrieveTranscript> {
        let status = player_data
            .get("playabilityStatus")
            .and_then(|s| s.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("ERROR");

        match status {
            "OK" => Ok(()),
            "LOGIN_REQUIRED" => {
                let reason = player_data
                    .get("playabilityStatus")
                    .and_then(|s| s.get("reason"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("");

                if reason.contains("age") {
                    Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::AgeRestricted),
                    })
                } else {
                    let mut sub_reasons = Vec::new();

                    if let Some(messages) = player_data
                        .get("playabilityStatus")
                        .and_then(|s| s.get("errorScreen"))
                        .and_then(|s| s.get("playerErrorMessageRenderer"))
                        .and_then(|s| s.get("subreason"))
                        .and_then(|s| s.get("runs"))
                        .and_then(|s| s.as_array())
                    {
                        for msg in messages {
                            if let Some(text) = msg.get("text").and_then(|t| t.as_str()) {
                                sub_reasons.push(text.to_string());
                            }
                        }
                    }

                    Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::VideoUnplayable {
                            reason: Some(reason.to_string()),
                            sub_reasons,
                        }),
                    })
                }
            }
            "ERROR" | _ => {
                let reason = player_data
                    .get("playabilityStatus")
                    .and_then(|s| s.get("reason"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("");

                if reason.contains("Video unavailable") {
                    Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
                    })
                } else {
                    let mut sub_reasons = Vec::new();

                    if let Some(messages) = player_data
                        .get("playabilityStatus")
                        .and_then(|s| s.get("errorScreen"))
                        .and_then(|s| s.get("playerErrorMessageRenderer"))
                        .and_then(|s| s.get("subreason"))
                        .and_then(|s| s.get("runs"))
                        .and_then(|s| s.as_array())
                    {
                        for msg in messages {
                            if let Some(text) = msg.get("text").and_then(|t| t.as_str()) {
                                sub_reasons.push(text.to_string());
                            }
                        }
                    }

                    Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::VideoUnplayable {
                            reason: Some(reason.to_string()),
                            sub_reasons,
                        }),
                    })
                }
            }
        }
    }

    /// Extracts detailed error messages from YouTube's player data.
    ///
    /// This helper method parses the nested JSON structure containing YouTube's
    /// error messages to provide more detailed information about why a video
    /// is unplayable.
    ///
    /// # Parameters
    ///
    /// * `player_data` - JSON data from YouTube's player response
    ///
    /// # Returns
    ///
    /// A vector of strings containing detailed error messages from YouTube.
    /// Returns an empty vector if no detailed messages are found.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::playability_asserter::PlayabilityAsserter;
    /// # use serde_json::json;
    /// # fn example() {
    /// let player_data = json!({
    ///     "playabilityStatus": {
    ///         "status": "ERROR",
    ///         "reason": "Video unavailable",
    ///         "errorScreen": {
    ///             "playerErrorMessageRenderer": {
    ///                 "subreason": {
    ///                     "runs": [
    ///                         {"text": "This video has been removed by the uploader"},
    ///                         {"text": "Contact the creator for more information"}
    ///                     ]
    ///                 }
    ///             }
    ///         }
    ///     }
    /// });
    ///
    /// let reasons = PlayabilityAsserter::extract_subreasons(&player_data);
    /// assert_eq!(reasons.len(), 2);
    /// assert_eq!(reasons[0], "This video has been removed by the uploader");
    /// # }
    /// ```
    pub fn extract_subreasons(player_data: &serde_json::Value) -> Vec<String> {
        let mut sub_reasons = Vec::new();

        if let Some(messages) = player_data
            .get("playabilityStatus")
            .and_then(|s| s.get("errorScreen"))
            .and_then(|s| s.get("playerErrorMessageRenderer"))
            .and_then(|s| s.get("subreason"))
            .and_then(|s| s.get("runs"))
            .and_then(|s| s.as_array())
        {
            for msg in messages {
                if let Some(text) = msg.get("text").and_then(|t| t.as_str()) {
                    sub_reasons.push(text.to_string());
                }
            }
        }

        sub_reasons
    }
}
