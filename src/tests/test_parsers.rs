#[allow(unused_imports)]
use serde_json::json;

#[allow(unused_imports)]
use crate::{js_var_parser::JsVarParser, transcript_parser::TranscriptParser};

/// Test video ID used for tests
#[allow(dead_code)]
const TEST_VIDEO_ID: &str = "arj7oStGLkU";

#[test]
fn test_js_var_parser() {
    let parser = JsVarParser::new("testVar");

    // Test HTML with a JavaScript variable
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title></head>
    <body>
        <script>
        var testVar = {"key1": "value1", "key2": {"nested": "value2"}, "array": [1, 2, 3]};
        </script>
    </body>
    </html>
    "#;

    let result = parser.parse(html, "test").unwrap();

    // Verify the parsed JSON
    assert_eq!(result["key1"], json!("value1"));
    assert_eq!(result["key2"]["nested"], json!("value2"));
    assert_eq!(result["array"], json!([1, 2, 3]));
}

#[test]
fn test_js_var_parser_with_complex_json() {
    let parser = JsVarParser::new("complexVar");

    // Test with a more complex JSON including escaped quotes
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title></head>
    <body>
        <script>
        var complexVar = {
            "text": "This has \"quotes\" inside",
            "nested": {
                "object": {"with": "more nesting"},
                "array": [{"item1": 1}, {"item2": 2}]
            },
            "escaped": "Line1\nLine2\\nLine3"
        };
        </script>
    </body>
    </html>
    "#;

    let result = parser.parse(html, "test").unwrap();

    // Verify the parsed JSON
    assert_eq!(result["text"], json!("This has \"quotes\" inside"));
    assert_eq!(result["nested"]["object"]["with"], json!("more nesting"));
    assert_eq!(result["nested"]["array"][0]["item1"], json!(1));
    assert_eq!(result["nested"]["array"][1]["item2"], json!(2));
    assert_eq!(result["escaped"], json!("Line1\nLine2\\nLine3"));
}

#[test]
fn test_js_var_parser_with_alternate_format() {
    let parser = JsVarParser::new("altVar");

    // Test with alternate variable assignment format (no spaces, different terminator)
    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <script>
        altVar={"simple": "value"};
        doSomething();
        </script>
    </body>
    </html>
    "#;

    let result = parser.parse(html, "test").unwrap();

    // Verify the parsed JSON
    assert_eq!(result["simple"], json!("value"));
}

/// Test the extraction of microformat data from a simplified JSON
#[tokio::test]
async fn test_microformat_extraction_simple() {
    use crate::microformat_extractor::MicroformatExtractor;
    use serde_json::json;

    // Create a minimal player response with just the essential fields
    let player_response = json!({
        "microformat": {
            "playerMicroformatRenderer": {
                "externalVideoId": TEST_VIDEO_ID,
                "title": {
                    "simpleText": "Test Video"
                }
            }
        }
    });

    // Extract microformat data using the extractor
    let result = MicroformatExtractor::extract_microformat_data(&player_response, TEST_VIDEO_ID);

    // Print the JSON for debugging
    println!(
        "Player response JSON: {}",
        serde_json::to_string_pretty(&player_response).unwrap()
    );

    // Verify the result
    assert!(
        result.is_ok(),
        "Failed to extract microformat data: {:?}",
        result.err()
    );

    let microformat = result.unwrap();

    // Verify basic fields
    assert_eq!(
        microformat.external_video_id,
        Some(TEST_VIDEO_ID.to_string())
    );
    assert_eq!(microformat.title, Some("Test Video".to_string()));
}

