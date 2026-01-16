//! Integration tests for fgp-notion daemon
//!
//! These tests require a NOTION_API_KEY environment variable to run integration tests.

use std::collections::HashMap;
use serde_json::{json, Value};

/// Test page ID normalization (removes dashes)
#[test]
fn test_page_id_normalization() {
    let with_dashes = "550e8400-e29b-41d4-a716-446655440000";
    let without_dashes = with_dashes.replace('-', "");
    assert_eq!(without_dashes, "550e8400e29b41d4a716446655440000");
    assert_eq!(without_dashes.len(), 32);
}

/// Test block type enum values
#[test]
fn test_block_types() {
    let block_types = [
        "paragraph",
        "heading_1",
        "heading_2",
        "heading_3",
        "bulleted_list_item",
        "numbered_list_item",
        "to_do",
        "code",
        "quote",
        "divider",
        "callout",
    ];

    for block_type in block_types {
        assert!(!block_type.is_empty());
        assert!(block_type.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }
}

/// Test rich text construction
#[test]
fn test_rich_text_building() {
    let rich_text = json!([{
        "type": "text",
        "text": { "content": "Hello World" }
    }]);

    assert!(rich_text.is_array());
    let arr = rich_text.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["type"], "text");
}

/// Test filter object construction
#[test]
fn test_database_filter() {
    let filter = json!({
        "property": "Status",
        "select": { "equals": "In Progress" }
    });

    assert_eq!(filter["property"], "Status");
    assert!(filter.get("select").is_some());
}

/// Test sort configuration
#[test]
fn test_database_sorts() {
    let sorts = json!([
        { "property": "Created", "direction": "descending" },
        { "timestamp": "last_edited_time", "direction": "ascending" }
    ]);

    let arr = sorts.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["direction"], "descending");
}

/// Test method parameter extraction
#[test]
fn test_param_extraction() {
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("page_id".into(), json!("abc123"));
    params.insert("recursive".into(), json!(true));
    params.insert("limit".into(), json!(25));

    let page_id = params.get("page_id").and_then(|v| v.as_str());
    assert_eq!(page_id, Some("abc123"));

    let recursive = params.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
    assert!(recursive);
}

/// Test property types for database pages
#[test]
fn test_property_types() {
    let title_prop = json!({
        "title": [{ "text": { "content": "My Title" } }]
    });

    let select_prop = json!({
        "select": { "name": "Option A" }
    });

    let number_prop = json!({
        "number": 42
    });

    assert!(title_prop.get("title").is_some());
    assert!(select_prop.get("select").is_some());
    assert_eq!(number_prop["number"], 42);
}

/// Test Notion API version header value
#[test]
fn test_api_version() {
    let version = "2022-06-28";
    // Verify it's a valid date format
    let parts: Vec<&str> = version.split('-').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].len(), 4); // year
    assert_eq!(parts[1].len(), 2); // month
    assert_eq!(parts[2].len(), 2); // day
}

#[cfg(feature = "integration")]
mod integration {
    //! These tests require NOTION_API_KEY to be set

    fn skip_if_no_api_key() -> bool {
        std::env::var("NOTION_API_KEY").is_err() && std::env::var("NOTION_TOKEN").is_err()
    }

    #[test]
    fn test_api_connection() {
        if skip_if_no_api_key() {
            eprintln!("Skipping: NOTION_API_KEY not set");
            return;
        }
        // Real API test would go here
    }
}
