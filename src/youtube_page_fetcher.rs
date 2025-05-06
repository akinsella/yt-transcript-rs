/// YouTube page fetching functionality.
///
/// This module provides utilities to fetch YouTube video pages while handling various
/// regional restrictions, proxy configurations, and cookie consent requirements that
/// may be encountered when accessing YouTube content.
///
/// It's primarily used by the transcript fetching components to access the initial
/// YouTube video page, which contains metadata needed for transcript retrieval.
use regex::Regex;
use reqwest::header;
use reqwest::{Client, StatusCode, Url};

use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use crate::proxies::ProxyConfig;

/// The URL template for YouTube video watch pages.
///
/// The `{video_id}` placeholder is replaced with the actual video ID when making requests.
pub const WATCH_URL: &str = "https://www.youtube.com/watch?v={video_id}";

/// Responsible for fetching YouTube video pages and handling special cases
/// like cookie consent, region restrictions, and proxy management.
///
/// # Features
///
/// * Automatically handles YouTube cookie consent forms for regions requiring consent (e.g., EU)
/// * Supports proxy configurations for accessing region-restricted content
/// * Provides detailed error information when requests fail
/// * Uses proper HTTP headers to improve success rates
///
/// # Example
///
/// ```rust,no_run
/// # use reqwest::Client;
/// # use yt_transcript_rs::youtube_page_fetcher::YoutubePageFetcher;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a fetcher with default client and no proxy
/// let client = Client::new();
/// let fetcher = YoutubePageFetcher::new(client, None);
///
/// // Fetch a YouTube video page
/// let video_id = "dQw4w9WgXcQ";
/// let html = fetcher.fetch_video_page(video_id).await?;
///
/// // The HTML can now be parsed for metadata or used for other purposes
/// println!("Retrieved {} bytes of HTML", html.len());
/// # Ok(())
/// # }
/// ```
pub struct YoutubePageFetcher {
    /// HTTP client used for making requests to YouTube
    client: Client,

    /// Optional proxy configuration for accessing region-restricted content
    proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>,
}

impl YoutubePageFetcher {
    /// Creates a new page fetcher with the provided HTTP client and proxy configuration.
    ///
    /// # Parameters
    ///
    /// * `client` - A configured reqwest HTTP client for making requests
    /// * `proxy_config` - Optional proxy configuration for routing requests through a proxy
    ///
    /// # Returns
    ///
    /// A new `YoutubePageFetcher` instance ready to fetch YouTube video pages.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use reqwest::{Client, ClientBuilder};
    /// # use yt_transcript_rs::youtube_page_fetcher::YoutubePageFetcher;
    /// # use yt_transcript_rs::proxies::GenericProxyConfig;
    /// # fn example() {
    /// // Create with default client, no proxy
    /// let client = Client::new();
    /// let basic_fetcher = YoutubePageFetcher::new(client, None);
    ///
    /// // Create with a custom client and proxy configuration
    /// let custom_client = ClientBuilder::new()
    ///     .timeout(std::time::Duration::from_secs(30))
    ///     .build()
    ///     .unwrap();
    ///     
    /// let proxy_config = Box::new(GenericProxyConfig::new(
    ///     Some("http://user:pass@proxy.example.com:8080".to_string()),
    ///     None
    /// ).unwrap());
    ///
    /// let proxy_fetcher = YoutubePageFetcher::new(custom_client, Some(proxy_config));
    /// # }
    /// ```
    pub fn new(client: Client, proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>) -> Self {
        Self {
            client,
            proxy_config,
        }
    }

    /// Fetches the HTML content for a YouTube video page.
    ///
    /// This method:
    /// 1. Makes an initial request to the YouTube video page
    /// 2. Automatically handles cookie consent forms if present
    /// 3. Properly categorizes errors based on status codes and content
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID to fetch (e.g., "dQw4w9WgXcQ")
    ///
    /// # Returns
    ///
    /// * `Result<String, CouldNotRetrieveTranscript>` - The HTML content on success, or a detailed error
    ///
    /// # Errors
    ///
    /// This method returns a `CouldNotRetrieveTranscript` error with specific reasons when:
    /// - The network request fails
    /// - YouTube returns a non-success status code
    /// - The request is blocked by YouTube (HTTP 403)
    /// - Too many requests have been made (HTTP 429)
    /// - Cookie consent handling fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use reqwest::Client;
    /// # use yt_transcript_rs::youtube_page_fetcher::YoutubePageFetcher;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new();
    /// let fetcher = YoutubePageFetcher::new(client, None);
    ///
    /// // Fetch a regular YouTube video
    /// let video_id = "dQw4w9WgXcQ";
    /// match fetcher.fetch_video_page(video_id).await {
    ///     Ok(html) => {
    ///         println!("Successfully fetched video page ({} bytes)", html.len());
    ///         // Process the HTML content...
    ///     },
    ///     Err(e) => {
    ///         println!("Failed to fetch video: {:?}", e.reason);
    ///         // Handle different error types...
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

                if let Some(_status @ (StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS)) =
                    e.status()
                {
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

    /// Creates and sets a consent cookie for YouTube.
    ///
    /// YouTube requires consent cookies in regions with strict privacy laws (like the EU).
    /// This method:
    /// 1. Extracts the consent form value from the YouTube page HTML
    /// 2. Creates a properly formatted consent cookie
    /// 3. Adds the cookie to the client's cookie jar
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content containing the consent form
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<(), CouldNotRetrieveTranscript>` - Success or error with details
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The consent form value cannot be extracted from the HTML
    /// - The request to set the cookie fails
    ///
    /// # Note
    ///
    /// This is an internal method used by `fetch_video_page` and typically should not
    /// be called directly.
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
