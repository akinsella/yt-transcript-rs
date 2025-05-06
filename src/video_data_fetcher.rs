use reqwest::Client;

use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use crate::js_var_parser::JsVarParser;
use crate::models::VideoDetails;
use crate::playability_asserter::PlayabilityAsserter;
use crate::proxies::ProxyConfig;
use crate::transcript_list::TranscriptList;
use crate::video_details_extractor::VideoDetailsExtractor;
use crate::youtube_page_fetcher::YoutubePageFetcher;

/// # VideoDataFetcher
///
/// Core component responsible for fetching transcript data and video details from YouTube.
///
/// This struct handles the low-level communication with YouTube's web API to:
/// - Fetch available transcripts for a video
/// - Extract caption JSON data from YouTube pages
/// - Retrieve detailed information about videos, including metadata
///
/// The VideoDataFetcher works by parsing YouTube's HTML and JavaScript variables
/// to extract the necessary data, since YouTube doesn't provide a public API for transcripts.
///
/// ## Internal Architecture
///
/// This component uses several helper classes to process data:
/// - `YoutubePageFetcher`: Handles HTTP requests to YouTube, including proxy support
/// - `JsVarParser`: Extracts JavaScript variables from YouTube's HTML
/// - `PlayabilityAsserter`: Verifies video availability and access permissions
/// - `VideoDetailsExtractor`: Extracts detailed information from video data
pub struct VideoDataFetcher {
    /// HTTP client for making requests
    client: Client,
    /// Specialized fetcher for YouTube pages
    page_fetcher: YoutubePageFetcher,
}

impl VideoDataFetcher {
    /// Creates a new VideoDataFetcher instance.
    ///
    /// # Parameters
    ///
    /// * `client` - A configured reqwest HTTP client to use for requests
    /// * `proxy_config` - Optional proxy configuration for routing requests through a proxy
    ///
    /// # Returns
    ///
    /// A new VideoDataFetcher instance.
    ///
    /// # Example (internal usage)
    ///
    /// ```rust,no_run
    /// # use reqwest::Client;
    /// # use yt_transcript_rs::proxies::GenericProxyConfig;
    /// # use yt_transcript_rs::video_data_fetcher::VideoDataFetcher;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a client
    /// let client = Client::new();
    ///
    /// // Create a proxy configuration (optional)
    /// let proxy_config = GenericProxyConfig::new(
    ///     Some("http://proxy.example.com:8080".to_string()),
    ///     None
    /// )?;
    ///
    /// // Create the fetcher
    /// let fetcher = VideoDataFetcher::new(
    ///     client,
    ///     Some(Box::new(proxy_config))
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(client: Client, proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>) -> Self {
        let page_fetcher = YoutubePageFetcher::new(client.clone(), proxy_config);

        Self {
            client,
            page_fetcher,
        }
    }

    /// Fetches the list of available transcripts for a YouTube video.
    ///
    /// This method:
    /// 1. Retrieves the video page HTML
    /// 2. Extracts the captions JSON data
    /// 3. Builds a TranscriptList from the extracted data
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID (e.g., "dQw4w9WgXcQ")
    ///
    /// # Returns
    ///
    /// * `Result<TranscriptList, CouldNotRetrieveTranscript>` - A TranscriptList on success, or an error if retrieval fails
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The video doesn't exist or is private
    /// - The video has no available transcripts
    /// - YouTube's HTML structure has changed and parsing fails
    /// - Network errors occur during the request
    ///
    /// # Example (internal usage)
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// let video_id = "dQw4w9WgXcQ";
    ///
    /// // This internally calls VideoDataFetcher::fetch_transcript_list
    /// let transcript_list = api.list_transcripts(video_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_transcript_list(
        &self,
        video_id: &str,
    ) -> Result<TranscriptList, CouldNotRetrieveTranscript> {
        let video_captions = self.fetch_video_captions(video_id).await?;

        TranscriptList::build(self.client.clone(), video_id.to_string(), &video_captions)
    }

    /// Fetches the captions JSON data from YouTube.
    ///
    /// This is an internal method that:
    /// 1. Retrieves the HTML for the video page
    /// 2. Extracts the captions JSON data from the page
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID
    ///
    /// # Returns
    ///
    /// * `Result<serde_json::Value, CouldNotRetrieveTranscript>` - The captions JSON data or an error
    async fn fetch_video_captions(
        &self,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        // Fetch the video HTML using the page fetcher
        let html = self.page_fetcher.fetch_video_page(video_id).await?;

        // Extract captions JSON
        self.extract_captions_json(&html, video_id)
    }

    /// Fetches detailed information about a YouTube video.
    ///
    /// This method retrieves comprehensive metadata about a video, including:
    /// - Title, author, channel ID
    /// - View count and video length
    /// - Thumbnails in various resolutions
    /// - Keywords and description
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID
    ///
    /// # Returns
    ///
    /// * `Result<VideoDetails, CouldNotRetrieveTranscript>` - Video details on success, or an error
    ///
    /// # Errors
    ///
    /// Similar to transcript fetching, this can fail if:
    /// - The video doesn't exist or is private
    /// - YouTube's HTML structure has changed and parsing fails
    /// - Network errors occur during the request
    ///
    /// # Example (internal usage)
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// let video_id = "dQw4w9WgXcQ";
    ///
    /// // This internally calls VideoDataFetcher::fetch_video_details
    /// let details = api.fetch_video_details(video_id).await?;
    ///
    /// println!("Video title: {}", details.title);
    /// println!("Author: {}", details.author);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_video_details(
        &self,
        video_id: &str,
    ) -> Result<VideoDetails, CouldNotRetrieveTranscript> {
        // Fetch the HTML and extract player response
        let html = self.page_fetcher.fetch_video_page(video_id).await?;
        let player_response = self.extract_yt_initial_player_response(&html, video_id)?;

        // Extract video details from player response
        VideoDetailsExtractor::extract_video_details(&player_response, video_id)
    }

    /// Extracts the ytInitialPlayerResponse JavaScript variable from YouTube's HTML.
    ///
    /// This variable contains detailed information about the video, including captions.
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content of the YouTube video page
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<serde_json::Value, CouldNotRetrieveTranscript>` - The parsed JavaScript object or an error
    fn extract_yt_initial_player_response(
        &self,
        html: &str,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        let js_var_parser = JsVarParser::new("ytInitialPlayerResponse");
        let player_response = js_var_parser.parse(html, video_id)?;

        Ok(player_response)
    }

    /// Extracts the captions JSON data from YouTube's HTML.
    ///
    /// This method:
    /// 1. Extracts the ytInitialPlayerResponse variable
    /// 2. Verifies the video is playable
    /// 3. Extracts the captions data from the player response
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content of the YouTube video page
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
    /// - The video is unavailable or restricted
    fn extract_captions_json(
        &self,
        html: &str,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        let player_response = self.extract_yt_initial_player_response(html, video_id)?;

        // Check playability status using the PlayabilityAsserter
        PlayabilityAsserter::assert_playability(&player_response, video_id)?;

        // Extract captions from player response
        let captions_json = match player_response.get("captions") {
            Some(captions) => match captions.get("playerCaptionsTracklistRenderer") {
                Some(renderer) => renderer.clone(),
                None => {
                    return Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::TranscriptsDisabled),
                    });
                }
            },
            None => {
                return Err(CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::TranscriptsDisabled),
                });
            }
        };

        Ok(captions_json)
    }
}
