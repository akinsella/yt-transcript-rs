use regex::Regex;
use reqwest::header;
use reqwest::{Client, StatusCode, Url};

use crate::errors::{
    CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason,
};
use crate::proxies::ProxyConfig;

pub const WATCH_URL: &str = "https://www.youtube.com/watch?v={video_id}";

/// Responsible for fetching YouTube video pages and handling special cases
/// like cookie consent, region restrictions, and proxy management
pub struct YoutubePageFetcher {
    client: Client,
    proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>,
}

impl YoutubePageFetcher {
    /// Creates a new page fetcher with the provided HTTP client and proxy configuration
    pub fn new(client: Client, proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>) -> Self {
        Self {
            client,
            proxy_config,
        }
    }

    /// Fetch HTML content for a YouTube video
    pub async fn fetch_video_page(
        &self,
        video_id: &str,
    ) -> Result<String, CouldNotRetrieveTranscript> {
        let url = WATCH_URL.replace("{video_id}", video_id);

        let response = self
            .client
            .get(&url)
            .header("Accept-Language", "en-US")
            .send()
            .await
            .map_err(|e| {
                let mut error = CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                        e.to_string(),
                    )),
                };

                if let Some(status) = e.status() {
                    match status {
                        StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS => {
                            // Create either RequestBlocked or IpBlocked error
                            // We can't clone the proxy_config directly, so we'll just check if it exists
                            error = CouldNotRetrieveTranscript {
                                video_id: video_id.to_string(),
                                reason: if self.proxy_config.is_some() {
                                    Some(CouldNotRetrieveTranscriptReason::RequestBlocked(None))
                                } else {
                                    Some(CouldNotRetrieveTranscriptReason::IpBlocked(None))
                                },
                            };
                        }
                        _ => {}
                    }
                }

                error
            })?;

        if !response.status().is_success() {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    format!("YouTube returned status code: {}", response.status()),
                )),
            });
        }

        let html = response
            .text()
            .await
            .map_err(|e| CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    e.to_string(),
                )),
            })?;

        // Check if we need to handle cookie consent form (for EU/regions with cookie consent laws)
        if html.contains("action=\"https://consent.youtube.com/s\"") {
            // Create and set consent cookie
            self.create_consent_cookie(&html, video_id).await?;

            // Fetch the HTML again with the consent cookie
            let consent_response = self
                .client
                .get(&url)
                .header("Accept-Language", "en-US")
                .send()
                .await
                .map_err(|e| CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                        e.to_string(),
                    )),
                })?;

            let html_with_consent =
                consent_response
                    .text()
                    .await
                    .map_err(|e| CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                            e.to_string(),
                        )),
                    })?;

            // Check if consent form still exists after setting cookie
            if html_with_consent.contains("action=\"https://consent.youtube.com/s\"") {
                return Err(CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie),
                });
            }

            Ok(html_with_consent)
        } else {
            Ok(html)
        }
    }

    /// Create and set a consent cookie for YouTube
    async fn create_consent_cookie(
        &self,
        html: &str,
        video_id: &str,
    ) -> Result<(), CouldNotRetrieveTranscript> {
        // Extract the value parameter from the consent form
        let re = Regex::new(r#"name="v" value="([^"]+)"#).unwrap();

        if let Some(caps) = re.captures(html) {
            if let Some(v_value) = caps.get(1) {
                // Create a cookie with the consent value
                let cookie_value = format!("YES+{}", v_value.as_str());

                // Set the cookie in the client's cookie jar
                // We create a cookie URL for youtube.com domain
                let cookie_url = Url::parse("https://www.youtube.com").unwrap();
                let cookie_str = format!(
                    "CONSENT={}; Domain=.youtube.com; Path=/; Max-Age=31536000",
                    cookie_value
                );

                // Add the cookie to the jar
                self.client
                    .get(cookie_url)
                    .header(header::COOKIE, &cookie_str)
                    .send()
                    .await
                    .map_err(|_| CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie),
                    })?;

                return Ok(());
            }
        }

        // Couldn't find the value parameter
        Err(CouldNotRetrieveTranscript {
            video_id: video_id.to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::FailedToCreateConsentCookie),
        })
    }
}
