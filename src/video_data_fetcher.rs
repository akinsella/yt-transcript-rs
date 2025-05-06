use reqwest::Client;
use std::fs;

use crate::errors::{
    CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason,
};
use crate::js_var_parser::JsVarParser;
use crate::models::VideoDetails;
use crate::playability_asserter::PlayabilityAsserter;
use crate::proxies::ProxyConfig;
use crate::transcript_list::TranscriptList;
use crate::video_details_extractor::VideoDetailsExtractor;
use crate::youtube_page_fetcher::YoutubePageFetcher;

/// Fetcher for transcript lists
pub struct VideoDataFetcher {
    client: Client,
    proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>,
    page_fetcher: YoutubePageFetcher,
}

impl VideoDataFetcher {
    pub fn new(client: Client, proxy_config: Option<Box<dyn ProxyConfig + Send + Sync>>) -> Self {
        // We can't clone the proxy_config since it's a trait object,
        // but we can pass it to the page fetcher and keep None for ourselves.
        // Given that we're now delegating all HTML fetching to the page fetcher,
        // we don't need to keep a copy of the proxy_config in this struct anymore.
        let page_fetcher = YoutubePageFetcher::new(client.clone(), proxy_config);

        Self {
            client,
            proxy_config: None,
            page_fetcher,
        }
    }

    /// Fetch the list of available transcripts for a video
    pub async fn fetch_transcript_list(
        &self,
        video_id: &str,
    ) -> Result<TranscriptList, CouldNotRetrieveTranscript> {
        let video_captions = self.fetch_video_captions(video_id).await?;

        // println!("captions_json: {}", captions_json);
        fs::write(
            "/Users/alexis.kinsella/Desktop/video_captions.json",
            video_captions.to_string(),
        )
        .unwrap();

        TranscriptList::build(self.client.clone(), video_id.to_string(), &video_captions)
    }

    /// Fetch the captions JSON data from YouTube
    async fn fetch_video_captions(
        &self,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        // Fetch the video HTML using the page fetcher
        let html = self.page_fetcher.fetch_video_page(video_id).await?;

        // println!("html: {}", html);
        fs::write("/Users/alexis.kinsella/Desktop/captions.json", html.clone()).unwrap();

        // Extract captions JSON
        self.extract_captions_json(&html, video_id)
    }

    /// Fetch video details from YouTube
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

    fn extract_yt_initial_player_response(
        &self,
        html: &str,
        video_id: &str,
    ) -> Result<serde_json::Value, CouldNotRetrieveTranscript> {
        let js_var_parser = JsVarParser::new("ytInitialPlayerResponse");
        let player_response = js_var_parser.parse(html, video_id)?;

        Ok(player_response)
    }

    /// Extract captions JSON from HTML
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
