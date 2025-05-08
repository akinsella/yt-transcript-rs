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

/// Test microformat extraction from the file-based real sample
#[tokio::test]
async fn test_microformat_extraction_from_file() {
    use crate::microformat_extractor::MicroformatExtractor;
    use std::fs::File;
    use std::io::Read;

    // Skip the test if the sample file doesn't exist
    let file_result = File::open("src/tests/resources/player_response.json");
    if file_result.is_err() {
        println!("Skipping test_microformat_extraction_from_file because src/tests/resources/player_response.json does not exist");
        return;
    }

    // Load the sample player response JSON from resources directory
    let mut file = file_result.unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)
        .expect("Failed to read player_response.json");

    // Parse the JSON
    let player_response: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON");

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
