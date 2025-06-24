#[allow(unused_imports)]
use super::test_utils::{create_api, setup, MULTILANG_VIDEO_ID, NON_EXISTENT_VIDEO_ID};

#[allow(unused_imports)]
use crate::errors::CouldNotRetrieveTranscriptReason;

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_list_transcripts() {
    setup();
    let api = create_api();

    // Test with a known video that has multiple languages
    let transcript_list = api.list_transcripts(MULTILANG_VIDEO_ID).await;

    // Check if we got a valid result or a specific error
    if let Ok(transcript_list) = transcript_list {
        assert_eq!(transcript_list.video_id, MULTILANG_VIDEO_ID);

        // Ensure we can iterate over the transcript list
        let mut found_transcripts = false;
        for transcript in &transcript_list {
            // We should have at least one transcript
            found_transcripts = true;
            // Each transcript should have the same video ID
            assert_eq!(transcript.video_id, MULTILANG_VIDEO_ID);
            // Language code should not be empty
            assert!(!transcript.language_code.is_empty());
            // Language name should not be empty
            assert!(!transcript.language.is_empty());
        }

        assert!(found_transcripts, "No transcripts found in the list");
    } else {
        let error = transcript_list.unwrap_err();
        // The API can fail with YouTubeDataUnparsable, which is acceptable for tests
        // since YouTube regularly changes their API
        match error.reason {
            Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) => {
                // This is an acceptable error, test passes
            }
            _ => {
                panic!("Unexpected error: {:?}", error);
            }
        }
    }
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_find_transcript() {
    setup();
    let api = create_api();

    // Get the list of transcripts
    let transcript_list_result = api.list_transcripts(MULTILANG_VIDEO_ID).await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &transcript_list_result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            return; // Skip test
        }
    }

    let transcript_list = match transcript_list_result {
        Ok(list) => list,
        Err(e) => panic!("Failed to get transcript list: {:?}", e),
    };

    // Try to find an English transcript
    let transcript = transcript_list.find_transcript(&["en"]);
    assert!(transcript.is_ok(), "Failed to find English transcript");

    // Try to find with fallback languages
    let transcript = transcript_list.find_transcript(&["non-existent", "en"]);
    assert!(
        transcript.is_ok(),
        "Failed to find transcript with fallback languages"
    );

    // Try to find a non-existent language
    let transcript = transcript_list.find_transcript(&["non-existent"]);
    assert!(transcript.is_err(), "Found a non-existent transcript");
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_fetch_transcript() {
    setup();
    let api = create_api();

    // Test fetching an English transcript
    let result = api
        .fetch_transcript(MULTILANG_VIDEO_ID, &["en"], false)
        .await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            return; // Skip test
        }
    }

    assert!(result.is_ok(), "Failed to fetch English transcript");

    let transcript = result.unwrap();
    assert_eq!(transcript.video_id, MULTILANG_VIDEO_ID);
    assert!(
        !transcript.snippets.is_empty(),
        "Transcript has no snippets"
    );

    // Each snippet should have text and timing information
    for snippet in transcript.snippets {
        assert!(!snippet.text.is_empty(), "Snippet has empty text");
        assert!(snippet.start >= 0.0, "Snippet has negative start time");
        assert!(
            snippet.duration > 0.0,
            "Snippet has zero or negative duration"
        );
    }
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_translate_transcript() {
    setup();
    let api = create_api();

    // Get a transcript list
    let transcript_list_result = api.list_transcripts(MULTILANG_VIDEO_ID).await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &transcript_list_result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            return; // Skip test
        }
    }

    let transcript_list = match transcript_list_result {
        Ok(list) => list,
        Err(e) => panic!("Failed to get transcript list: {:?}", e),
    };

    // Find a transcript that supports translation
    let mut found_translatable = false;
    for transcript in &transcript_list {
        if transcript.is_translatable() {
            found_translatable = true;

            // Try to translate to French
            if transcript
                .translation_languages
                .iter()
                .any(|lang| lang.language_code == "fr")
            {
                let translated = transcript.translate("fr").unwrap();
                assert_eq!(translated.language_code, "fr");
                break;
            }
        }
    }

    assert!(found_translatable, "No translatable transcripts found");
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_fetch_non_existent_video() {
    setup();
    let api = create_api();

    // Test fetching a non-existent video
    let result = api.list_transcripts(NON_EXISTENT_VIDEO_ID).await;
    assert!(result.is_err(), "Successfully fetched non-existent video");

    // Check the error type
    let error = result.unwrap_err();
    match error.reason {
        Some(CouldNotRetrieveTranscriptReason::VideoUnavailable)
        | Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) => {
            // These are acceptable errors
        }
        _ => panic!("Unexpected error type: {:?}", error),
    }
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_transcriptlist() {
    setup();
    let api = create_api();

    // Get the list of transcripts
    let transcript_list_result = api.list_transcripts(MULTILANG_VIDEO_ID).await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &transcript_list_result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            // Skip test
            return;
        }
        panic!("Failed to get transcript list: {:?}", error);
    }

    let transcript_list = transcript_list_result.unwrap();

    // Check that the string representation is not empty
    let transcript_list_str = format!("{}", transcript_list);
    assert!(!transcript_list_str.is_empty());
    assert!(transcript_list_str.contains("Available transcripts"));
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_transcript_formatter() {
    setup();
    let api = create_api();

    // Get the list of transcripts
    let transcript_list_result = api.list_transcripts(MULTILANG_VIDEO_ID).await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &transcript_list_result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            // Skip test
            return;
        }
        panic!("Failed to get transcript list: {:?}", error);
    }

    let transcript_list = transcript_list_result.unwrap();

    // Try to find a transcript
    let transcript_result = transcript_list.find_transcript(&["en"]);

    if transcript_result.is_err() {
        // This might happen if the transcript list doesn't have English
        return;
    }

    let transcript = transcript_result.unwrap();

    // Check that the string representation is not empty
    let transcript_str = format!("{}", transcript);
    assert!(!transcript_str.is_empty());
    assert!(transcript_str.contains("en"));
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_preserve_formatting() {
    setup();
    let api = create_api();

    // Test fetching a transcript with formatting preserved
    let with_formatting_result = api
        .fetch_transcript(MULTILANG_VIDEO_ID, &["en"], true)
        .await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &with_formatting_result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            return; // Skip test
        }
    }

    let with_formatting = match with_formatting_result {
        Ok(transcript) => transcript,
        Err(e) => panic!("Failed to fetch transcript with formatting: {:?}", e),
    };

    // Test fetching the same transcript without formatting
    let without_formatting_result = api
        .fetch_transcript(MULTILANG_VIDEO_ID, &["en"], false)
        .await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &without_formatting_result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            return; // Skip test
        }
    }

    let without_formatting = match without_formatting_result {
        Ok(transcript) => transcript,
        Err(e) => panic!("Failed to fetch transcript without formatting: {:?}", e),
    };

    // Compare if they are different in some way
    // Note: This may not always be true if the transcript has no formatting
    // So this is more of a smoke test
    assert_eq!(with_formatting.video_id, without_formatting.video_id);
    assert_eq!(with_formatting.language, without_formatting.language);
    assert_eq!(
        with_formatting.language_code,
        without_formatting.language_code
    );
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_fetch_microformat() {
    setup();
    let api = create_api();

    // Test fetching microformat data
    let result = api.fetch_microformat(MULTILANG_VIDEO_ID).await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            return; // Skip test
        }
    }

    assert!(result.is_ok(), "Failed to fetch microformat data");

    let microformat = result.unwrap();

    // Verify that we have some basic data
    if let Some(countries) = &microformat.available_countries {
        assert!(!countries.is_empty(), "Available countries list is empty");
    }

    if let Some(external_video_id) = &microformat.external_video_id {
        assert_eq!(external_video_id, MULTILANG_VIDEO_ID, "Video ID mismatch");
    }

    // Check if we have category information
    if let Some(category) = &microformat.category {
        assert!(!category.is_empty(), "Category is empty");
    }

    // Check if we have embed information
    if let Some(embed) = &microformat.embed {
        if let Some(iframe_url) = &embed.iframe_url {
            assert!(
                iframe_url.contains(MULTILANG_VIDEO_ID),
                "Embed URL doesn't contain video ID"
            );
        }
    }
}

