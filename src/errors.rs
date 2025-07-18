use crate::proxies::{GenericProxyConfig, ProxyConfig, WebshareProxyConfig};
use crate::TranscriptList;
use thiserror::Error;

/// # YouTubeTranscriptApiError
///
/// The base error type for the library.
///
/// This is mainly used as a generic error type for cases that don't fall into
/// more specific error categories.
#[derive(Debug, Error)]
pub enum YouTubeTranscriptApiError {
    #[error("YouTube Transcript API error")]
    Generic,
}

/// # CookieError
///
/// Errors related to cookie handling and authentication.
///
/// These errors occur when there are issues with loading or using cookies
/// for authenticated requests to YouTube.
#[derive(Debug, Error)]
pub enum CookieError {
    #[error("Cookie error")]
    Generic,

    /// Error when the specified cookie file path is invalid or inaccessible
    #[error("Can't load the provided cookie file: {0}")]
    PathInvalid(String),

    /// Error when the cookies are invalid (possibly expired or malformed)
    #[error("The cookies provided are not valid (may have expired): {0}")]
    Invalid(String),
}

/// Type alias for cookie path invalid errors
pub type CookiePathInvalid = CookieError;

/// Type alias for invalid cookie errors
pub type CookieInvalid = CookieError;

/// # CouldNotRetrieveTranscript
///
/// The primary error type when transcript retrieval fails.
///
/// This error provides detailed information about why a transcript couldn't be retrieved,
/// with specific reasons and helpful suggestions for resolving the issue.
///
/// ## Usage Example
///
/// ```rust,no_run
/// # use yt_transcript_rs::YouTubeTranscriptApi;
/// # async fn example() {
/// let api = YouTubeTranscriptApi::new(None, None, None).unwrap();
///
/// match api.fetch_transcript("dQw4w9WgXcQ", &["en"], false).await {
///     Ok(transcript) => {
///         println!("Successfully retrieved transcript");
///     },
///     Err(err) => {
///         // The error message contains detailed information about what went wrong
///         eprintln!("Error: {}", err);
///         
///         // You can also check for specific error types
///         if err.reason.is_some() {
///             match err.reason.as_ref().unwrap() {
///                 // Handle specific error cases
///                 _ => {}
///             }
///         }
///     }
/// }
/// # }
/// ```
#[derive(Debug, Error)]
#[error("{}", self.build_error_message())]
pub struct CouldNotRetrieveTranscript {
    /// The YouTube video ID that was being accessed
    pub video_id: String,

    /// The specific reason why the transcript couldn't be retrieved
    pub reason: Option<CouldNotRetrieveTranscriptReason>,
}

/// # CouldNotRetrieveTranscriptReason
///
/// Detailed reasons why a transcript couldn't be retrieved.
///
/// This enum provides specific information about why transcript retrieval failed,
/// which is useful for error handling and providing helpful feedback to users.
#[derive(Debug)]
pub enum CouldNotRetrieveTranscriptReason {
    /// Subtitles/transcripts are disabled for this video
    TranscriptsDisabled,

    /// No transcript was found in any of the requested languages
    NoTranscriptFound {
        /// The language codes that were requested but not found
        requested_language_codes: Vec<String>,

        /// Information about available transcripts that could be used instead
        transcript_data: TranscriptList,
    },

    /// The video is no longer available (removed, private, etc.)
    VideoUnavailable,

    /// The video cannot be played for some reason
    VideoUnplayable {
        /// The main reason why the video is unplayable
        reason: Option<String>,

        /// Additional details about why the video is unplayable
        sub_reasons: Vec<String>,
    },

    /// YouTube is blocking requests from your IP address
    IpBlocked(Option<Box<dyn ProxyConfig>>),

    /// YouTube is blocking your request (rate limiting, etc.)
    RequestBlocked(Option<Box<dyn ProxyConfig>>),

    /// The requested transcript cannot be translated with specific error details
    TranslationUnavailable(String),

    /// The requested translation language is not available with specific error details
    TranslationLanguageUnavailable(String),

    /// Failed to create a consent cookie required by YouTube
    FailedToCreateConsentCookie,

