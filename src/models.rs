
use serde::{Deserialize, Serialize};


/// Information about a language that can be used for translation
#[derive(Debug, Clone)]
pub struct TranslationLanguage {
    pub language: String,
    pub language_code: String,
}

/// Represents a snippet of transcript text with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedTranscriptSnippet {
    /// The text content of this snippet
    pub text: String,

    /// The timestamp at which this snippet appears on screen in seconds
    pub start: f64,

    /// The duration of how long the snippet stays on screen in seconds
    /// Note that there can be overlaps between snippets
    pub duration: f64,
}

/// Video details parsed from YouTube's response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDetails {
    pub video_id: String,
    pub title: String,
    pub length_seconds: u32,
    pub keywords: Option<Vec<String>>,
    pub channel_id: String,
    pub short_description: String,
    pub view_count: String,
    pub author: String,
    pub thumbnails: Vec<VideoThumbnail>,
    pub is_live_content: bool,
}

/// Represents a YouTube video thumbnail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoThumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}
