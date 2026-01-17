//! Data models for Cloudflare API.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};

/// Cloudflare API response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(default)]
    pub errors: Vec<ApiError>,
    #[serde(default)]
    pub messages: Vec<serde_json::Value>,
    #[serde(default)]
    pub result: Option<T>,
    #[serde(default)]
    pub result_info: Option<ResultInfo>,
}

/// API error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}

/// Pagination info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultInfo {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub per_page: Option<i64>,
    #[serde(default)]
    pub total_count: Option<i64>,
    #[serde(default)]
    pub total_pages: Option<i64>,
}

/// A Cloudflare zone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Zone {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub paused: Option<bool>,
    #[serde(default, rename = "type")]
    pub zone_type: Option<String>,
    #[serde(default)]
    pub name_servers: Option<Vec<String>>,
}

/// A DNS record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DnsRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "type")]
    pub record_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub proxied: Option<bool>,
    #[serde(default)]
    pub ttl: Option<i64>,
    #[serde(default)]
    pub priority: Option<i64>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub modified_on: Option<String>,
}

/// A KV namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvNamespace {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub supports_url_encoding: Option<bool>,
}

/// A KV key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvKey {
    pub name: String,
    #[serde(default)]
    pub expiration: Option<i64>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// A Worker script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub id: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub modified_on: Option<String>,
}

/// Worker route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRoute {
    pub id: String,
    pub pattern: String,
    #[serde(default)]
    pub script: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn api_response_defaults_empty_collections() {
        let value = json!({
            "success": true
        });
        let response: ApiResponse<serde_json::Value> =
            serde_json::from_value(value).expect("deserialize response");

        assert!(response.success);
        assert!(response.errors.is_empty());
        assert!(response.messages.is_empty());
        assert!(response.result.is_none());
        assert!(response.result_info.is_none());
    }

    #[test]
    fn zone_defaults_optional_fields() {
        let value = json!({
            "id": "zone_1",
            "name": "example.com",
            "status": "active"
        });
        let zone: Zone = serde_json::from_value(value).expect("deserialize zone");

        assert_eq!(zone.id, "zone_1");
        assert_eq!(zone.name, "example.com");
        assert_eq!(zone.status, "active");
        assert!(zone.zone_type.is_none());
        assert!(zone.name_servers.is_none());
    }

    #[test]
    fn dns_record_reads_type_field() {
        let value = json!({
            "id": "rec_1",
            "type": "A",
            "name": "example.com",
            "content": "1.2.3.4"
        });
        let record: DnsRecord = serde_json::from_value(value).expect("deserialize record");

        assert_eq!(record.record_type, "A");
        assert_eq!(record.name, "example.com");
        assert_eq!(record.content, "1.2.3.4");
    }

    #[test]
    fn kv_namespace_optional_flag() {
        let value = json!({
            "id": "kv_1",
            "title": "store"
        });
        let namespace: KvNamespace = serde_json::from_value(value).expect("deserialize namespace");

        assert_eq!(namespace.title, "store");
        assert!(namespace.supports_url_encoding.is_none());
    }
}
