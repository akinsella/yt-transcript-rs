use reqwest::Client;
use std::path::Path;
use std::sync::Arc;

use crate::video_data_fetcher::VideoDataFetcher;
use crate::cookie_jar_loader::CookieJarLoader;
use crate::errors::{CookieError, CouldNotRetrieveTranscript};
use crate::models::VideoDetails;
use crate::proxies::ProxyConfig;
use crate::{FetchedTranscript, TranscriptList};

/// Main API for fetching YouTube transcripts
#[derive(Clone)]
pub struct YouTubeTranscriptApi {
    fetcher: Arc<VideoDataFetcher>,
    client: Client,
}

impl YouTubeTranscriptApi {
    /// Create a new instance of the API
    pub fn new(
        cookie_path: Option<&Path>,
        proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>,
        http_client: Option<Client>,
    ) -> Result<Self, CookieError> {
        let client = match http_client {
            Some(client) => client,
            None => {
                let mut builder = Client::builder()
                    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
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

        let fetcher = Arc::new(VideoDataFetcher::new(client.clone(), proxy_config));

        Ok(Self { fetcher, client })
    }

    /// Fetch a transcript for a single video
    pub async fn fetch_transcript(
        &self,
        video_id: &str,
        languages: &[&str],
        preserve_formatting: bool,
    ) -> Result<FetchedTranscript, CouldNotRetrieveTranscript> {
        let transcript_list = self.list_transcripts(video_id).await?;
        let transcript = transcript_list.find_transcript(languages)?;
        transcript.fetch(preserve_formatting).await
    }

    /// List all available transcripts for a video
    pub async fn list_transcripts(
        &self,
        video_id: &str,
    ) -> Result<TranscriptList, CouldNotRetrieveTranscript> {
        self.fetcher.fetch_transcript_list(video_id).await
    }

    /// Fetch video details (title, description, view count, etc.)
    pub async fn fetch_video_details(
        &self,
        video_id: &str,
    ) -> Result<VideoDetails, CouldNotRetrieveTranscript> {
        self.fetcher.fetch_video_details(video_id).await
    }
}