    /// The request to YouTube failed with a specific error
    YouTubeRequestFailed(String),

    /// The provided video ID is invalid
    InvalidVideoId,

    /// The video is age-restricted and requires authentication
    AgeRestricted,

    /// The YouTube data structure couldn't be parsed
    YouTubeDataUnparsable(String),
}

impl CouldNotRetrieveTranscript {
    /// Builds a detailed error message based on the error reason
    fn build_error_message(&self) -> String {
        let base_error = format!(
            "Could not retrieve a transcript for the video {}!",
            self.video_id.replace("{video_id}", &self.video_id)
        );

        match &self.reason {
            Some(reason) => {
                let cause = match reason {
                    CouldNotRetrieveTranscriptReason::TranscriptsDisabled => {
                        "Subtitles are disabled for this video".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::NoTranscriptFound { requested_language_codes, transcript_data } => {
                        format!("No transcripts were found for any of the requested language codes: {:?}\n\n{}", 
                            requested_language_codes, transcript_data)
                    },
                    CouldNotRetrieveTranscriptReason::VideoUnavailable => {
                        "The video is no longer available".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::VideoUnplayable { reason, sub_reasons } => {
                        let reason_str = reason.clone().unwrap_or_else(|| "No reason specified!".to_string());
                        let mut message = format!("The video is unplayable for the following reason: {}", reason_str);
                        if !sub_reasons.is_empty() {
                            message.push_str("\n\nAdditional Details:\n");
                            for sub_reason in sub_reasons {
                                message.push_str(&format!(" - {}\n", sub_reason));
                            }
                        }
                        message
                    },
                    CouldNotRetrieveTranscriptReason::IpBlocked(proxy_config) => {
                        let base_cause = "YouTube is blocking requests from your IP. This usually is due to one of the following reasons:
- You have done too many requests and your IP has been blocked by YouTube
- You are doing requests from an IP belonging to a cloud provider (like AWS, Google Cloud Platform, Azure, etc.). Unfortunately, most IPs from cloud providers are blocked by YouTube.";
                        match proxy_config {
                            Some(config) if config.as_any().is::<WebshareProxyConfig>() => {
                                format!("{}\n\nYouTube is blocking your requests, despite you using Webshare proxies. Please make sure that you have purchased \"Residential\" proxies and NOT \"Proxy Server\" or \"Static Residential\", as those won't work as reliably! The free tier also uses \"Proxy Server\" and will NOT work!\n\nThe only reliable option is using \"Residential\" proxies (not \"Static Residential\"), as this allows you to rotate through a pool of over 30M IPs, which means you will always find an IP that hasn't been blocked by YouTube yet!", base_cause)
                            },
                            Some(config) if config.as_any().is::<GenericProxyConfig>() => {
                                format!("{}\n\nYouTube is blocking your requests, despite you using proxies. Keep in mind a proxy is just a way to hide your real IP behind the IP of that proxy, but there is no guarantee that the IP of that proxy won't be blocked as well.\n\nThe only truly reliable way to prevent IP blocks is rotating through a large pool of residential IPs, by using a provider like Webshare.", base_cause)
                            },
                            _ => {
                                format!("{}\n\nIp blocked.", base_cause)
                            }
                        }
                    },
                    CouldNotRetrieveTranscriptReason::RequestBlocked(proxy_config) => {
                        let base_cause = "YouTube is blocking requests from your IP. This usually is due to one of the following reasons:
- You have done too many requests and your IP has been blocked by YouTube
- You are doing requests from an IP belonging to a cloud provider (like AWS, Google Cloud Platform, Azure, etc.). Unfortunately, most IPs from cloud providers are blocked by YouTube.";
                        match proxy_config {
                            Some(config) if config.as_any().is::<WebshareProxyConfig>() => {
                                format!("{}\n\nYouTube is blocking your requests, despite you using Webshare proxies. Please make sure that you have purchased \"Residential\" proxies and NOT \"Proxy Server\" or \"Static Residential\", as those won't work as reliably! The free tier also uses \"Proxy Server\" and will NOT work!\n\nThe only reliable option is using \"Residential\" proxies (not \"Static Residential\"), as this allows you to rotate through a pool of over 30M IPs, which means you will always find an IP that hasn't been blocked by YouTube yet!", base_cause)
                            },
                            Some(config) if config.as_any().is::<GenericProxyConfig>() => {
                                format!("{}\n\nYouTube is blocking your requests, despite you using proxies. Keep in mind a proxy is just a way to hide your real IP behind the IP of that proxy, but there is no guarantee that the IP of that proxy won't be blocked as well.\n\nThe only truly reliable way to prevent IP blocks is rotating through a large pool of residential IPs, by using a provider like Webshare.", base_cause)
                            },
                            _ => {
                                format!("{}\n\nRequest blocked.", base_cause)
                            }
                        }
                    },
                    CouldNotRetrieveTranscriptReason::TranslationUnavailable(message) => {
                        format!("The requested transcript cannot be translated: {}", message)
                    },
                    CouldNotRetrieveTranscriptReason::TranslationLanguageUnavailable(message) => {
                        format!("The requested translation language is not available: {}", message)
                    },
                    CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie => {
                        "Failed to automatically give consent to saving cookies".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(error) => {
                        format!("Failed to make a request to YouTube. Error: {}", error)
                    },
                    CouldNotRetrieveTranscriptReason::InvalidVideoId => {
                        "You provided an invalid video id. Make sure you are using the video id and NOT the url!`".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::AgeRestricted => {
                        "This video is age-restricted. Therefore, you will have to authenticate to be able to retrieve transcripts for it. You will have to provide a cookie to authenticate yourself.".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(details) => {
                        format!("The data required to fetch the transcript is not parsable: {}. This should not happen, please open an issue (make sure to include the video ID)!", details)
                    },
                };

                format!("{} This is most likely caused by:\n\n{}", base_error, cause)
            }
            None => base_error,
        }
    }
}

/// Type alias for when transcripts are disabled for a video
pub type TranscriptsDisabled = CouldNotRetrieveTranscript;

/// Type alias for when no transcript is found in the requested languages
pub type NoTranscriptFound = CouldNotRetrieveTranscript;

/// Type alias for when the video is no longer available
pub type VideoUnavailable = CouldNotRetrieveTranscript;

/// Type alias for when the video cannot be played
pub type VideoUnplayable = CouldNotRetrieveTranscript;

/// Type alias for when YouTube is blocking your IP address
pub type IpBlocked = CouldNotRetrieveTranscript;

/// Type alias for when YouTube is blocking your request
pub type RequestBlocked = CouldNotRetrieveTranscript;

/// Type alias for when the requested transcript cannot be translated
pub type NotTranslatable = CouldNotRetrieveTranscript;

/// Type alias for when the requested translation language is not available
pub type TranslationLanguageNotAvailable = CouldNotRetrieveTranscript;

/// Type alias for when creating a consent cookie fails
pub type FailedToCreateConsentCookie = CouldNotRetrieveTranscript;

/// Type alias for when a request to YouTube fails
pub type YouTubeRequestFailed = CouldNotRetrieveTranscript;

/// Type alias for when an invalid video ID is provided
pub type InvalidVideoId = CouldNotRetrieveTranscript;

/// Type alias for when the video is age-restricted and requires authentication
pub type AgeRestricted = CouldNotRetrieveTranscript;

/// Type alias for when YouTube data cannot be parsed
pub type YouTubeDataUnparsable = CouldNotRetrieveTranscript;

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::collections::HashMap;

    // Mock implementation for testing
    #[derive(Debug)]
    struct MockProxy;

    impl ProxyConfig for MockProxy {
        fn to_requests_dict(&self) -> HashMap<String, String> {
            HashMap::new()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_build_error_message_no_reason() {
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: None,
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("dQw4w9WgXcQ"));
        // Should be a simple message without additional cause
        assert!(!message.contains("This is most likely caused by"));
    }

    #[test]
    fn test_build_error_message_transcripts_disabled() {
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::TranscriptsDisabled),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("Subtitles are disabled"));
    }

    #[test]
    fn test_build_error_message_no_transcript_found() {
        let transcript_list = TranscriptList {
            video_id: "dQw4w9WgXcQ".to_string(),
            manually_created_transcripts: HashMap::new(),
            generated_transcripts: HashMap::new(),
            translation_languages: vec![],
        };

        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::NoTranscriptFound {
                requested_language_codes: vec!["fr".to_string(), "es".to_string()],
                transcript_data: transcript_list,
            }),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("No transcripts were found"));
        assert!(message.contains("fr"));
        assert!(message.contains("es"));
    }

    #[test]
    fn test_build_error_message_video_unavailable() {
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("video is no longer available"));
    }