#[cfg(not(feature = "ci"))]
#[tokio::test]
async fn test_fetch_streaming_data() {
    setup();
    let api = create_api();

    // Test fetching streaming data
    let result = api.fetch_streaming_data(MULTILANG_VIDEO_ID).await;

    // If YouTube API returns an error, skip this test
    if let Err(error) = &result {
        if let Some(CouldNotRetrieveTranscriptReason::YouTubeDataUnparsable(_)) = error.reason {
            return; // Skip test
        }
    }

    assert!(result.is_ok(), "Failed to fetch streaming data");

    let streaming_data = result.unwrap();

    // Verify that we have some basic data
    assert!(!streaming_data.formats.is_empty(), "Formats list is empty");
    assert!(
        !streaming_data.adaptive_formats.is_empty(),
        "Adaptive formats list is empty"
    );
    assert!(
        !streaming_data.expires_in_seconds.is_empty(),
        "Expiration time is empty"
    );

    // Check some format details
    for format in &streaming_data.formats {
        assert!(format.itag > 0, "Invalid itag value");
        assert!(!format.mime_type.is_empty(), "MIME type is empty");
        assert!(format.bitrate > 0, "Bitrate is zero or negative");
        assert!(!format.approx_duration_ms.is_empty(), "Duration is empty");
    }

    // Check some adaptive format details
    for format in &streaming_data.adaptive_formats {
        assert!(format.itag > 0, "Invalid itag value");
        assert!(!format.mime_type.is_empty(), "MIME type is empty");
        assert!(format.bitrate > 0, "Bitrate is zero or negative");
        assert!(!format.approx_duration_ms.is_empty(), "Duration is empty");

        // Check for video-specific properties
        if format.mime_type.starts_with("video/") {
            assert!(format.width.is_some(), "Video width is missing");
            assert!(format.height.is_some(), "Video height is missing");
        }

        // Check for audio-specific properties
        if format.mime_type.starts_with("audio/") {
            assert!(format.audio_quality.is_some(), "Audio quality is missing");
        }
    }
}
