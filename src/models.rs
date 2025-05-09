use crate::transcript_list::TranscriptList;
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
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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

/// Represents microformat data for a YouTube video
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MicroformatData {
    /// Countries where the video is available
    pub available_countries: Option<Vec<String>>,
    /// Category of the video
    pub category: Option<String>,
    /// Description of the video
    pub description: Option<String>,
    /// Embed information
    pub embed: Option<MicroformatEmbed>,
    /// External channel ID
    pub external_channel_id: Option<String>,
    /// External video ID
    pub external_video_id: Option<String>,
    /// Whether the video has YPC metadata
    pub has_ypc_metadata: Option<bool>,
    /// Whether the video is family safe
    pub is_family_safe: Option<bool>,
    /// Whether the video is eligible for Shorts
    pub is_shorts_eligible: Option<bool>,
    /// Whether the video is unlisted
    pub is_unlisted: Option<bool>,
    /// Duration of the video in seconds
    pub length_seconds: Option<String>,
    /// Number of likes
    pub like_count: Option<String>,
    /// Name of the channel owner
    pub owner_channel_name: Option<String>,
    /// URL to the owner's profile
    pub owner_profile_url: Option<String>,
    /// Date when the video was published
    pub publish_date: Option<String>,
    /// Thumbnail information
    pub thumbnail: Option<MicroformatThumbnail>,
    /// Title of the video
    pub title: Option<String>,
    /// Date when the video was uploaded
    pub upload_date: Option<String>,
    /// Number of views
    pub view_count: Option<String>,
}

/// Represents embed information in microformat data
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MicroformatEmbed {
    /// Height of the embed
    pub height: Option<i32>,
    /// URL for the iframe embed
    pub iframe_url: Option<String>,
    /// Width of the embed
    pub width: Option<i32>,
}

/// Represents thumbnail information in microformat data
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MicroformatThumbnail {
    /// List of thumbnails in different sizes
    pub thumbnails: Option<Vec<VideoThumbnail>>,
}

/// Represents a range with start and end values
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Range {
    /// Start position
    pub start: String,
    /// End position
    pub end: String,
}

/// Represents color information for a video format
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ColorInfo {
    /// Primary colors used
    pub primaries: Option<String>,
    /// Transfer characteristics
    pub transfer_characteristics: Option<String>,
    /// Matrix coefficients
    pub matrix_coefficients: Option<String>,
}

/// Represents a single video or audio format available for streaming
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StreamingFormat {
    /// Format identification number
    pub itag: u32,
    /// URL to the media
    pub url: Option<String>,
    /// MIME type and codec information
    pub mime_type: String,
    /// Bitrate in bits per second
    pub bitrate: u64,
    /// Video width in pixels (video only)
    pub width: Option<u32>,
    /// Video height in pixels (video only)
    pub height: Option<u32>,
    /// Initialization range for segmented formats
    pub init_range: Option<Range>,
    /// Index range for segmented formats
    pub index_range: Option<Range>,
    /// Last modification timestamp
    pub last_modified: Option<String>,
    /// Content length in bytes
    pub content_length: Option<String>,
    /// Quality label (e.g., "medium", "hd720")
    pub quality: String,
    /// Frames per second (video only)
    pub fps: Option<u32>,
    /// Human-readable quality label (e.g., "720p")
    pub quality_label: Option<String>,
    /// Projection type (e.g., "RECTANGULAR")
    pub projection_type: String,
    /// Average bitrate in bits per second
    pub average_bitrate: Option<u64>,
    /// Audio quality (audio only)
    pub audio_quality: Option<String>,
    /// Approximate duration in milliseconds
    pub approx_duration_ms: String,
    /// Audio sample rate (audio only)
    pub audio_sample_rate: Option<String>,
    /// Number of audio channels (audio only)
    pub audio_channels: Option<u32>,
    /// Quality ordinal value
    pub quality_ordinal: Option<String>,
    /// High replication flag
    pub high_replication: Option<bool>,
    /// Color information
    pub color_info: Option<ColorInfo>,
    /// Loudness in decibels (audio only)
    pub loudness_db: Option<f64>,
    /// Whether DRC (Dynamic Range Compression) is used
    pub is_drc: Option<bool>,
    /// Extra tags
    pub xtags: Option<String>,
}

/// Represents all available streaming data for a YouTube video
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StreamingData {
    /// Time in seconds until the streaming URLs expire
    pub expires_in_seconds: String,
    /// Combined formats with both audio and video
    pub formats: Vec<StreamingFormat>,
    /// Separate adaptive formats for audio or video
    pub adaptive_formats: Vec<StreamingFormat>,
    /// Server ABR streaming URL
    pub server_abr_streaming_url: Option<String>,
}

/// # VideoInfos
///
/// Comprehensive container for all available information about a YouTube video.
///
/// This struct combines metadata from different extractors into a single structure,
/// providing a complete view of a video's details, streaming options, and available transcripts.
///
/// The advantage of using this struct is that it retrieves all information in a single request
/// to YouTube, rather than making multiple separate requests.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VideoInfos {
    /// Basic details about the video (title, author, view count, etc.)
    pub video_details: VideoDetails,

    /// Extended metadata from the microformat section (categories, countries, embed info)
    pub microformat: MicroformatData,

    /// Information about available streaming formats (qualities, codecs, URLs)
    pub streaming_data: StreamingData,

    /// List of available transcripts/captions for the video
    pub transcript_list: TranscriptList,
}