    #[test]
    fn test_build_error_message_video_unplayable() {
        // Test with reason and sub-reasons
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::VideoUnplayable {
                reason: Some("Content is private".to_string()),
                sub_reasons: vec![
                    "The owner has made this content private".to_string(),
                    "You need permission to access".to_string(),
                ],
            }),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("video is unplayable"));
        assert!(message.contains("Content is private"));
        assert!(message.contains("The owner has made this content private"));
        assert!(message.contains("You need permission to access"));

        // Test with no reason (just sub-reasons)
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::VideoUnplayable {
                reason: None,
                sub_reasons: vec!["Region restricted".to_string()],
            }),
        };

        let message = error.build_error_message();
        assert!(message.contains("No reason specified"));
        assert!(message.contains("Region restricted"));

        // Test with reason but no sub-reasons
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::VideoUnplayable {
                reason: Some("Premium content".to_string()),
                sub_reasons: vec![],
            }),
        };

        let message = error.build_error_message();
        assert!(message.contains("Premium content"));
        assert!(!message.contains("Additional Details"));
    }

    #[test]
    fn test_build_error_message_ip_blocked() {
        // Without proxy config
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::IpBlocked(None)),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("YouTube is blocking requests from your IP"));
        assert!(message.contains("Ip blocked"));

        // With MockProxy
        let mock_proxy = Box::new(MockProxy);
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::IpBlocked(Some(
                mock_proxy,
            ))),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("YouTube is blocking requests from your IP"));
    }

    #[test]
    fn test_build_error_message_request_blocked() {
        // Without proxy config
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::RequestBlocked(None)),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("YouTube is blocking requests from your IP"));
        assert!(message.contains("Request blocked"));

        // With MockProxy
        let mock_proxy = Box::new(MockProxy);
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::RequestBlocked(Some(
                mock_proxy,
            ))),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("YouTube is blocking requests from your IP"));
    }

    #[test]
    fn test_build_error_message_translation_errors() {
        // TranslationUnavailable
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::TranslationUnavailable(
                "Manual transcripts cannot be translated".to_string(),
            )),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("transcript cannot be translated"));
        assert!(message.contains("Manual transcripts cannot be translated"));

        // TranslationLanguageUnavailable
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(
                CouldNotRetrieveTranscriptReason::TranslationLanguageUnavailable(
                    "Klingon is not supported".to_string(),
                ),
            ),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("translation language is not available"));
        assert!(message.contains("Klingon is not supported"));
    }

    #[test]
    fn test_build_error_message_misc_errors() {
        // FailedToCreateConsentCookie
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("Failed to automatically give consent"));

        // YouTubeRequestFailed
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                "Connection timed out".to_string(),
            )),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("Failed to make a request to YouTube"));
        assert!(message.contains("Connection timed out"));

        // InvalidVideoId
        let error = CouldNotRetrieveTranscript {
            video_id: "invalid".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::InvalidVideoId),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("invalid video id"));

        // AgeRestricted
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::AgeRestricted),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("age-restricted"));
        assert!(message.contains("authenticate"));

        // YouTubeDataUnparsable
        let error = CouldNotRetrieveTranscript {
            video_id: "dQw4w9WgXcQ".to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(
                "Invalid XML format".to_string(),
            )),
        };

        let message = error.build_error_message();
        assert!(message.contains("Could not retrieve a transcript"));
        assert!(message.contains("not parsable"));
        assert!(message.contains("open an issue"));
    }
}
