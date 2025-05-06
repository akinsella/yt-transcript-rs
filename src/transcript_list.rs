use reqwest::Client;
use std::collections::HashMap;
use std::fmt;

use crate::errors::{
    CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason,
};
use crate::models::TranslationLanguage;
use crate::transcript::Transcript;

/// A collection of available transcripts for a video
#[derive(Debug, Clone)]
pub struct TranscriptList {
    pub video_id: String,
    pub manually_created_transcripts: HashMap<String, Transcript>,
    pub generated_transcripts: HashMap<String, Transcript>,
    pub translation_languages: Vec<TranslationLanguage>,
}

impl TranscriptList {
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

    /// Create a TranscriptList from YouTube caption data
    pub fn build(
        client: Client,
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
                client.clone(),
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

    /// Find a transcript by language code
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

    /// Find a manually created transcript
    pub fn find_manually_created_transcript(
        &self,
        language_codes: &[&str],
    ) -> Result<Transcript, CouldNotRetrieveTranscript> {
        self.find_transcript_in_maps(language_codes, &[&self.manually_created_transcripts])
    }

    /// Find an automatically generated transcript
    pub fn find_generated_transcript(
        &self,
        language_codes: &[&str],
    ) -> Result<Transcript, CouldNotRetrieveTranscript> {
        self.find_transcript_in_maps(language_codes, &[&self.generated_transcripts])
    }

    /// Helper to find a transcript in multiple maps
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
        fn id<'a>(t: &'a Transcript) -> &'a Transcript {
            t
        }
        self.manually_created_transcripts
            .values()
            .map(id as _)
            .chain(self.generated_transcripts.values().map(id as _))
    }
}
