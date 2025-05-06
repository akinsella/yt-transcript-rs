use thiserror::Error;
use crate::TranscriptList;
use crate::proxies::{ProxyConfig, GenericProxyConfig, WebshareProxyConfig};

#[derive(Debug, Error)]
pub enum YouTubeTranscriptApiError {
    #[error("YouTube Transcript API error")]
    Generic,
}

#[derive(Debug, Error)]
pub enum CookieError {
    #[error("Cookie error")]
    Generic,
    #[error("Can't load the provided cookie file: {0}")]
    PathInvalid(String),
    #[error("The cookies provided are not valid (may have expired): {0}")]
    Invalid(String),
}

pub type CookiePathInvalid = CookieError;
pub type CookieInvalid = CookieError;

#[derive(Debug, Error)]
#[error("{}", self.build_error_message())]
pub struct CouldNotRetrieveTranscript {
    pub video_id: String,
    pub reason: Option<CouldNotRetrieveTranscriptReason>,
}

#[derive(Debug)]
pub enum CouldNotRetrieveTranscriptReason {
    TranscriptsDisabled,
    NoTranscriptFound {
        requested_language_codes: Vec<String>,
        transcript_data: TranscriptList,
    },
    VideoUnavailable,
    VideoUnplayable {
        reason: Option<String>,
        sub_reasons: Vec<String>,
    },
    IpBlocked(Option<Box<dyn ProxyConfig>>),
    RequestBlocked(Option<Box<dyn ProxyConfig>>),
    NotTranslatable,
    TranslationLanguageNotAvailable,
    FailedToCreateConsentCookie,
    YouTubeRequestFailed(String),
    InvalidVideoId,
    AgeRestricted,
    YouTubeDataUnparsable,
}

impl CouldNotRetrieveTranscript {
    fn build_error_message(&self) -> String {
        let base_error = format!("Could not retrieve a transcript for the video {}!", 
            self.video_id.replace("{video_id}", &self.video_id));
            
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
                                format!("{}\n\nYouTube is blocking your requests, despite you using proxies. Keep in mind a proxy is just a way to hide your real IP behind the IP of that proxy, but there is no guarantee that the IP of that proxy won't be blocked as well.\n\nThe only truly reliable way to prevent IP blocks is rotating through a large pool of residential IPs.", base_cause)
                            },
                            _ => {
                                format!("{}\n\nRequest blocked.", base_cause)
                            }
                        }
                    },
                    CouldNotRetrieveTranscriptReason::NotTranslatable => {
                        "The requested language is not translatable".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::TranslationLanguageNotAvailable => {
                        "The requested translation language is not available".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie => {
                        "Failed to automatically give consent to saving cookies".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(reason) => {
                        format!("Request to YouTube failed: {}", reason)
                    },
                    CouldNotRetrieveTranscriptReason::InvalidVideoId => {
                        "You provided an invalid video id. Make sure you are using the video id and NOT the url!`".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::AgeRestricted => {
                        "This video is age-restricted. Therefore, you will have to authenticate to be able to retrieve transcripts for it. You will have to provide a cookie to authenticate yourself.".to_string()
                    },
                    CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable => {
                        "The data required to fetch the transcript is not parsable. This should not happen, please open an issue (make sure to include the video ID)!".to_string()
                    }
                };
                
                format!("{} This is most likely caused by:\n\n{}", base_error, cause)
            },
            None => base_error,
        }
    }
}

// Type aliases for simpler usage
pub type TranscriptsDisabled = CouldNotRetrieveTranscript;
pub type NoTranscriptFound = CouldNotRetrieveTranscript;
pub type VideoUnavailable = CouldNotRetrieveTranscript;
pub type VideoUnplayable = CouldNotRetrieveTranscript;
pub type IpBlocked = CouldNotRetrieveTranscript;
pub type RequestBlocked = CouldNotRetrieveTranscript;
pub type NotTranslatable = CouldNotRetrieveTranscript;
pub type TranslationLanguageNotAvailable = CouldNotRetrieveTranscript;
pub type FailedToCreateConsentCookie = CouldNotRetrieveTranscript;
pub type YouTubeRequestFailed = CouldNotRetrieveTranscript;
pub type InvalidVideoId = CouldNotRetrieveTranscript;
pub type AgeRestricted = CouldNotRetrieveTranscript;
pub type YouTubeDataUnparsable = CouldNotRetrieveTranscript; 