/// Test microformat extraction from embedded JSON data
#[tokio::test]
async fn test_microformat_extraction_from_file() {
    use crate::microformat_extractor::MicroformatExtractor;

    // Create minimal test JSON directly in the test
    let json_str = r#"{
        "microformat": {
            "playerMicroformatRenderer": {
                "title": {
                    "simpleText": "Inside the Mind of a Master Procrastinator | Tim Urban | TED"
                },
                "description": {
                    "simpleText": "Tim Urban knows that procrastination doesn't make sense, but he's never been able to shake his habit of waiting until the last minute to get things done."
                },
                "lengthSeconds": "844",
                "ownerChannelName": "TED",
                "externalChannelId": "UCAuUUnT6oDeKwE6v1NGQxug",
                "externalVideoId": "arj7oStGLkU",
                "viewCount": "58968839",
                "category": "People & Blogs",
                "publishDate": "2016-04-06T09:59:35-07:00",
                "uploadDate": "2016-04-06T09:59:35-07:00",
                "isFamilySafe": true,
                "availableCountries": ["US", "GB", "CA", "JP", "DE", "FR", "IT", "ES", "NL", "AU"],
                "isUnlisted": false,
                "hasYpcMetadata": false,
                "isShortsEligible": false,
                "embed": {
                    "iframeUrl": "https://www.youtube.com/embed/arj7oStGLkU",
                    "width": 1280,
                    "height": 720
                },
                "thumbnail": {
                    "thumbnails": [
                        {
                            "url": "https://i.ytimg.com/vi/arj7oStGLkU/maxresdefault.jpg",
                            "width": 1280,
                            "height": 720
                        }
                    ]
                }
            }
        }
    }"#;

    // Parse the JSON
    let player_response: serde_json::Value =
        serde_json::from_str(json_str).expect("Failed to parse JSON");

    // Extract microformat data using the extractor
    let result = MicroformatExtractor::extract_microformat_data(&player_response, TEST_VIDEO_ID);

    // Verify the result
    assert!(
        result.is_ok(),
        "Failed to extract microformat data: {:?}",
        result.err()
    );

    let microformat = result.unwrap();

    // Basic verification that the data was parsed
    assert_eq!(
        microformat.external_video_id,
        Some(TEST_VIDEO_ID.to_string())
    );
    assert_eq!(microformat.owner_channel_name, Some("TED".to_string()));

    // Check for available countries
    assert!(microformat.available_countries.is_some());
    let countries = microformat.available_countries.unwrap();
    assert!(!countries.is_empty());

    // Check title
    assert!(microformat.title.is_some());
    assert_eq!(
        microformat.title.unwrap(),
        "Inside the Mind of a Master Procrastinator | Tim Urban | TED"
    );
}

/// Test the extraction of microformat data with a full set of fields
#[tokio::test]
async fn test_microformat_extraction_comprehensive() {
    use crate::microformat_extractor::MicroformatExtractor;
    use serde_json::json;

    // Create a comprehensive player response with all fields
    let player_response = json!({
        "microformat": {
            "playerMicroformatRenderer": {
                "availableCountries": ["US", "GB", "CA", "JP", "DE", "FR"],
                "category": "People & Blogs",
                "description": {
                    "simpleText": "This is a test description"
                },
                "embed": {
                    "height": 720,
                    "iframeUrl": format!("https://www.youtube.com/embed/{}", TEST_VIDEO_ID),
                    "width": 1280
                },
                "externalChannelId": "UCAuUUnT6oDeKwE6v1NGQxug",
                "externalVideoId": TEST_VIDEO_ID,
                "hasYpcMetadata": false,
                "isFamilySafe": true,
                "isShortsEligible": false,
                "isUnlisted": false,
                "lengthSeconds": "844",
                "likeCount": "2009473",
                "ownerChannelName": "TED",
                "ownerProfileUrl": "http://www.youtube.com/@TED",
                "publishDate": "2016-04-06T09:59:35-07:00",
                "thumbnail": {
                    "thumbnails": [
                        {
                            "height": 720,
                            "url": format!("https://i.ytimg.com/vi/{}/maxresdefault.jpg", TEST_VIDEO_ID),
                            "width": 1280
                        }
                    ]
                },
                "title": {
                    "simpleText": "Inside the Mind of a Master Procrastinator | Tim Urban | TED"
                },
                "uploadDate": "2016-04-06T09:59:35-07:00",
                "viewCount": "58968839"
            }
        }
    });

    // Extract microformat data using the extractor
    let result = MicroformatExtractor::extract_microformat_data(&player_response, TEST_VIDEO_ID);

    // Verify the result
    assert!(
        result.is_ok(),
        "Failed to extract microformat data: {:?}",
        result.err()
    );

    let microformat = result.unwrap();

    // Verify specific fields from the known data
    assert_eq!(
        microformat.external_video_id,
        Some(TEST_VIDEO_ID.to_string())
    );
    assert_eq!(microformat.owner_channel_name, Some("TED".to_string()));
    assert_eq!(
        microformat.external_channel_id,
        Some("UCAuUUnT6oDeKwE6v1NGQxug".to_string())
    );
    assert_eq!(microformat.category, Some("People & Blogs".to_string()));
    assert_eq!(microformat.is_family_safe, Some(true));
    assert_eq!(microformat.is_unlisted, Some(false));
    assert_eq!(microformat.length_seconds, Some("844".to_string()));
    assert_eq!(microformat.like_count, Some("2009473".to_string()));
    assert_eq!(microformat.view_count, Some("58968839".to_string()));

    // Check if title exists and has the expected content
    assert!(microformat.title.is_some());
    assert_eq!(
        microformat.title.unwrap(),
        "Inside the Mind of a Master Procrastinator | Tim Urban | TED"
    );

    // Check description
    assert!(microformat.description.is_some());
    assert_eq!(
        microformat.description.unwrap(),
        "This is a test description"
    );

    // Check embed information
    assert!(microformat.embed.is_some());
    let embed = microformat.embed.unwrap();
    assert_eq!(embed.width, Some(1280));
    assert_eq!(embed.height, Some(720));
    assert_eq!(
        embed.iframe_url,
        Some(format!("https://www.youtube.com/embed/{}", TEST_VIDEO_ID))
    );

    // Verify thumbnail exists
    assert!(microformat.thumbnail.is_some());
    let thumbnail = microformat.thumbnail.unwrap();
    assert!(thumbnail.thumbnails.is_some());
    let thumbnails = thumbnail.thumbnails.unwrap();
    assert_eq!(thumbnails.len(), 1);
    assert_eq!(thumbnails[0].width, 1280);
    assert_eq!(thumbnails[0].height, 720);
    assert_eq!(
        thumbnails[0].url,
        format!("https://i.ytimg.com/vi/{}/maxresdefault.jpg", TEST_VIDEO_ID)
    );

    // Check available countries
    assert!(microformat.available_countries.is_some());
    let countries = microformat.available_countries.unwrap();
    assert!(!countries.is_empty());
    assert!(countries.contains(&"US".to_string()));
    assert!(countries.contains(&"GB".to_string()));
    assert!(countries.contains(&"JP".to_string()));
    assert_eq!(countries.len(), 6); // There should be 6 countries in our test data

    // Check upload and publish dates
    assert_eq!(
        microformat.upload_date,
        Some("2016-04-06T09:59:35-07:00".to_string())
    );
    assert_eq!(
        microformat.publish_date,
        Some("2016-04-06T09:59:35-07:00".to_string())
    );
}

