#[allow(unused_imports)]
use super::mocks::{
    create_mock_fetched_transcript, create_mock_transcript_list, mock_youtube_player_response,
};
#[allow(unused_imports)]
use super::test_utils::{create_api, setup, MULTILANG_VIDEO_ID, NON_EXISTENT_VIDEO_ID};

// #[cfg(feature = "ci")]
#[tokio::test]
async fn test_list_transcripts() {
    setup();
    let api = create_api();

    // In CI mode, we'll mock the response
    let transcript_list = api.list_transcripts(MULTILANG_VIDEO_ID).await;

    assert!(transcript_list.is_ok(), "Failed to get transcript list");
    let transcript_list = transcript_list.unwrap();

    // Verify the mock data
    assert_eq!(transcript_list.video_id, MULTILANG_VIDEO_ID);

    // Verify we can iterate over transcripts
    let mut found_transcripts = false;
    for transcript in &transcript_list {
        found_transcripts = true;
        assert_eq!(transcript.video_id, MULTILANG_VIDEO_ID);
        assert!(!transcript.language_code.is_empty());
        assert!(!transcript.language.is_empty());
    }

    assert!(found_transcripts, "No transcripts found in the list");
}

// Similarly implement the other test methods with mocking...

#[cfg(feature = "ci")]
#[tokio::test]
async fn test_find_transcript() {
    setup();
    let api = create_api();

    let transcript_list = api.list_transcripts(MULTILANG_VIDEO_ID).await.unwrap();

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

// Implement the other tests similarly...

#[cfg(feature = "ci")]
#[tokio::test]
async fn test_fetch_microformat() {
    setup();
    let api = create_api();

    // Fetch microformat data
    let microformat = api.fetch_microformat(MULTILANG_VIDEO_ID).await;

    assert!(microformat.is_ok(), "Failed to fetch microformat data");
    let microformat = microformat.unwrap();

    // Verify the mock data
    // Check for available countries
    if let Some(countries) = microformat.available_countries {
        assert!(!countries.is_empty(), "Available countries list is empty");
    } else {
        panic!("Available countries is None");
    }

    // Check for category
    if let Some(category) = microformat.category {
        assert_eq!(
            category, "Science & Technology",
            "Category doesn't match mock data"
        );
    } else {
        panic!("Category is None");
    }

    // Check for family safe flag
    if let Some(is_family_safe) = microformat.is_family_safe {
        assert!(is_family_safe, "Family safe flag doesn't match mock data");
    } else {
        panic!("Family safe flag is None");
    }

    // Test non-existent video
    let result = api.fetch_microformat(NON_EXISTENT_VIDEO_ID).await;
    assert!(result.is_err(), "Successfully fetched non-existent video");
}

#[cfg(feature = "ci")]
#[tokio::test]
async fn test_fetch_streaming_data() {
    setup();
    let api = create_api();

    // Fetch streaming data
    let streaming_data = api.fetch_streaming_data(MULTILANG_VIDEO_ID).await;

    assert!(streaming_data.is_ok(), "Failed to fetch streaming data");
    let streaming_data = streaming_data.unwrap();

    // Verify the mock data
    // Check formats
    assert!(!streaming_data.formats.is_empty(), "Formats list is empty");
    let format = &streaming_data.formats[0];
    assert_eq!(format.itag, 18, "Format itag doesn't match mock data");
    assert_eq!(
        format.quality, "medium",
        "Format quality doesn't match mock data"
    );
    assert_eq!(
        format.width,
        Some(640),
        "Format width doesn't match mock data"
    );
    assert_eq!(
        format.height,
        Some(360),
        "Format height doesn't match mock data"
    );

    // Check adaptive formats
    assert!(
        !streaming_data.adaptive_formats.is_empty(),
        "Adaptive formats list is empty"
    );

    // Find a video format
    let video_format = streaming_data
        .adaptive_formats
        .iter()
        .find(|f| f.mime_type.starts_with("video/"))
        .expect("No video format in adaptive formats");

    assert!(video_format.width.is_some(), "Video format has no width");
    assert!(video_format.height.is_some(), "Video format has no height");

    // Find an audio format
    let audio_format = streaming_data
        .adaptive_formats
        .iter()
        .find(|f| f.mime_type.starts_with("audio/"))
        .expect("No audio format in adaptive formats");

    assert!(
        audio_format.audio_quality.is_some(),
        "Audio format has no audio quality"
    );

    // Check expiration
    assert!(
        !streaming_data.expires_in_seconds.is_empty(),
        "Expiration time is empty"
    );

    // Test non-existent video
    let result = api.fetch_streaming_data(NON_EXISTENT_VIDEO_ID).await;
    assert!(result.is_err(), "Successfully fetched non-existent video");
}
