//! Data models for Google Drive API.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};

/// A file or folder in Google Drive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default, rename = "createdTime")]
    pub created_time: Option<String>,
    #[serde(default, rename = "modifiedTime")]
    pub modified_time: Option<String>,
    #[serde(default)]
    pub parents: Option<Vec<String>>,
    #[serde(default, rename = "webViewLink")]
    pub web_view_link: Option<String>,
    #[serde(default, rename = "webContentLink")]
    pub web_content_link: Option<String>,
    #[serde(default)]
    pub shared: Option<bool>,
    #[serde(default)]
    pub trashed: Option<bool>,
}

/// List files response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListResponse {
    #[serde(default)]
    pub files: Vec<DriveFile>,
    #[serde(default, rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

/// Permission for sharing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub permission_type: String,
    pub role: String,
    #[serde(default, rename = "emailAddress")]
    pub email_address: Option<String>,
}

/// About response (user info, storage quota).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AboutResponse {
    pub user: DriveUser,
    #[serde(rename = "storageQuota")]
    pub storage_quota: StorageQuota,
}

/// Drive user info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveUser {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    #[serde(default, rename = "photoLink")]
    pub photo_link: Option<String>,
}

/// Storage quota info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuota {
    #[serde(default)]
    pub limit: Option<String>,
    #[serde(default)]
    pub usage: Option<String>,
    #[serde(default, rename = "usageInDrive")]
    pub usage_in_drive: Option<String>,
    #[serde(default, rename = "usageInDriveTrash")]
    pub usage_in_drive_trash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn drive_file_defaults_optional_fields() {
        let value = json!({
            "id": "file_1",
            "name": "Doc",
            "mimeType": "application/pdf"
        });
        let file: DriveFile = serde_json::from_value(value).expect("deserialize file");

        assert_eq!(file.id, "file_1");
        assert_eq!(file.mime_type, "application/pdf");
        assert!(file.size.is_none());
        assert!(file.parents.is_none());
        assert!(file.web_view_link.is_none());
    }

    #[test]
    fn file_list_response_defaults_files() {
        let value = json!({});
        let response: FileListResponse =
            serde_json::from_value(value).expect("deserialize list");

        assert!(response.files.is_empty());
        assert!(response.next_page_token.is_none());
    }

    #[test]
    fn permission_maps_email_field() {
        let value = json!({
            "id": "perm_1",
            "type": "user",
            "role": "reader",
            "emailAddress": "user@example.com"
        });
        let permission: Permission = serde_json::from_value(value).expect("deserialize");

        assert_eq!(permission.permission_type, "user");
        assert_eq!(permission.email_address.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn about_response_maps_storage_quota() {
        let value = json!({
            "user": {
                "displayName": "User",
                "emailAddress": "user@example.com"
            },
            "storageQuota": {
                "limit": "10",
                "usage": "1",
                "usageInDrive": "1",
                "usageInDriveTrash": "0"
            }
        });
        let response: AboutResponse = serde_json::from_value(value).expect("deserialize");

        assert_eq!(response.user.display_name, "User");
        assert_eq!(response.storage_quota.limit.as_deref(), Some("10"));
        assert_eq!(response.storage_quota.usage_in_drive_trash.as_deref(), Some("0"));
    }
}
