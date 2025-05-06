#[allow(unused_imports)]
use serde_json::json;

#[allow(unused_imports)]
use crate::{js_var_parser::JsVarParser, transcript_parser::TranscriptParser};

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
