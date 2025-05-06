use crate::errors::{
    CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason,
};

/// Responsible for checking if a YouTube video is playable
/// and handling various error conditions (age restriction, unavailable, etc.)
pub struct PlayabilityAsserter;

impl PlayabilityAsserter {
    /// Check if the video is playable and handle various error conditions
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

    /// Extract subreasons from error screen if available
    fn extract_subreasons(player_data: &serde_json::Value) -> Vec<String> {
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
