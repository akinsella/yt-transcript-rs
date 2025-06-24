use reqwest::Client;
use std::path::Path;
use std::sync::Arc;

use crate::cookie_jar_loader::CookieJarLoader;
#[cfg(not(feature = "ci"))]
use crate::errors::{CookieError, CouldNotRetrieveTranscript};
#[cfg(feature = "ci")]
use crate::errors::{CookieError, CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use crate::models::{MicroformatData, StreamingData, VideoDetails, VideoInfos};
use crate::proxies::ProxyConfig;
#[cfg(not(feature = "ci"))]
use crate::video_data_fetcher::VideoDataFetcher;
use crate::{FetchedTranscript, TranscriptList};

/// # YouTubeTranscriptApi
///
/// The main interface for retrieving YouTube video transcripts and metadata.
///
/// This API provides methods to:
/// - Fetch transcripts from YouTube videos in various languages
/// - List all available transcript languages for a video
/// - Retrieve detailed video metadata
///
/// The API supports advanced features like:
/// - Custom HTTP clients and proxies for handling geo-restrictions
/// - Cookie management for accessing restricted content
/// - Preserving text formatting in transcripts
///
/// ## Simple Usage Example
///
/// ```rust,no_run
/// use yt_transcript_rs::api::YouTubeTranscriptApi;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a new API instance with default settings
///     let api = YouTubeTranscriptApi::new(None, None, None)?;
///     
///     // Fetch an English transcript
///     let transcript = api.fetch_transcript(
///         "dQw4w9WgXcQ",      // Video ID
///         &["en"],            // Preferred languages
///         false               // Don't preserve formatting
///     ).await?;
///     
///     // Print each snippet of the transcript
///     for snippet in transcript.parts() {
///         println!("[{:.1}s]: {}", snippet.start, snippet.text);
///     }
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct YouTubeTranscriptApi {
    /// The internal data fetcher used to retrieve information from YouTube
    #[cfg(not(feature = "ci"))]
    fetcher: Arc<VideoDataFetcher>,
    #[cfg(feature = "ci")]
    client: Client,
}

impl YouTubeTranscriptApi {
    /// Creates a new YouTube Transcript API instance.
    ///
    /// This method initializes an API instance with optional customizations for
    /// cookies, proxies, and HTTP client settings.
    ///
    /// # Parameters
    ///
    /// * `cookie_path` - Optional path to a Netscape-format cookie file for authenticated requests
    /// * `proxy_config` - Optional proxy configuration for routing requests through a proxy service
    /// * `http_client` - Optional pre-configured HTTP client to use instead of the default one
    ///
    /// # Returns
    ///
    /// * `Result<Self, CookieError>` - A new API instance or a cookie-related error
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cookie file exists but cannot be read or parsed
    /// - The cookie file is not in the expected Netscape format
    ///
    /// # Examples
    ///
    /// ## Basic usage with default settings
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Using a cookie file for authenticated access
    ///
    /// ```rust,no_run
    /// # use std::path::Path;
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cookie_path = Path::new("path/to/cookies.txt");
    /// let api = YouTubeTranscriptApi::new(Some(&cookie_path), None, None)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Using a proxy to bypass geographical restrictions
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # use yt_transcript_rs::proxies::GenericProxyConfig;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a proxy configuration
    /// let proxy = GenericProxyConfig::new(
    ///     Some("http://proxy.example.com:8080".to_string()),
    ///     None
    /// )?;
    ///
    /// let api = YouTubeTranscriptApi::new(
    ///     None,
    ///     Some(Box::new(proxy)),
    ///     None
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        cookie_path: Option<&Path>,
        proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>,
        http_client: Option<Client>,
    ) -> Result<Self, CookieError> {
        let client = match http_client {
            Some(client) => client,
            None => {
                let mut builder = Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36")
                    .default_headers({
                        let mut headers = reqwest::header::HeaderMap::new();
                        headers.insert(
                            reqwest::header::ACCEPT_LANGUAGE,
                            reqwest::header::HeaderValue::from_static("en-US"),
                        );
                        headers
                    });

                // Add cookie jar if needed
                if let Some(cookie_path) = cookie_path {
                    let cookie_jar = CookieJarLoader::load_cookie_jar(cookie_path)?;
                    let cookie_jar = Arc::new(cookie_jar);
                    builder = builder.cookie_store(true).cookie_provider(cookie_jar);
                }

                // Add proxy configuration if needed
                if let Some(proxy_config_ref) = &proxy_config {
                    // Convert the proxy configuration to a map first to avoid borrowing issues
                    let proxy_map = proxy_config_ref.to_requests_dict();

                    let proxies = reqwest::Proxy::custom(move |url| {
                        if url.scheme() == "http" {
                            if let Some(http_proxy) = proxy_map.get("http") {
                                return Some(http_proxy.clone());
                            }
                        } else if url.scheme() == "https" {
                            if let Some(https_proxy) = proxy_map.get("https") {
                                return Some(https_proxy.clone());
                            }
                        }

                        None
                    });

                    builder = builder.proxy(proxies);

                    // Disable keep-alive if needed
                    if proxy_config_ref.prevent_keeping_connections_alive() {
                        builder = builder.connection_verbose(true).tcp_keepalive(None);

                        let mut headers = reqwest::header::HeaderMap::new();
                        headers.insert(
                            reqwest::header::CONNECTION,
                            reqwest::header::HeaderValue::from_static("close"),
                        );
                        builder = builder.default_headers(headers);
                    }
                }

                builder.build().unwrap()
            }
        };

        #[cfg(not(feature = "ci"))]
        let fetcher = Arc::new(VideoDataFetcher::new(client.clone()));

        Ok(Self {
            #[cfg(not(feature = "ci"))]
            fetcher,
            #[cfg(feature = "ci")]
            client,
        })
    }

    /// Fetches a transcript for a YouTube video in the specified languages.
    ///
    /// This method attempts to retrieve a transcript in the first available language
    /// from the provided list of language preferences. If none of the specified languages
    /// are available, an error is returned.
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID (e.g., "dQw4w9WgXcQ" from https://www.youtube.com/watch?v=dQw4w9WgXcQ)
    /// * `languages` - A list of language codes in order of preference (e.g., ["en", "es", "fr"])
    /// * `preserve_formatting` - Whether to preserve HTML formatting in the transcript text
    ///
    /// # Returns
    ///
    /// * `Result<FetchedTranscript, CouldNotRetrieveTranscript>` - The transcript or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The video does not exist or is private
    /// - The video has no transcripts available
    /// - None of the requested languages are available
    /// - Network issues prevent fetching the transcript
    ///
    /// # Examples
    ///
    /// ## Basic usage - get English transcript
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    ///
    /// // Fetch English transcript
    /// let transcript = api.fetch_transcript(
    ///     "dQw4w9WgXcQ",  // Video ID
    ///     &["en"],        // Try English
    ///     false           // Don't preserve formatting
    /// ).await?;
    ///
    /// println!("Full transcript text: {}", transcript.text());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Multiple language preferences with formatting preserved
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    ///
    /// // Try English first, then Spanish, then auto-generated English
    /// let transcript = api.fetch_transcript(
    ///     "dQw4w9WgXcQ",
    ///     &["en", "es", "en-US"],
    ///     true  // Preserve formatting like <b>bold</b> text
    /// ).await?;
    ///
    /// // Print each segment with timing information
    /// for snippet in transcript.parts() {
    ///     println!("[{:.1}s-{:.1}s]: {}",
    ///         snippet.start,
    ///         snippet.start + snippet.duration,
    ///         snippet.text);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ci")]
    pub async fn fetch_transcript(
        &self,
        video_id: &str,
        languages: &[&str],
        _preserve_formatting: bool,
    ) -> Result<FetchedTranscript, CouldNotRetrieveTranscript> {
        if video_id == crate::tests::test_utils::NON_EXISTENT_VIDEO_ID {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
            });
        }

        let transcript =
            crate::tests::mocks::create_mock_fetched_transcript(video_id, languages[0]);
        Ok(transcript)
    }

    #[cfg(not(feature = "ci"))]
    pub async fn fetch_transcript(
        &self,
        video_id: &str,
        languages: &[&str],
        preserve_formatting: bool,
    ) -> Result<FetchedTranscript, CouldNotRetrieveTranscript> {
        // First list all available transcripts
        let transcript_list = self.list_transcripts(video_id).await?;

        // Then find the best matching transcript based on language preferences
        let transcript = transcript_list.find_transcript(languages)?;

        // Use the client from the fetcher
        let client = &self.fetcher.client;

        // Finally fetch the actual transcript content
        transcript.fetch(client, preserve_formatting).await
    }

    /// Lists all available transcripts for a YouTube video.
    ///
    /// This method retrieves a list of all available transcripts/captions for a video,
    /// categorized by:
    /// - Language
    /// - Whether they were manually created or automatically generated
    /// - Whether they support translation to other languages
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID (e.g., "dQw4w9WgXcQ")
    ///
    /// # Returns
    ///
    /// * `Result<TranscriptList, CouldNotRetrieveTranscript>` - A list of available transcripts or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The video ID is invalid
    /// - The video doesn't exist or is private
    /// - YouTube is blocking your requests
    /// - Network errors occur
    /// - No transcripts are available for the video
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// let video_id = "dQw4w9WgXcQ";
    ///
    /// let transcript_list = api.list_transcripts(video_id).await?;
    ///
    /// // Print available transcripts
    /// println!("{}", transcript_list);
    ///
    /// // Count manually created vs. generated transcripts
    /// println!(
    ///     "{} manual, {} auto-generated transcripts available",
    ///     transcript_list.manually_created_transcripts.len(),
    ///     transcript_list.generated_transcripts.len()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ci")]
    pub async fn list_transcripts(
        &self,
        video_id: &str,
    ) -> Result<TranscriptList, CouldNotRetrieveTranscript> {
        // For non-existent video ID, return an error
        if video_id == crate::tests::test_utils::NON_EXISTENT_VIDEO_ID {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
            });
        }

        // Return mock transcript list
        Ok(crate::tests::mocks::create_mock_transcript_list(
            self.client.clone(),
        ))
    }

    #[cfg(not(feature = "ci"))]
    pub async fn list_transcripts(
        &self,
        video_id: &str,
    ) -> Result<TranscriptList, CouldNotRetrieveTranscript> {
        self.fetcher.fetch_transcript_list(video_id).await
    }

    /// Fetches detailed metadata about a YouTube video.
    ///
    /// This method retrieves comprehensive information about a video, including its
    /// title, author, view count, description, thumbnails, and other metadata.
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID (e.g., "dQw4w9WgXcQ")
    ///
    /// # Returns
    ///
    /// * `Result<VideoDetails, CouldNotRetrieveTranscript>` - Video details or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The video does not exist or is private
    /// - Network issues prevent fetching the video details
    /// - The YouTube page structure has changed and details cannot be extracted
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    ///
    /// // Fetch details about a video
    /// let details = api.fetch_video_details("dQw4w9WgXcQ").await?;
    ///
    /// // Print basic information
    /// println!("Title: {}", details.title);
    /// println!("Channel: {}", details.author);
    /// println!("Views: {}", details.view_count);
    /// println!("Duration: {} seconds", details.length_seconds);
    ///
    /// // Print keywords if available
    /// if let Some(keywords) = &details.keywords {
    ///     println!("Keywords: {}", keywords.join(", "));
    /// }
    ///
    /// // Get the highest quality thumbnail
    /// if let Some(best_thumb) = details.thumbnails.iter()
    ///     .max_by_key(|t| t.width * t.height) {
    ///     println!("Best thumbnail: {} ({}x{})",
    ///         best_thumb.url, best_thumb.width, best_thumb.height);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ci")]
    pub async fn fetch_video_details(
        &self,
        video_id: &str,
    ) -> Result<VideoDetails, CouldNotRetrieveTranscript> {
        // For non-existent video ID, return an error
        if video_id == crate::tests::test_utils::NON_EXISTENT_VIDEO_ID {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
            });
        }

        // Return mock data
        Ok(crate::tests::mocks::create_mock_video_details())
    }

    #[cfg(not(feature = "ci"))]
    pub async fn fetch_video_details(
        &self,
        video_id: &str,
    ) -> Result<VideoDetails, CouldNotRetrieveTranscript> {
        self.fetcher.fetch_video_details(video_id).await
    }

    /// Fetches microformat data for a YouTube video.
    ///
    /// This method retrieves additional metadata about a video that's not included
    /// in the main video details, such as available countries, category, and embed information.
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID (e.g., "dQw4w9WgXcQ")
    ///
    /// # Returns
    ///
    /// * `Result<MicroformatData, CouldNotRetrieveTranscript>` - Microformat data or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The video does not exist or is private
    /// - Network issues prevent fetching the data
    /// - The YouTube page structure has changed and data cannot be extracted
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    ///
    /// // Fetch microformat data about a video
    /// let microformat = api.fetch_microformat("dQw4w9WgXcQ").await?;
    ///
    /// // Check if the video is unlisted
    /// if let Some(is_unlisted) = microformat.is_unlisted {
    ///     println!("Video is unlisted: {}", is_unlisted);
    /// }
    ///
    /// // Get video category
    /// if let Some(category) = microformat.category {
    ///     println!("Video category: {}", category);
    /// }
    ///
    /// // Check availability by country
    /// if let Some(countries) = microformat.available_countries {
    ///     println!("Video available in {} countries", countries.len());
    ///     if countries.contains(&"US".to_string()) {
    ///         println!("Video is available in the United States");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ci")]
    pub async fn fetch_microformat(
        &self,
        video_id: &str,
    ) -> Result<MicroformatData, CouldNotRetrieveTranscript> {
        // For non-existent video ID, return an error
        if video_id == crate::tests::test_utils::NON_EXISTENT_VIDEO_ID {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
            });
        }

        // Return mock data
        Ok(crate::tests::mocks::create_mock_microformat_data())
    }

    #[cfg(not(feature = "ci"))]
    pub async fn fetch_microformat(
        &self,
        video_id: &str,
    ) -> Result<MicroformatData, CouldNotRetrieveTranscript> {
        self.fetcher.fetch_microformat(video_id).await
    }

    /// Fetches streaming data for a YouTube video.
    ///
    /// This method retrieves information about available video and audio formats, including:
    /// - URLs for different quality versions of the video
    /// - Resolution, bitrate, and codec information
    /// - Both combined formats (with audio and video) and separate adaptive formats
    /// - Information about format expiration
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID (e.g., "dQw4w9WgXcQ")
    ///
    /// # Returns
    ///
    /// * `Result<StreamingData, CouldNotRetrieveTranscript>` - Streaming data or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The video does not exist or is private
    /// - The video has geo-restrictions that prevent access
    /// - Network issues prevent fetching the data
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    ///
    /// // Fetch streaming data about a video
    /// let streaming = api.fetch_streaming_data("dQw4w9WgXcQ").await?;
    ///
    /// // Print information about formats
    /// println!("Combined formats: {}", streaming.formats.len());
    /// println!("Adaptive formats: {}", streaming.adaptive_formats.len());
    ///
    /// // Find the highest resolution video format
    /// if let Some(best_video) = streaming.adaptive_formats.iter()
    ///     .filter(|f| f.width.is_some() && f.height.is_some())
    ///     .max_by_key(|f| f.height.unwrap_or(0)) {
    ///     println!("Best video quality: {}p", best_video.height.unwrap());
    ///     println!("Codec: {}", best_video.mime_type);
    /// }
    ///
    /// // Find the best audio format
    /// if let Some(best_audio) = streaming.adaptive_formats.iter()
    ///     .filter(|f| f.audio_quality.is_some())
    ///     .max_by_key(|f| f.bitrate) {
    ///     println!("Best audio quality: {}", best_audio.audio_quality.as_ref().unwrap());
    ///     println!("Bitrate: {} bps", best_audio.bitrate);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ci")]
    pub async fn fetch_streaming_data(
        &self,
        video_id: &str,
    ) -> Result<StreamingData, CouldNotRetrieveTranscript> {
        // For non-existent video ID, return an error
        if video_id == crate::tests::test_utils::NON_EXISTENT_VIDEO_ID {
            return Err(CouldNotRetrieveTranscript {
                video_id: video_id.to_string(),
                reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
            });
        }

        // Return mock data
        Ok(crate::tests::mocks::create_mock_streaming_data())
    }

    #[cfg(not(feature = "ci"))]
    pub async fn fetch_streaming_data(
        &self,
        video_id: &str,
    ) -> Result<StreamingData, CouldNotRetrieveTranscript> {
        self.fetcher.fetch_streaming_data(video_id).await
    }

    /// Fetches all available information about a YouTube video in a single request.
    ///
    /// This method retrieves comprehensive information about a video in one network call, including:
    /// - Video details (title, author, etc.)
    /// - Microformat data (category, available countries, etc.)
    /// - Streaming data (available formats, qualities, etc.)
    /// - Transcript list (available caption languages)
    ///
    /// This is more efficient than calling individual methods separately when multiple
    /// types of information are needed, as it avoids multiple HTTP requests.
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID (e.g., "dQw4w9WgXcQ")
    ///
    /// # Returns
    ///
    /// * `Result<VideoInfos, CouldNotRetrieveTranscript>` - Combined video information on success, or an error
    ///
    /// # Errors
    ///
    /// This method will return a `CouldNotRetrieveTranscript` error if:
    /// - The video doesn't exist or is private
    /// - The video has geo-restrictions that prevent access
    /// - Network errors occur during the request
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::api::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// let video_id = "dQw4w9WgXcQ";
    ///
    /// // Fetch all information in a single request
    /// let infos = api.fetch_video_infos(video_id).await?;
    ///
    /// // Access combined information
    /// println!("Title: {}", infos.video_details.title);
    /// println!("Author: {}", infos.video_details.author);
    ///
    /// if let Some(category) = &infos.microformat.category {
    ///     println!("Category: {}", category);
    /// }
    ///
    /// println!("Available formats: {}", infos.streaming_data.formats.len());
    /// println!("Available transcripts: {}", infos.transcript_list.transcripts().count());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(feature = "ci"))]
    pub async fn fetch_video_infos(
        &self,
        video_id: &str,
    ) -> Result<VideoInfos, CouldNotRetrieveTranscript> {
        self.fetcher.fetch_video_infos(video_id).await
    }

    /// Fetches all available information about a YouTube video in a single request.
    ///
    /// This is a CI-mode placeholder that always returns an error.
    #[cfg(feature = "ci")]
    pub async fn fetch_video_infos(
        &self,
        video_id: &str,
    ) -> Result<VideoInfos, CouldNotRetrieveTranscript> {
        Err(CouldNotRetrieveTranscript {
            video_id: video_id.to_string(),
            reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
        })
    }
}
