use reqwest::Client;
use std::collections::HashMap;
use std::fmt;

use crate::errors::{
    CouldNotRetrieveTranscript, CouldNotRetrieveTranscriptReason,
};
use crate::fetched_transcript::FetchedTranscript;
use crate::transcript_parser::TranscriptParser;
use crate::models::TranslationLanguage;
use std::fs;

/// Represents a transcript that can be fetched
#[derive(Debug, Clone)]
pub struct Transcript {
    pub client: Client,
    pub video_id: String,
    pub url: String,
    pub language: String,
    pub language_code: String,
    pub is_generated: bool,
    pub translation_languages: Vec<TranslationLanguage>,
    pub translation_languages_map: HashMap<String, String>,
}

impl Transcript {
    /// Create a new transcript
    pub fn new(
        client: Client,
        video_id: String,
        url: String,
        language: String,
        language_code: String,
        is_generated: bool,
        translation_languages: Vec<TranslationLanguage>,
    ) -> Self {
        let translation_languages_map = translation_languages
            .iter()
            .map(|lang| (lang.language_code.clone(), lang.language.clone()))
            .collect();

        Self {
            client,
            video_id,
            url,
            language,
            language_code,
            is_generated,
            translation_languages,
            translation_languages_map,
        }
    }

    /// Fetch the actual transcript data
    pub async fn fetch(
        &self,
        preserve_formatting: bool,
    ) -> Result<FetchedTranscript, CouldNotRetrieveTranscript> {
        let response =
            self.client
                .get(&self.url)
                .send()
                .await
                .map_err(|e| CouldNotRetrieveTranscript {
                    video_id: self.video_id.clone(),
                    reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                        e.to_string(),
                    )),
                })?;

        if response.status() != reqwest::StatusCode::OK {
            return Err(CouldNotRetrieveTranscript {
                video_id: self.video_id.clone(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    format!("YouTube returned status code: {}", response.status()),
                )),
            });
        }

        let text = response
            .text()
            .await
            .map_err(|e| CouldNotRetrieveTranscript {
                video_id: self.video_id.clone(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeRequestFailed(
                    e.to_string(),
                )),
            })?;

        fs::write("/Users/alexis.kinsella/Desktop/transcript_data.txt", text.clone()).unwrap();

        let snippets = TranscriptParser::new(preserve_formatting)
            .parse(&text.clone())
            .map_err(|_| CouldNotRetrieveTranscript {
                video_id: self.video_id.clone(),
                reason: Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable),
            })?;

        fs::write(
            "/Users/alexis.kinsella/Desktop/transcript_snippets.txt",
            format!("{:?}", snippets),
        )
        .unwrap();

        Ok(FetchedTranscript {
            snippets,
            video_id: self.video_id.clone(),
            language: self.language.clone(),
            language_code: self.language_code.clone(),
            is_generated: self.is_generated,
        })
    }

    /// Check if this transcript is translatable
    pub fn is_translatable(&self) -> bool {
        !self.translation_languages.is_empty()
    }

    /// Translate this transcript to another language
    pub fn translate(&self, language_code: &str) -> Result<Self, CouldNotRetrieveTranscript> {
        if !self.is_translatable() {
            return Err(CouldNotRetrieveTranscript {
                video_id: self.video_id.clone(),
                reason: Some(CouldNotRetrieveTranscriptReason::NotTranslatable),
            });
        }

        if !self.translation_languages_map.contains_key(language_code) {
            return Err(CouldNotRetrieveTranscript {
                video_id: self.video_id.clone(),
                reason: Some(CouldNotRetrieveTranscriptReason::TranslationLanguageNotAvailable),
            });
        }

        let language = self
            .translation_languages_map
            .get(language_code)
            .unwrap()
            .clone();
        let url = format!("{}&tlang={}", self.url, language_code);

        Ok(Transcript::new(
            self.client.clone(),
            self.video_id.clone(),
            url,
            language,
            language_code.to_string(),
            true,
            vec![],
        ))
    }
}

impl fmt::Display for Transcript {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let translation_desc = if self.is_translatable() {
            "[TRANSLATABLE]"
        } else {
            ""
        };
        write!(
            f,
            "{} ({}){}",
            self.language_code, self.language, translation_desc
        )
    }
}
