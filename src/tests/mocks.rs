use reqwest::Client;
use std::collections::HashMap;

use serde_json::json;

use crate::fetched_transcript::FetchedTranscript;
use crate::models::{
    FetchedTranscriptSnippet, MicroformatData, MicroformatEmbed, MicroformatThumbnail,
    TranslationLanguage, VideoThumbnail,
};
use crate::transcript::Transcript;
use crate::transcript_list::TranscriptList;
use crate::VideoDetails;

// Mock video IDs that match the ones used in tests
pub const MOCK_MULTILANG_VIDEO_ID: &str = "arj7oStGLkU";
pub const MOCK_NON_EXISTENT_VIDEO_ID: &str = "xxxxxxxxxxx";

// Create mock transcript data
pub fn create_mock_transcript_list(client: Client) -> TranscriptList {
    let video_id = MOCK_MULTILANG_VIDEO_ID.to_string();

    // Create translation languages
    let translation_languages = vec![
        TranslationLanguage {
            language: "English".to_string(),
            language_code: "en".to_string(),
        },
        TranslationLanguage {
            language: "French".to_string(),
            language_code: "fr".to_string(),
        },
        TranslationLanguage {
            language: "Spanish".to_string(),
            language_code: "es".to_string(),
        },
    ];

    // Create manually created transcripts
    let mut manually_created_transcripts = HashMap::new();
    let english_transcript = Transcript::new(
        client.clone(),
        video_id.clone(),
        "https://mock.url/en".to_string(),
        "English".to_string(),
        "en".to_string(),
        false,
        translation_languages.clone(),
    );
    manually_created_transcripts.insert("en".to_string(), english_transcript);

    // Create generated transcripts
    let mut generated_transcripts = HashMap::new();
    let spanish_transcript = Transcript::new(
        client.clone(),
        video_id.clone(),
        "https://mock.url/es".to_string(),
        "Spanish".to_string(),
        "es".to_string(),
        true,
        vec![],
    );
    generated_transcripts.insert("es".to_string(), spanish_transcript);

    TranscriptList::new(
        video_id,
        manually_created_transcripts,
        generated_transcripts,
        translation_languages,
    )
}

// Create a mock fetched transcript
pub fn create_mock_fetched_transcript(video_id: &str, language_code: &str) -> FetchedTranscript {
    let snippets = vec![
        FetchedTranscriptSnippet {
            text: "Hello, this is a test transcript.".to_string(),
            start: 0.0,
            duration: 2.5,
        },
        FetchedTranscriptSnippet {
            text: "Welcome to the mock implementation.".to_string(),
            start: 2.5,
            duration: 3.0,
        },
        FetchedTranscriptSnippet {
            text: "This helps avoid hitting YouTube's API in tests.".to_string(),
            start: 5.5,
            duration: 4.0,
        },
    ];

    let language = match language_code {
        "en" => "English",
        "fr" => "French",
        "es" => "Spanish",
        _ => "Unknown",
    };

    FetchedTranscript {
        snippets,
        video_id: video_id.to_string(),
        language: language.to_string(),
        language_code: language_code.to_string(),
        is_generated: language_code != "en",
    }
}

// Mock JSON data that we'd normally get from YouTube
pub fn mock_youtube_player_response() -> serde_json::Value {
    json!({
        "playabilityStatus": {
            "status": "OK"
        },
        "captions": {
            "playerCaptionsTracklistRenderer": {
                "captionTracks": [
                    {
                        "baseUrl": "https://mock.url/en",
                        "name": {"simpleText": "English"},
                        "languageCode": "en",
                        "kind": "manual",
                        "isTranslatable": true
                    },
                    {
                        "baseUrl": "https://mock.url/es",
                        "name": {"simpleText": "Spanish"},
                        "languageCode": "es",
                        "kind": "asr",
                        "isTranslatable": false
                    }
                ],
                "translationLanguages": [
                    {
                        "languageCode": "en",
                        "languageName": {"simpleText": "English"}
                    },
                    {
                        "languageCode": "fr",
                        "languageName": {"simpleText": "French"}
                    },
                    {
                        "languageCode": "es",
                        "languageName": {"simpleText": "Spanish"}
                    }
                ]
            }
        }
    })
}

// Mock transcript data that we'd get from a transcript URL
pub fn mock_transcript_data() -> serde_json::Value {
    json!([
        {
            "text": "Hello, this is a test transcript.",
            "start": 0.0,
            "duration": 2.5
        },
        {
            "text": "Welcome to the mock implementation.",
            "start": 2.5,
            "duration": 3.0
        },
        {
            "text": "This helps avoid hitting YouTube's API in tests.",
            "start": 5.5,
            "duration": 4.0
        }
    ])
}

// Create a mock video details
pub fn create_mock_video_details() -> VideoDetails {
    VideoDetails {
        video_id: MOCK_MULTILANG_VIDEO_ID.to_string(),
        title: "Test Video".to_string(),
        author: "Test Author".to_string(),
        length_seconds: 100,
        keywords: Some(vec!["test".to_string(), "mock".to_string()]),
        channel_id: "test-channel".to_string(),
        short_description: "Test description".to_string(),
        view_count: "100".to_string(),
        thumbnails: vec![],
        is_live_content: false,
    }
}

// Create a mock HTTP client for tests
pub fn create_mock_client() -> Client {
    // In real implementation, we'd use a more sophisticated HTTP mocking library
    // For this example, we're keeping it simple
    Client::builder().user_agent("Mock Client").build().unwrap()
}

/// Creates mock microformat data for testing
pub fn create_mock_microformat_data() -> MicroformatData {
    let thumbnails = vec![VideoThumbnail {
        url: "https://i.ytimg.com/vi/mock_video_id/maxresdefault.jpg".to_string(),
        width: 1280,
        height: 720,
    }];

    let thumbnail = MicroformatThumbnail {
        thumbnails: Some(thumbnails),
    };

    let embed = MicroformatEmbed {
        height: Some(720),
        iframe_url: Some("https://www.youtube.com/embed/mock_video_id".to_string()),
        width: Some(1280),
    };

    let available_countries = vec![
        "US".to_string(),
        "GB".to_string(),
        "CA".to_string(),
        "AU".to_string(),
        "DE".to_string(),
        "FR".to_string(),
        "JP".to_string(),
    ];

    MicroformatData {
        available_countries: Some(available_countries),
        category: Some("Science & Technology".to_string()),
        description: Some("This is a mock video description for testing.".to_string()),
        embed: Some(embed),
        external_channel_id: Some("UC_mock_channel_id".to_string()),
        external_video_id: Some("mock_video_id".to_string()),
        has_ypc_metadata: Some(false),
        is_family_safe: Some(true),
        is_shorts_eligible: Some(false),
        is_unlisted: Some(false),
        length_seconds: Some("300".to_string()),
        like_count: Some("1000".to_string()),
        owner_channel_name: Some("Mock Channel".to_string()),
        owner_profile_url: Some("https://www.youtube.com/@MockChannel".to_string()),
        publish_date: Some("2023-01-01T12:00:00Z".to_string()),
        thumbnail: Some(thumbnail),
        title: Some("Mock Video Title".to_string()),
        upload_date: Some("2023-01-01T12:00:00Z".to_string()),
        view_count: Some("10000".to_string()),
    }
}
