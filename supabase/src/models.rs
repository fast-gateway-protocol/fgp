//! Data models for Supabase API responses.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};

/// Supabase Auth user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub last_sign_in_at: Option<String>,
    #[serde(default)]
    pub app_metadata: serde_json::Value,
    #[serde(default)]
    pub user_metadata: serde_json::Value,
}

/// Auth session response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub user: User,
}

/// Auth error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthError {
    pub error: String,
    pub error_description: Option<String>,
}

/// Storage bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub id: String,
    pub name: String,
    pub owner: Option<String>,
    pub public: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Storage file object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObject {
    pub name: String,
    pub id: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_accessed_at: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Storage upload result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    #[serde(rename = "Key")]
    pub key: Option<String>,
    #[serde(rename = "Id")]
    pub id: Option<String>,
}

/// Edge function invocation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    pub data: serde_json::Value,
    pub status: u16,
}

/// PostgREST error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgrestError {
    pub message: String,
    pub code: Option<String>,
    pub details: Option<String>,
    pub hint: Option<String>,
}

/// Vector search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMatch {
    pub id: serde_json::Value,
    pub similarity: f64,
    pub content: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn user_defaults_metadata_to_null() {
        let value = json!({
            "id": "u1",
            "email": "user@example.com",
            "phone": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": null,
            "last_sign_in_at": null
        });
        let user: User = serde_json::from_value(value).expect("deserialize user");

        assert!(user.app_metadata.is_null());
        assert!(user.user_metadata.is_null());
    }

    #[test]
    fn storage_object_defaults_optional_fields() {
        let value = json!({
            "name": "file.txt"
        });
        let object: StorageObject = serde_json::from_value(value).expect("deserialize object");

        assert_eq!(object.name, "file.txt");
        assert!(object.updated_at.is_none());
        assert!(object.metadata.is_none());
    }

    #[test]
    fn upload_result_reads_renamed_fields() {
        let value = json!({
            "Key": "bucket/file.txt",
            "Id": "upload_1"
        });
        let result: UploadResult = serde_json::from_value(value).expect("deserialize upload");

        assert_eq!(result.key.as_deref(), Some("bucket/file.txt"));
        assert_eq!(result.id.as_deref(), Some("upload_1"));
    }

    #[test]
    fn vector_match_defaults_metadata_to_null() {
        let value = json!({
            "id": 1,
            "similarity": 0.9,
            "content": "hello"
        });
        let result: VectorMatch = serde_json::from_value(value).expect("deserialize match");

        assert_eq!(result.similarity, 0.9);
        assert!(result.metadata.is_null());
    }
}
