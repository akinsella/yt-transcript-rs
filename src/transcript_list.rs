use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::errors::{CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason};
use crate::models::TranslationLanguage;
use crate::transcript::Transcript;

/// # TranscriptList
///
/// A collection of available transcripts for a YouTube video.
///
/// This struct provides access to all transcripts available for a video, including:
/// - Manually created transcripts (by the video owner or contributors)
/// - Automatically generated transcripts (created by YouTube's speech recognition)
/// - Available translation languages for translatable transcripts
///
/// The `TranscriptList` differentiates between manually created and automatically generated
/// transcripts, as the manually created ones tend to be more accurate. This allows you
/// to prioritize manually created transcripts over automatically generated ones.
///
/// ## Usage Example
///
/// ```rust,no_run
/// # use yt_transcript_rs::YouTubeTranscriptApi;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let api = YouTubeTranscriptApi::new(None, None, None)?;
///
/// // Get a list of all available transcripts for a video
/// let transcript_list = api.list_transcripts("dQw4w9WgXcQ").await?;
///
/// // Print all available transcripts
/// println!("Available transcripts: {}", transcript_list);
///
/// // Find a transcript in a specific language (prioritizing English)
/// let transcript = transcript_list.find_transcript(&["en", "en-US"])?;
///
/// // Or specifically find a manually created transcript
/// let manual_transcript = transcript_list.find_manually_created_transcript(&["en"])?;
///
/// // Or retrieve an automatically generated transcript
/// let auto_transcript = transcript_list.find_generated_transcript(&["en"])?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranscriptList {
    /// The YouTube video ID this transcript list belongs to
    pub video_id: String,

    /// Map of language codes to manually created transcripts
    pub manually_created_transcripts: HashMap<String, Transcript>,

    /// Map of language codes to automatically generated transcripts
    pub generated_transcripts: HashMap<String, Transcript>,

    /// List of languages available for translation
    pub translation_languages: Vec<TranslationLanguage>,
}

impl TranscriptList {
    /// Creates a new TranscriptList with the provided components.
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID this transcript list belongs to
    /// * `manually_created_transcripts` - Map of language codes to manually created transcripts
    /// * `generated_transcripts` - Map of language codes to automatically generated transcripts
    /// * `translation_languages` - List of languages available for translation
    ///
    /// # Returns
    ///
    /// A new `TranscriptList` instance
    pub fn new(
        video_id: String,
        manually_created_transcripts: HashMap<String, Transcript>,
        generated_transcripts: HashMap<String, Transcript>,
        translation_languages: Vec<TranslationLanguage>,
    ) -> Self {
        Self {
            video_id,
            manually_created_transcripts,
            generated_transcripts,
            translation_languages,
        }
    }

    /// Creates a TranscriptList from YouTube's caption JSON data.
    ///
    /// This method parses YouTube's internal caption data structure to extract:
    /// - Available transcripts (both manual and automatic)
    /// - Their respective language codes and names
    /// - Information about available translation languages
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID
    /// * `video_page_html` - JSON data extracted from YouTube's page containing caption information
    ///
    /// # Returns
    ///
    /// * `Result<Self, CouldNotRetrieveTranscript>` - A transcript list or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the caption data cannot be properly parsed.
    pub fn build(
        video_id: String,
        video_page_html: &serde_json::Value,
    ) -> Result<Self, CouldNotRetrieveTranscript> {
        let transcript_list = Self::build_without_client(video_id, video_page_html)?;

        Ok(transcript_list)
    }

