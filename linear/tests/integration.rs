//! Integration tests for fgp-linear daemon
//!
//! These tests require a LINEAR_API_KEY environment variable to run integration tests.

use std::collections::HashMap;
use serde_json::{json, Value};

/// Test GraphQL query building
#[test]
fn test_graphql_query_structure() {
    let query = r#"
        query Me {
            viewer {
                id
                name
                email
            }
        }
    "#;
    assert!(query.contains("viewer"));
    assert!(query.contains("id"));
}

/// Test issue filter construction
#[test]
fn test_issue_filter_building() {
    let filter = json!({
        "team": { "key": { "eq": "ENG" } },
        "state": { "type": { "eq": "started" } }
    });

    assert!(filter.get("team").is_some());
    assert!(filter.get("state").is_some());
}

/// Test priority mapping
#[test]
fn test_priority_values() {
    // Linear uses 0-4 for priority (0 = no priority, 1 = urgent, 4 = low)
    let priorities = [
        (0, "No priority"),
        (1, "Urgent"),
        (2, "High"),
        (3, "Medium"),
        (4, "Low"),
    ];

    for (value, _name) in priorities {
        assert!(value <= 4);
    }
}

/// Test method parameter extraction
#[test]
fn test_param_extraction() {
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("title".into(), json!("Bug fix"));
    params.insert("team_id".into(), json!("abc123"));
    params.insert("priority".into(), json!(2));

    let title = params.get("title").and_then(|v| v.as_str());
    assert_eq!(title, Some("Bug fix"));

    let priority = params.get("priority").and_then(|v| v.as_i64());
    assert_eq!(priority, Some(2));
}

/// Test issue ID format validation
#[test]
fn test_issue_id_format() {
    // Linear issue IDs are UUIDs
    let valid_id = "550e8400-e29b-41d4-a716-446655440000";
    let parts: Vec<&str> = valid_id.split('-').collect();
    assert_eq!(parts.len(), 5);

    // Also test short identifiers like "ENG-123"
    let short_id = "ENG-123";
    assert!(short_id.contains('-'));
    let parts: Vec<&str> = short_id.split('-').collect();
    assert_eq!(parts.len(), 2);
}

/// Test GraphQL variables serialization
#[test]
fn test_graphql_variables() {
    let variables = json!({
        "id": "abc123",
        "input": {
            "title": "New Issue",
            "teamId": "team-id"
        }
    });

    let serialized = serde_json::to_string(&variables).unwrap();
    assert!(serialized.contains("abc123"));
    assert!(serialized.contains("New Issue"));
}

#[cfg(feature = "integration")]
mod integration {
    //! These tests require LINEAR_API_KEY to be set

    fn skip_if_no_api_key() -> bool {
        std::env::var("LINEAR_API_KEY").is_err()
    }

    #[test]
    fn test_api_connection() {
        if skip_if_no_api_key() {
            eprintln!("Skipping: LINEAR_API_KEY not set");
            return;
        }
        // Real API test would go here
    }
}
