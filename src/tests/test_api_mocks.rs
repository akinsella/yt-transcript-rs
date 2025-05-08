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
