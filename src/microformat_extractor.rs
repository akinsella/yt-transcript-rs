use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use crate::models::{MicroformatData, MicroformatEmbed, MicroformatThumbnail, VideoThumbnail};

/// # MicroformatExtractor
///
/// Extracts microformat information from YouTube's player response data.
///
/// The microformat data contains additional metadata about the video that is not included
/// in the main video details, such as available countries, category, and embed information.
pub struct MicroformatExtractor;

impl MicroformatExtractor {
    /// Extracts microformat data from the player response JSON.
    ///
    /// # Parameters
    ///
    /// * `player_response` - The parsed YouTube player response JSON object
    /// * `video_id` - The YouTube video ID (used for error reporting)
    ///
    /// # Returns
    ///
    /// * `Result<MicroformatData, CouldNotRetrieveTranscript>` - The parsed microformat data or an error
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The microformat data is missing from the player response
    /// - The JSON structure does not match the expected format
    pub fn extract_microformat_data(
        player_response: &serde_json::Value,
        video_id: &str,
    ) -> Result<MicroformatData, CouldNotRetrieveTranscript> {
        let renderer = match player_response.get("microformat") {
            Some(microformat) => match microformat.get("playerMicroformatRenderer") {
                Some(renderer) => renderer,
                None => {
                    return Err(CouldNotRetrieveTranscript {
                        video_id: video_id.to_string(),
                        reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
                    });
                }
            },
            None => {
                return Err(CouldNotRetrieveTranscript {
                    video_id: video_id.to_string(),
                    reason: Some(CouldNotRetrieveTranscriptReason::VideoUnavailable),
                });
            }
        };

        // Manual extraction of fields to handle YouTube's nested format
        let mut microformat_data = MicroformatData {
            available_countries: None,
            category: None,
            description: None,
            embed: None,
            external_channel_id: None,
            external_video_id: None,
            has_ypc_metadata: None,
            is_family_safe: None,
            is_shorts_eligible: None,
            is_unlisted: None,
            length_seconds: None,
            like_count: None,
            owner_channel_name: None,
            owner_profile_url: None,
            publish_date: None,
            thumbnail: None,
            title: None,
            upload_date: None,
            view_count: None,
        };

        // Extract simple string fields
        if let Some(value) = renderer.get("externalVideoId") {
            if let Some(s) = value.as_str() {
                microformat_data.external_video_id = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("externalChannelId") {
            if let Some(s) = value.as_str() {
                microformat_data.external_channel_id = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("ownerChannelName") {
            if let Some(s) = value.as_str() {
                microformat_data.owner_channel_name = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("ownerProfileUrl") {
            if let Some(s) = value.as_str() {
                microformat_data.owner_profile_url = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("category") {
            if let Some(s) = value.as_str() {
                microformat_data.category = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("lengthSeconds") {
            if let Some(s) = value.as_str() {
                microformat_data.length_seconds = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("viewCount") {
            if let Some(s) = value.as_str() {
                microformat_data.view_count = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("likeCount") {
            if let Some(s) = value.as_str() {
                microformat_data.like_count = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("uploadDate") {
            if let Some(s) = value.as_str() {
                microformat_data.upload_date = Some(s.to_string());
            }
        }

        if let Some(value) = renderer.get("publishDate") {
            if let Some(s) = value.as_str() {
                microformat_data.publish_date = Some(s.to_string());
            }
        }

        // Extract boolean fields
        if let Some(value) = renderer.get("isFamilySafe") {
            if let Some(b) = value.as_bool() {
                microformat_data.is_family_safe = Some(b);
            }
        }

        if let Some(value) = renderer.get("isUnlisted") {
            if let Some(b) = value.as_bool() {
                microformat_data.is_unlisted = Some(b);
            }
        }

        if let Some(value) = renderer.get("isShortsEligible") {
            if let Some(b) = value.as_bool() {
                microformat_data.is_shorts_eligible = Some(b);
            }
        }

        if let Some(value) = renderer.get("hasYpcMetadata") {
            if let Some(b) = value.as_bool() {
                microformat_data.has_ypc_metadata = Some(b);
            }
        }

        // Extract nested fields
        // Title (which is in simpleText format)
        if let Some(title) = renderer.get("title") {
            if let Some(simple_text) = title.get("simpleText") {
                if let Some(text) = simple_text.as_str() {
                    microformat_data.title = Some(text.to_string());
                }
            }
        }

        // Description (which is in simpleText format)
        if let Some(description) = renderer.get("description") {
            if let Some(simple_text) = description.get("simpleText") {
                if let Some(text) = simple_text.as_str() {
                    microformat_data.description = Some(text.to_string());
                }
            }
        }

        // Available countries (which is an array)
        if let Some(countries) = renderer.get("availableCountries") {
            if let Some(countries_array) = countries.as_array() {
                let mut country_list = Vec::new();
                for country in countries_array {
                    if let Some(country_code) = country.as_str() {
                        country_list.push(country_code.to_string());
                    }
                }
                if !country_list.is_empty() {
                    microformat_data.available_countries = Some(country_list);
                }
            }
        }

        // Embed information
        if let Some(embed_obj) = renderer.get("embed") {
            let mut embed = MicroformatEmbed {
                height: None,
                iframe_url: None,
                width: None,
            };

            if let Some(height) = embed_obj.get("height") {
                if let Some(h) = height.as_i64() {
                    embed.height = Some(h as i32);
                }
            }

            if let Some(width) = embed_obj.get("width") {
                if let Some(w) = width.as_i64() {
                    embed.width = Some(w as i32);
                }
            }

            if let Some(url) = embed_obj.get("iframeUrl") {
                if let Some(u) = url.as_str() {
                    embed.iframe_url = Some(u.to_string());
                }
            }

            microformat_data.embed = Some(embed);
        }

        // Thumbnail information
        if let Some(thumbnail_obj) = renderer.get("thumbnail") {
            if let Some(thumbnails) = thumbnail_obj.get("thumbnails") {
                if let Some(thumbnail_array) = thumbnails.as_array() {
                    let mut thumb_list = Vec::new();

                    for thumb in thumbnail_array {
                        let mut video_thumb = VideoThumbnail {
                            url: String::new(),
                            width: 0,
                            height: 0,
                        };

                        if let Some(url) = thumb.get("url") {
                            if let Some(u) = url.as_str() {
                                video_thumb.url = u.to_string();
                            }
                        }

                        if let Some(width) = thumb.get("width") {
                            if let Some(w) = width.as_i64() {
                                video_thumb.width = w as u32;
                            }
                        }

                        if let Some(height) = thumb.get("height") {
                            if let Some(h) = height.as_i64() {
                                video_thumb.height = h as u32;
                            }
                        }

                        // Only add if we have a valid URL
                        if !video_thumb.url.is_empty() {
                            thumb_list.push(video_thumb);
                        }
                    }

                    if !thumb_list.is_empty() {
                        microformat_data.thumbnail = Some(MicroformatThumbnail {
                            thumbnails: Some(thumb_list),
                        });
                    }
                }
            }
        }

        Ok(microformat_data)
    }
}