    /// Creates a TranscriptList from YouTube's caption JSON data without requiring a client.
    ///
    /// This method is similar to `build` but doesn't take a client parameter, making it
    /// suitable for use in serialization/deserialization contexts.
    ///
    /// # Parameters
    ///
    /// * `video_id` - The YouTube video ID
    /// * `video_page_html` - JSON data extracted from YouTube's page containing caption information
    ///
    /// # Returns
    ///
    /// * `Result<Self, CouldNotRetrieveTranscript>` - A transcript list or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the caption data cannot be properly parsed.
    pub fn build_without_client(
        video_id: String,
        video_page_html: &serde_json::Value,
    ) -> Result<Self, CouldNotRetrieveTranscript> {
        // Extract translation languages
        let empty_vec = vec![];
        let translation_languages_json = match video_page_html.get("translationLanguages") {
            Some(val) => val.as_array().unwrap_or(&empty_vec),
            None => &empty_vec,
        };

        let translation_languages = translation_languages_json
            .iter()
            .filter_map(|lang| {
                let language_name = lang.get("languageName")?.get("simpleText")?.as_str()?;
                let language_code = lang.get("languageCode")?.as_str()?;

                Some(TranslationLanguage {
                    language: language_name.to_string(),
                    language_code: language_code.to_string(),
                })
            })
            .collect::<Vec<_>>();

        // Extract transcripts
        let caption_tracks = match video_page_html.get("captionTracks") {
            Some(val) => val.as_array().unwrap_or(&empty_vec),
            None => &empty_vec,
        };

        let mut manually_created_transcripts = HashMap::new();
        let mut generated_transcripts = HashMap::new();

        for caption in caption_tracks {
            let is_asr = caption
                .get("kind")
                .and_then(|k| k.as_str())
                .map(|k| k == "asr")
                .unwrap_or(false);

            let language_code = match caption.get("languageCode").and_then(|lc| lc.as_str()) {
                Some(code) => code.to_string(),
                None => continue,
            };

            let base_url = match caption.get("baseUrl").and_then(|url| url.as_str()) {
                Some(url) => url.to_string(),
                None => continue,
            };

            let name = match caption
                .get("name")
                .and_then(|n| n.get("simpleText"))
                .and_then(|st| st.as_str())
            {
                Some(name) => name.to_string(),
                None => continue,
            };

            let is_translatable = caption
                .get("isTranslatable")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);

            let tl = if is_translatable {
                translation_languages.clone()
            } else {
                vec![]
            };

            let transcript = Transcript::new(
                video_id.clone(),
                base_url,
                name,
                language_code.clone(),
                is_asr,
                tl,
            );

            if is_asr {
                generated_transcripts.insert(language_code, transcript);
            } else {
                manually_created_transcripts.insert(language_code, transcript);
            }
        }