/// Test the extraction of streaming data with a full set of fields
#[tokio::test]
async fn test_streaming_data_extraction_comprehensive() {
    use crate::streaming_data_extractor::StreamingDataExtractor;
    use serde_json::json;

    // Create a comprehensive player response with streaming data
    let player_response = json!({
        "streamingData": {
            "expiresInSeconds": "21540",
            "formats": [
                {
                    "itag": 18,
                    "url": "https://example.com/video.mp4",
                    "mimeType": "video/mp4; codecs=\"avc1.42001E, mp4a.40.2\"",
                    "bitrate": 347177,
                    "width": 640,
                    "height": 360,
                    "lastModified": "1739036631573310",
                    "contentLength": "36612752",
                    "quality": "medium",
                    "fps": 24,
                    "qualityLabel": "360p",
                    "projectionType": "RECTANGULAR",
                    "averageBitrate": 347136,
                    "audioQuality": "AUDIO_QUALITY_LOW",
                    "approxDurationMs": "843766",
                    "audioSampleRate": "44100",
                    "audioChannels": 2,
                    "qualityOrdinal": "QUALITY_ORDINAL_360P"
                }
            ],
            "adaptiveFormats": [
                {
                    "itag": 136,
                    "mimeType": "video/mp4; codecs=\"avc1.4d401f\"",
                    "bitrate": 582678,
                    "width": 1280,
                    "height": 720,
                    "initRange": {
                        "start": "0",
                        "end": "739"
                    },
                    "indexRange": {
                        "start": "740",
                        "end": "2703"
                    },
                    "lastModified": "1739044586331610",
                    "contentLength": "26758604",
                    "quality": "hd720",
                    "fps": 24,
                    "qualityLabel": "720p",
                    "projectionType": "RECTANGULAR",
                    "averageBitrate": 253736,
                    "approxDurationMs": "843666",
                    "qualityOrdinal": "QUALITY_ORDINAL_720P"
                },
                {
                    "itag": 140,
                    "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"",
                    "bitrate": 130904,
                    "initRange": {
                        "start": "0",
                        "end": "722"
                    },
                    "indexRange": {
                        "start": "723",
                        "end": "1774"
                    },
                    "lastModified": "1739026344625664",
                    "contentLength": "13656269",
                    "quality": "tiny",
                    "projectionType": "RECTANGULAR",
                    "averageBitrate": 129479,
                    "highReplication": true,
                    "audioQuality": "AUDIO_QUALITY_MEDIUM",
                    "approxDurationMs": "843766",
                    "audioSampleRate": "44100",
                    "audioChannels": 2,
                    "loudnessDb": -2.6200008,
                    "qualityOrdinal": "QUALITY_ORDINAL_UNKNOWN"
                },
                {
                    "itag": 247,
                    "mimeType": "video/webm; codecs=\"vp9\"",
                    "bitrate": 676276,
                    "width": 1280,
                    "height": 720,
                    "initRange": {
                        "start": "0",
                        "end": "219"
                    },
                    "indexRange": {
                        "start": "220",
                        "end": "2984"
                    },
                    "lastModified": "1739038691129096",
                    "contentLength": "22548557",
                    "quality": "hd720",
                    "fps": 24,
                    "qualityLabel": "720p",
                    "projectionType": "RECTANGULAR",
                    "averageBitrate": 213815,
                    "colorInfo": {
                        "primaries": "COLOR_PRIMARIES_BT709",
                        "transferCharacteristics": "COLOR_TRANSFER_CHARACTERISTICS_BT709",
                        "matrixCoefficients": "COLOR_MATRIX_COEFFICIENTS_BT709"
                    },
                    "approxDurationMs": "843666",
                    "qualityOrdinal": "QUALITY_ORDINAL_720P"
                }
            ],
            "serverAbrStreamingUrl": "https://example.com/streaming.mp4"
        }
    });

    // Extract streaming data using the extractor
    let result = StreamingDataExtractor::extract_streaming_data(&player_response, TEST_VIDEO_ID);

    // Verify the result
    assert!(
        result.is_ok(),
        "Failed to extract streaming data: {:?}",
        result.err()
    );

    let streaming_data = result.unwrap();

    // Verify expires_in_seconds
    assert_eq!(streaming_data.expires_in_seconds, "21540");

    // Verify formats
    assert_eq!(streaming_data.formats.len(), 1);
    let format = &streaming_data.formats[0];
    assert_eq!(format.itag, 18);
    assert_eq!(format.width, Some(640));
    assert_eq!(format.height, Some(360));
    assert_eq!(format.fps, Some(24));
    assert_eq!(format.quality_label, Some("360p".to_string()));
    assert_eq!(format.audio_channels, Some(2));

    // Verify adaptive formats
    assert_eq!(streaming_data.adaptive_formats.len(), 3);

    // Find video format with initRange and indexRange
    let video_format = streaming_data
        .adaptive_formats
        .iter()
        .find(|f| f.itag == 136)
        .expect("Missing expected video format with itag 136");

    assert_eq!(video_format.width, Some(1280));
    assert_eq!(video_format.height, Some(720));
    assert_eq!(video_format.quality_label, Some("720p".to_string()));

    // Check range data
    assert!(video_format.init_range.is_some());
    let init_range = video_format.init_range.as_ref().unwrap();
    assert_eq!(init_range.start, "0");
    assert_eq!(init_range.end, "739");

    assert!(video_format.index_range.is_some());
    let index_range = video_format.index_range.as_ref().unwrap();
    assert_eq!(index_range.start, "740");
    assert_eq!(index_range.end, "2703");

    // Find audio format
    let audio_format = streaming_data
        .adaptive_formats
        .iter()
        .find(|f| f.itag == 140)
        .expect("Missing expected audio format with itag 140");

    assert!(audio_format.width.is_none());
    assert!(audio_format.height.is_none());
    assert_eq!(
        audio_format.audio_quality,
        Some("AUDIO_QUALITY_MEDIUM".to_string())
    );
    assert_eq!(audio_format.audio_sample_rate, Some("44100".to_string()));
    assert_eq!(audio_format.audio_channels, Some(2));
    assert!(audio_format.loudness_db.is_some());
    assert!(audio_format.high_replication.is_some());
    assert!(audio_format.high_replication.unwrap());

    // Find format with color info
    let color_format = streaming_data
        .adaptive_formats
        .iter()
        .find(|f| f.itag == 247)
        .expect("Missing expected format with itag 247");

    assert!(color_format.color_info.is_some());
    let color_info = color_format.color_info.as_ref().unwrap();
    assert_eq!(
        color_info.primaries,
        Some("COLOR_PRIMARIES_BT709".to_string())
    );
    assert_eq!(
        color_info.transfer_characteristics,
        Some("COLOR_TRANSFER_CHARACTERISTICS_BT709".to_string())
    );
    assert_eq!(
        color_info.matrix_coefficients,
        Some("COLOR_MATRIX_COEFFICIENTS_BT709".to_string())
    );

    // Check server ABR streaming URL
    assert_eq!(
        streaming_data.server_abr_streaming_url,
        Some("https://example.com/streaming.mp4".to_string())
    );
}
