//! # yt-transcript-rs
//!
//! A Rust library for fetching YouTube video transcripts and metadata.
//!
//! This library provides a robust, feature-rich interface for retrieving transcripts
//! from YouTube videos. It can fetch transcripts in various languages, auto-generated
//! captions, and detailed video metadata, all without using YouTube's official API
//! (which doesn't provide transcript access).
//!
//! ## Key Features
//!
//! - **Transcript Retrieval**: Fetch transcripts in any available language
//! - **Multi-language Support**: Request transcripts in order of language preference
//! - **Transcript Translation**: Translate transcripts to different languages
//! - **Formatting Preservation**: Option to keep HTML formatting in transcripts
//! - **Video Metadata**: Retrieve detailed information about videos (title, author, etc.)
//! - **Advanced Authentication**: Support for cookies to access restricted content
//! - **Proxy Support**: Route requests through proxies to bypass geo-restrictions
//!
//! ## Simple Usage Example
//!
//! ```rust,no_run
//! use yt_transcript_rs::YouTubeTranscriptApi;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new API instance
//!     let api = YouTubeTranscriptApi::new(None, None, None)?;
//!
//!     // Fetch an English transcript
//!     let transcript = api.fetch_transcript(
//!         "dQw4w9WgXcQ",  // Video ID
//!         &["en"],        // Preferred languages
//!         false           // Don't preserve formatting
//!     ).await?;
//!
//!     // Print the full transcript text
//!     println!("Transcript: {}", transcript.text());
//!
//!     // Or work with individual snippets
//!     for snippet in transcript.parts() {
//!         println!("[{:.1}s]: {}", snippet.start, snippet.text);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Use Cases
//!
//! ### Language Preferences
//!
//! ```rust,no_run
//! # use yt_transcript_rs::YouTubeTranscriptApi;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let api = YouTubeTranscriptApi::new(None, None, None)?;
//!
//! // Try English first, then Spanish, then auto-generated
//! let transcript = api.fetch_transcript(
//!     "dQw4w9WgXcQ",
//!     &["en", "es", "en-US"],
//!     false
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### List Available Transcripts
//!
//! ```rust,no_run
//! # use yt_transcript_rs::YouTubeTranscriptApi;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let api = YouTubeTranscriptApi::new(None, None, None)?;
//!
//! let transcript_list = api.list_transcripts("dQw4w9WgXcQ").await?;
//!
//! for transcript in transcript_list.transcripts() {
//!     println!("Language: {} ({}) - {} generated",
//!         transcript.language(),
//!         transcript.language_code(),
//!         if transcript.is_generated() { "Auto" } else { "Manually" });
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Fetch Video Details
//!
//! ```rust,no_run
//! # use yt_transcript_rs::YouTubeTranscriptApi;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let api = YouTubeTranscriptApi::new(None, None, None)?;
//!
//! let details = api.fetch_video_details("dQw4w9WgXcQ").await?;
//!
//! println!("Title: {}", details.title);
//! println!("Author: {}", details.author);
//! println!("Views: {}", details.view_count);
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod captions_extractor;
pub mod cookie_jar_loader;
pub mod errors;
pub mod fetched_transcript;
pub mod js_var_parser;
pub mod microformat_extractor;
pub mod models;
pub mod playability_asserter;
pub mod proxies;
pub mod streaming_data_extractor;
pub mod tests;
pub mod transcript;
pub mod transcript_list;
pub mod transcript_parser;
pub mod video_data_fetcher;
pub mod video_details_extractor;
pub mod youtube_page_fetcher;

pub use api::YouTubeTranscriptApi;
pub use cookie_jar_loader::CookieJarLoader;
pub use errors::{
    AgeRestricted, CookieError, CookieInvalid, CookiePathInvalid, CouldNotRetrieveTranscript,
    FailedToCreateConsentCookie, InvalidVideoId, IpBlocked, NoTranscriptFound, NotTranslatable,
    RequestBlocked, TranscriptsDisabled, TranslationLanguageNotAvailable, VideoUnavailable,
    VideoUnplayable, YouTubeDataUnparsable, YouTubeRequestFailed, YouTubeTranscriptApiError,
};

pub use captions_extractor::CaptionsExtractor;
pub use fetched_transcript::FetchedTranscript;
pub use models::FetchedTranscriptSnippet;
pub use models::VideoDetails;
pub use models::VideoInfos;
pub use models::VideoThumbnail;
pub use models::{ColorInfo, Range, StreamingData, StreamingFormat};
pub use models::{MicroformatData, MicroformatEmbed, MicroformatThumbnail};
pub use playability_asserter::PlayabilityAsserter;
pub use streaming_data_extractor::StreamingDataExtractor;
pub use transcript::Transcript;
pub use transcript_list::TranscriptList;
pub use video_details_extractor::VideoDetailsExtractor;
pub use youtube_page_fetcher::YoutubePageFetcher;
