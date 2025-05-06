use serde::{Deserialize, Serialize};

/// # TranslationLanguage
///
/// Represents a language option available for YouTube transcript translation.
///
/// This struct contains both the human-readable language name and the
/// ISO language code that YouTube uses to identify the language.
///
/// ## Fields
///
/// * `language` - The full, human-readable name of the language (e.g., "English")
/// * `language_code` - The ISO language code used by YouTube (e.g., "en")
///
/// ## Example Usage
///
/// ```rust,no_run
/// # use yt_transcript_rs::models::TranslationLanguage;
/// let english = TranslationLanguage {
///     language: "English".to_string(),
///     language_code: "en".to_string(),
/// };
///
/// println!("Language: {} ({})", english.language, english.language_code);
/// ```
#[derive(Debug, Clone)]
pub struct TranslationLanguage {
    /// The human-readable language name (e.g., "English", "Español", "Français")
    pub language: String,

    /// The ISO language code used by YouTube's API (e.g., "en", "es", "fr")
    pub language_code: String,
}

/// # FetchedTranscriptSnippet
///
/// Represents a single segment of transcript text with its timing information.
///
/// YouTube transcripts are divided into discrete text segments, each with a
/// specific start time and duration. This struct captures one such segment.
///
/// ## Fields
///
/// * `text` - The actual transcript text for this segment
/// * `start` - The timestamp when this text appears (in seconds)
/// * `duration` - How long this text stays on screen (in seconds)
///
/// ## Notes
///
/// - Transcript segments may overlap in time
/// - The `text` may include HTML formatting if formatting preservation is enabled
/// - Time values are floating point to allow for sub-second precision
///
/// ## Example Usage
///
/// ```rust,no_run
/// # use yt_transcript_rs::models::FetchedTranscriptSnippet;
/// let snippet = FetchedTranscriptSnippet {
///     text: "Hello, world!".to_string(),
///     start: 5.2,
///     duration: 2.5,
/// };
///
/// println!("[{:.1}s-{:.1}s]: {}",
///     snippet.start,
///     snippet.start + snippet.duration,
///     snippet.text);
/// // Outputs: [5.2s-7.7s]: Hello, world!
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedTranscriptSnippet {
    /// The text content of this snippet
    pub text: String,

    /// The timestamp at which this snippet appears on screen in seconds
    pub start: f64,

    /// The duration of how long the snippet stays on screen in seconds.
    /// Note that there can be overlaps between snippets
    pub duration: f64,
}

/// # VideoDetails
///
/// Comprehensive metadata about a YouTube video.
///
/// This struct contains detailed information about a video, extracted from
/// YouTube's player response. It includes basic information like title and author,
/// as well as more detailed metadata like view count, keywords, and thumbnails.
///
/// ## Fields
///
/// * `video_id` - The unique YouTube ID for the video
/// * `title` - The video's title
/// * `length_seconds` - The video duration in seconds
/// * `keywords` - Optional list of keywords/tags associated with the video
/// * `channel_id` - The YouTube channel ID
/// * `short_description` - The video description
/// * `view_count` - Number of views as a string (to handle very large numbers)
/// * `author` - Name of the channel/creator
/// * `thumbnails` - List of available thumbnail images in various resolutions
/// * `is_live_content` - Whether the video is or was a live stream
///
/// ## Example Usage
///
/// ```rust,no_run
/// # use yt_transcript_rs::api::YouTubeTranscriptApi;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let api = YouTubeTranscriptApi::new(None, None, None)?;
/// let video_id = "dQw4w9WgXcQ";
///
/// // Fetch video details
/// let details = api.fetch_video_details(video_id).await?;
///
/// println!("Title: {}", details.title);
/// println!("By: {} ({})", details.author, details.channel_id);
/// println!("Duration: {} seconds", details.length_seconds);
/// println!("Views: {}", details.view_count);
///
/// // Get highest resolution thumbnail
/// if let Some(thumbnail) = details.thumbnails.iter().max_by_key(|t| t.width * t.height) {
///     println!("Thumbnail URL: {}", thumbnail.url);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDetails {
    /// The unique YouTube video ID (e.g., "dQw4w9WgXcQ")
    pub video_id: String,

    /// The video's title
    pub title: String,

    /// Total duration of the video in seconds
    pub length_seconds: u32,

    /// Optional list of keywords/tags associated with the video
    pub keywords: Option<Vec<String>>,

    /// The YouTube channel ID that published the video
    pub channel_id: String,

    /// The video description text
    pub short_description: String,

    /// Number of views as a string (to handle potentially very large numbers)
    pub view_count: String,

    /// Name of the channel/creator who published the video
    pub author: String,

    /// List of available thumbnail images in various resolutions
    pub thumbnails: Vec<VideoThumbnail>,

    /// Whether the video is or was a live stream
    pub is_live_content: bool,
}

/// # VideoThumbnail
///
/// Represents a single thumbnail image for a YouTube video.
///
/// YouTube provides thumbnails in multiple resolutions, and this struct
/// stores information about one such thumbnail, including its URL and dimensions.
///
/// ## Fields
///
/// * `url` - Direct URL to the thumbnail image
/// * `width` - Width of the thumbnail in pixels
/// * `height` - Height of the thumbnail in pixels
///
/// ## Common Resolutions
///
/// YouTube typically provides thumbnails in these standard resolutions:
/// - Default: 120×90
/// - Medium: 320×180
/// - High: 480×360
/// - Standard: 640×480
/// - Maxres: 1280×720
///
/// ## Example Usage
///
/// ```rust,no_run
/// # use yt_transcript_rs::models::VideoThumbnail;
/// let thumbnail = VideoThumbnail {
///     url: "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg".to_string(),
///     width: 480,
///     height: 360,
/// };
///
/// println!("Thumbnail ({}×{}): {}", thumbnail.width, thumbnail.height, thumbnail.url);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoThumbnail {
    /// Direct URL to the thumbnail image
    pub url: String,

    /// Width of the thumbnail in pixels
    pub width: u32,

    /// Height of the thumbnail in pixels
    pub height: u32,
}
