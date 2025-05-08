use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use crate::models::{ColorInfo, Range, StreamingData, StreamingFormat};
use serde_json::Value;

/// # StreamingDataExtractor
///
/// Extracts streaming data information from YouTube's player response data.
///
/// The streaming data contains information about available video and audio formats,
/// including URLs, quality options, bitrates, and codec details.
pub struct StreamingDataExtractor;

impl StreamingDataExtractor {
    /// Extracts streaming data from the player response JSON.
    ///
    /// # Parameters
    ///
    /// * `player_response` - The parsed YouTube player response JSON object
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<StreamingData, CouldNotRetrieveTranscript>` - The parsed streaming data or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The streaming data is missing from the player response
    /// - The JSON structure does not match the expected format
    pub fn extract_streaming_data(
        player_response: &Value,
        video_id: &str,
    ) -> Result<StreamingData, CouldNotRetrieveTranscript> {
        let streaming_data = match player_response.get("streamingData") {
            Some(data) => data,
            None => {
                return Err(CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
                });
            }
        };

        // Extract expires_in_seconds
        let expires_in_seconds = match streaming_data.get("expiresInSeconds") {
            Some(Value::String(s)) => s.clone(),
            _ => "0".to_string(), // Default to 0 if not found
        };

        // Extract formats
        let formats = Self::extract_formats(streaming_data.get("formats"));

        // Extract adaptive formats
        let adaptive_formats = Self::extract_formats(streaming_data.get("adaptiveFormats"));

        // Extract server ABR streaming URL
        let server_abr_streaming_url = match streaming_data.get("serverAbrStreamingUrl") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        };

        Ok(StreamingData {
            expires_in_seconds,
            formats,
            adaptive_formats,
            server_abr_streaming_url,
        })
    }

    /// Extracts formats from a JSON array
    ///
    /// # Parameters
    ///
    /// * `formats_value` - Optional JSON value containing an array of format objects
    ///
    /// # Returns
    ///
    /// * `Vec<StreamingFormat>` - Vector of parsed streaming formats
    fn extract_formats(formats_value: Option<&Value>) -> Vec<StreamingFormat> {
        let mut formats = Vec::new();

        if let Some(Value::Array(array)) = formats_value {
            for item in array {
                if let Some(format) = Self::parse_format(item) {
                    formats.push(format);
                }
            }
        }

        formats
    }

    /// Parses a single format from JSON
    ///
    /// # Parameters
    ///
    /// * `format_json` - JSON object containing format data
    ///
    /// # Returns
    ///
    /// * `Option<StreamingFormat>` - Parsed format or None if parsing failed
    fn parse_format(format_json: &Value) -> Option<StreamingFormat> {
        // Required fields
        let itag = format_json.get("itag")?.as_u64()? as u32;
        let mime_type = format_json.get("mimeType")?.as_str()?.to_string();
        let bitrate = format_json.get("bitrate")?.as_u64()?;
        let quality = format_json.get("quality")?.as_str()?.to_string();
        let projection_type = format_json.get("projectionType")?.as_str()?.to_string();
        let approx_duration_ms = format_json.get("approxDurationMs")?.as_str()?.to_string();

        // Optional fields
        let url = format_json
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let width = format_json
            .get("width")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let height = format_json
            .get("height")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let fps = format_json
            .get("fps")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let quality_label = format_json
            .get("qualityLabel")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let average_bitrate = format_json.get("averageBitrate").and_then(|v| v.as_u64());

        let audio_quality = format_json
            .get("audioQuality")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let audio_sample_rate = format_json
            .get("audioSampleRate")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let audio_channels = format_json
            .get("audioChannels")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let quality_ordinal = format_json
            .get("qualityOrdinal")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let high_replication = format_json.get("highReplication").and_then(|v| v.as_bool());

        let last_modified = format_json
            .get("lastModified")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let content_length = format_json
            .get("contentLength")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let loudness_db = format_json.get("loudnessDb").and_then(|v| v.as_f64());

        let is_drc = format_json.get("isDrc").and_then(|v| v.as_bool());

        let xtags = format_json
            .get("xtags")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract init_range if present
        let init_range = format_json.get("initRange").and_then(|range| {
            let start = range.get("start")?.as_str()?.to_string();
            let end = range.get("end")?.as_str()?.to_string();
            Some(Range { start, end })
        });

        // Extract index_range if present
        let index_range = format_json.get("indexRange").and_then(|range| {
            let start = range.get("start")?.as_str()?.to_string();
            let end = range.get("end")?.as_str()?.to_string();
            Some(Range { start, end })
        });

        // Extract color_info if present
        let color_info = format_json.get("colorInfo").and_then(|color| {
            let primaries = color
                .get("primaries")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let transfer_characteristics = color
                .get("transferCharacteristics")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let matrix_coefficients = color
                .get("matrixCoefficients")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(ColorInfo {
                primaries,
                transfer_characteristics,
                matrix_coefficients,
            })
        });

        Some(StreamingFormat {
            itag,
            url,
            mime_type,
            bitrate,
            width,
            height,
            init_range,
            index_range,
            last_modified,
            content_length,
            quality,
            fps,
            quality_label,
            projection_type,
            average_bitrate,
            audio_quality,
            approx_duration_ms,
            audio_sample_rate,
            audio_channels,
            quality_ordinal,
            high_replication,
            color_info,
            loudness_db,
            is_drc,
            xtags,
        })
    }
}
