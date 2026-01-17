//! Google Drive HTTP API client with connection pooling.
//!
//! Uses OAuth2 tokens from ~/.fgp/auth/google/token.json
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::path::PathBuf;

use crate::models::{AboutResponse, DriveFile, FileListResponse, Permission};

const DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";
const UPLOAD_API_BASE: &str = "https://www.googleapis.com/upload/drive/v3";

/// Google Drive API client with persistent connection pooling.
pub struct DriveClient {
    client: Client,
    token_path: PathBuf,
}

impl DriveClient {
    /// Create a new Drive client.
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let token_path = home.join(".fgp/auth/google/token.json");

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("fgp-google-drive/1.0.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, token_path })
    }

    /// Get OAuth2 access token.
    fn get_token(&self) -> Result<String> {
        if !self.token_path.exists() {
            bail!(
                "Google OAuth token not found at {:?}. Run OAuth flow first.",
                self.token_path
            );
        }

        let content = std::fs::read_to_string(&self.token_path)
            .context("Failed to read token file")?;
        let token: Value = serde_json::from_str(&content)
            .context("Failed to parse token file")?;

        token["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No access_token in token file"))
    }

    /// List files in Drive.
    pub async fn list_files(
        &self,
        query: Option<&str>,
        page_size: Option<i32>,
        page_token: Option<&str>,
        folder_id: Option<&str>,
    ) -> Result<FileListResponse> {
        let token = self.get_token()?;

        let mut url = format!(
            "{}/files?fields=files(id,name,mimeType,size,createdTime,modifiedTime,parents,webViewLink,webContentLink,shared,trashed),nextPageToken",
            DRIVE_API_BASE
        );

        if let Some(size) = page_size {
            url.push_str(&format!("&pageSize={}", size));
        }

        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        // Build query
        let mut q_parts = Vec::new();
        if let Some(folder) = folder_id {
            q_parts.push(format!("'{}' in parents", folder));
        }
        if let Some(q) = query {
            q_parts.push(q.to_string());
        }
        if !q_parts.is_empty() {
            url.push_str(&format!("&q={}", urlencoding::encode(&q_parts.join(" and "))));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list files")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse file list")
    }

    /// Get file metadata.
    pub async fn get_file(&self, file_id: &str) -> Result<DriveFile> {
        let token = self.get_token()?;

        let url = format!(
            "{}/files/{}?fields=id,name,mimeType,size,createdTime,modifiedTime,parents,webViewLink,webContentLink,shared,trashed",
            DRIVE_API_BASE, file_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to get file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse file")
    }

    /// Download file content.
    pub async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let token = self.get_token()?;

        let url = format!("{}/files/{}?alt=media", DRIVE_API_BASE, file_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to download file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.bytes().await.map(|b| b.to_vec()).context("Failed to read file content")
    }

    /// Upload a file.
    pub async fn upload_file(
        &self,
        name: &str,
        content: &[u8],
        mime_type: &str,
        parent_id: Option<&str>,
    ) -> Result<DriveFile> {
        let token = self.get_token()?;

        // Use multipart upload for simplicity
        let mut metadata = serde_json::json!({
            "name": name
        });

        if let Some(parent) = parent_id {
            metadata["parents"] = serde_json::json!([parent]);
        }

        let url = format!(
            "{}/files?uploadType=multipart&fields=id,name,mimeType,size,createdTime,modifiedTime,parents,webViewLink",
            UPLOAD_API_BASE
        );

        // Build multipart body manually
        let boundary = "fgp_boundary_12345";
        let mut body = Vec::new();

        // Metadata part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(metadata.to_string().as_bytes());
        body.extend_from_slice(b"\r\n");

        // File content part
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
        body.extend_from_slice(content);
        body.extend_from_slice(format!("\r\n--{}--", boundary).as_bytes());

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", format!("multipart/related; boundary={}", boundary))
            .body(body)
            .send()
            .await
            .context("Failed to upload file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse upload response")
    }

    /// Create a folder.
    pub async fn create_folder(&self, name: &str, parent_id: Option<&str>) -> Result<DriveFile> {
        let token = self.get_token()?;

        let mut metadata = serde_json::json!({
            "name": name,
            "mimeType": "application/vnd.google-apps.folder"
        });

        if let Some(parent) = parent_id {
            metadata["parents"] = serde_json::json!([parent]);
        }

        let url = format!(
            "{}/files?fields=id,name,mimeType,createdTime,modifiedTime,parents,webViewLink",
            DRIVE_API_BASE
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&metadata)
            .send()
            .await
            .context("Failed to create folder")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse folder response")
    }

    /// Move a file to a new parent.
    pub async fn move_file(&self, file_id: &str, new_parent_id: &str) -> Result<DriveFile> {
        let token = self.get_token()?;

        // First get current parents
        let file = self.get_file(file_id).await?;
        let current_parents = file.parents.unwrap_or_default().join(",");

        let url = format!(
            "{}/files/{}?addParents={}&removeParents={}&fields=id,name,mimeType,parents,webViewLink",
            DRIVE_API_BASE, file_id, new_parent_id, current_parents
        );

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to move file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse move response")
    }

    /// Copy a file.
    pub async fn copy_file(&self, file_id: &str, new_name: Option<&str>) -> Result<DriveFile> {
        let token = self.get_token()?;

        let mut body = serde_json::json!({});
        if let Some(name) = new_name {
            body["name"] = serde_json::json!(name);
        }

        let url = format!(
            "{}/files/{}/copy?fields=id,name,mimeType,size,createdTime,modifiedTime,parents,webViewLink",
            DRIVE_API_BASE, file_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to copy file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse copy response")
    }

    /// Delete (trash) a file.
    pub async fn delete_file(&self, file_id: &str, permanent: bool) -> Result<()> {
        let token = self.get_token()?;

        if permanent {
            let url = format!("{}/files/{}", DRIVE_API_BASE, file_id);
            let response = self
                .client
                .delete(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .context("Failed to delete file")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                bail!("Drive API error: {} - {}", status, text);
            }
        } else {
            // Move to trash
            let url = format!("{}/files/{}", DRIVE_API_BASE, file_id);
            let response = self
                .client
                .patch(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({"trashed": true}))
                .send()
                .await
                .context("Failed to trash file")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                bail!("Drive API error: {} - {}", status, text);
            }
        }

        Ok(())
    }

    /// Share a file with a user.
    pub async fn share_file(
        &self,
        file_id: &str,
        email: &str,
        role: &str, // "reader", "writer", "commenter"
    ) -> Result<Permission> {
        let token = self.get_token()?;

        let body = serde_json::json!({
            "type": "user",
            "role": role,
            "emailAddress": email
        });

        let url = format!("{}/files/{}/permissions", DRIVE_API_BASE, file_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to share file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse share response")
    }

    /// Get user and quota info.
    pub async fn about(&self) -> Result<AboutResponse> {
        let token = self.get_token()?;

        let url = format!(
            "{}/about?fields=user(displayName,emailAddress,photoLink),storageQuota(limit,usage,usageInDrive,usageInDriveTrash)",
            DRIVE_API_BASE
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to get about info")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Drive API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse about response")
    }

    /// Search for files.
    pub async fn search(&self, query: &str, page_size: Option<i32>) -> Result<FileListResponse> {
        self.list_files(Some(query), page_size, None, None).await
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                ' ' => result.push_str("%20"),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}