        Ok(TranscriptList::new(
            video_id,
            manually_created_transcripts,
            generated_transcripts,
            translation_languages,
        ))
    }

    /// Finds a transcript matching one of the specified language codes.
    ///
    /// This method searches for transcripts in the order of priority:
    /// 1. Manually created transcripts with the specified language codes (in order)
    /// 2. Automatically generated transcripts with the specified language codes (in order)
    ///
    /// # Parameters
    ///
    /// * `language_codes` - Array of language codes to search for, in order of preference
    ///
    /// # Returns
    ///
    /// * `Result<Transcript, CouldNotRetrieveTranscript>` - Matching transcript or an error
    ///
    /// # Errors
    ///
    /// Returns an error if no transcript is found for any of the specified language codes.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// let transcript_list = api.list_transcripts("dQw4w9WgXcQ").await?;
    ///
    /// // Try to find English, fall back to Spanish, then auto-generated English
    /// let transcript = transcript_list.find_transcript(&["en", "es", "en-US"])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_transcript(
        &self,
        language_codes: &[&str],
    ) -> Result<Transcript, CouldNotRetrieveTranscript> {
        self.find_transcript_in_maps(
            language_codes,
            &[
                &self.manually_created_transcripts,
                &self.generated_transcripts,
            ],
        )
    }

    /// Finds a manually created transcript matching one of the specified language codes.
    ///
    /// This method only searches the manually created transcripts, skipping any
    /// automatically generated ones. This is useful when you want to ensure you're
    /// getting a human-created transcript for better accuracy.
    ///
    /// # Parameters
    ///
    /// * `language_codes` - Array of language codes to search for, in order of preference
    ///
    /// # Returns
    ///
    /// * `Result<Transcript, CouldNotRetrieveTranscript>` - Matching transcript or an error
    ///
    /// # Errors
    ///
    /// Returns an error if no manually created transcript is found for any of the
    /// specified language codes.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// let transcript_list = api.list_transcripts("dQw4w9WgXcQ").await?;
    ///
    /// // Only look for manually created transcripts
    /// match transcript_list.find_manually_created_transcript(&["en"]) {
    ///     Ok(transcript) => {
    ///         println!("Found manual transcript!");
    ///     },
    ///     Err(_) => {
    ///         println!("No manual transcript available, falling back to auto-generated");
    ///         let auto_transcript = transcript_list.find_generated_transcript(&["en"])?;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_manually_created_transcript(
        &self,
        language_codes: &[&str],
    ) -> Result<Transcript, CouldNotRetrieveTranscript> {
        self.find_transcript_in_maps(language_codes, &[&self.manually_created_transcripts])
    }

    /// Finds an automatically generated transcript matching one of the specified language codes.
    ///
    /// This method only searches the automatically generated transcripts, skipping any
    /// manually created ones. This might be useful in rare cases where you specifically
    /// want the auto-generated version.
    ///
    /// # Parameters
    ///
    /// * `language_codes` - Array of language codes to search for, in order of preference
    ///
    /// # Returns
    ///
    /// * `Result<Transcript, CouldNotRetrieveTranscript>` - Matching transcript or an error
    ///
    /// # Errors
    ///
    /// Returns an error if no automatically generated transcript is found for any of the
    /// specified language codes.
    pub fn find_generated_transcript(
        &self,
        language_codes: &[&str],
    ) -> Result<Transcript, CouldNotRetrieveTranscript> {
        self.find_transcript_in_maps(language_codes, &[&self.generated_transcripts])
    }

    /// Helper method to find a transcript in multiple transcript maps.
    ///
    /// This internal method is used by the public transcript finding methods to search
    /// through the provided maps of transcripts for the first match with the specified
    /// language codes.
    ///
    /// # Parameters
    ///
    /// * `language_codes` - Array of language codes to search for, in order of preference
    /// * `transcript_maps` - Array of transcript maps to search through, in order of priority
    ///
    /// # Returns
    ///
    /// * `Result<Transcript, CouldNotRetrieveTranscript>` - Matching transcript or an error
    ///
    /// # Errors
    ///
    /// Returns an error if no transcript is found for any of the specified language codes
    /// in any of the provided transcript maps.
    fn find_transcript_in_maps(
        &self,
        language_codes: &[&str],
        transcript_maps: &[&HashMap<String, Transcript>],
    ) -> Result<Transcript, CouldNotRetrieveTranscript> {
        for lang_code in language_codes {
            for transcript_map in transcript_maps {
                if let Some(transcript) = transcript_map.get(*lang_code) {
                    return Ok(transcript.clone());
                }
            }
        }

        Err(CouldNotRetrieveTranscript {
            video_id: self.video_id.clone(),
            reason: Some(CouldNotRetrieveTranscriptReason::NoTranscriptFound {
                requested_language_codes: language_codes.iter().map(|&s| s.to_string()).collect(),
                transcript_data: self.clone(),
            }),
        })
    }

    /// Returns a reference to all available transcripts.
    ///
    /// This method provides access to both manually created and automatically generated
    /// transcripts as an iterator.
    ///
    /// # Returns
    ///
    /// An iterator over references to all available transcripts.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use yt_transcript_rs::YouTubeTranscriptApi;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let api = YouTubeTranscriptApi::new(None, None, None)?;
    /// let transcript_list = api.list_transcripts("dQw4w9WgXcQ").await?;
    ///
    /// // Print info about all available transcripts
    /// for transcript in transcript_list.transcripts() {
    ///     println!("Language: {} ({}), Auto-generated: {}",
    ///         transcript.language(),
    ///         transcript.language_code(),
    ///         transcript.is_generated());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn transcripts(&self) -> impl Iterator<Item = &Transcript> {
        self.into_iter()
    }
}

impl fmt::Display for TranscriptList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut transcript_strings = Vec::new();

        // Add manually created transcripts
        for transcript in self.manually_created_transcripts.values() {
            transcript_strings.push(format!("{}", transcript));
        }

        // Add generated transcripts
        for transcript in self.generated_transcripts.values() {
            transcript_strings.push(format!("{}", transcript));
        }

        // Format the output
        let language_desc = if transcript_strings.is_empty() {
            "No transcripts found".to_string()
        } else {
            format!("Available transcripts: {}", transcript_strings.join(", "))
        };

        write!(f, "{}", language_desc)
    }
}

impl IntoIterator for TranscriptList {
    type Item = Transcript;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut transcripts = Vec::new();
        transcripts.extend(self.manually_created_transcripts.into_values());
        transcripts.extend(self.generated_transcripts.into_values());
        transcripts.into_iter()
    }
}

impl<'a> IntoIterator for &'a TranscriptList {
    type Item = &'a Transcript;
    type IntoIter = std::iter::Chain<
        std::iter::Map<
            std::collections::hash_map::Values<'a, String, Transcript>,
            fn(&'a Transcript) -> &'a Transcript,
        >,
        std::iter::Map<
            std::collections::hash_map::Values<'a, String, Transcript>,
            fn(&'a Transcript) -> &'a Transcript,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        fn id(t: &Transcript) -> &Transcript {
            t
        }
        self.manually_created_transcripts
            .values()
            .map(id as _)
            .chain(self.generated_transcripts.values().map(id as _))
    }
}
