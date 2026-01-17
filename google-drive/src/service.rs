//! FGP service implementation for Google Drive.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::DriveClient;

/// FGP service for Google Drive operations.
pub struct DriveService {
    client: Arc<DriveClient>,
    runtime: Runtime,
}

impl DriveService {
    /// Create a new DriveService.
    pub fn new() -> Result<Self> {
        let client = DriveClient::new()?;
        let runtime = Runtime::new()?;

        Ok(Self {
            client: Arc::new(client),
            runtime,
        })
    }

    /// Helper to get a string parameter.
    fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Helper to get an i64 parameter.
    fn get_i64(params: &HashMap<String, Value>, key: &str) -> Option<i64> {
        params.get(key).and_then(|v| v.as_i64())
    }

    /// Helper to get a bool parameter.
    fn get_bool(params: &HashMap<String, Value>, key: &str) -> Option<bool> {
        params.get(key).and_then(|v| v.as_bool())
    }

    // ========================================================================
    // File operations
    // ========================================================================

    fn list_files(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_str(&params, "query");
        let page_size = Self::get_i64(&params, "page_size").map(|v| v as i32);
        let page_token = Self::get_str(&params, "page_token");
        let folder_id = Self::get_str(&params, "folder_id");

        let client = self.client.clone();
        let query_owned = query.map(|s| s.to_string());
        let page_token_owned = page_token.map(|s| s.to_string());
        let folder_id_owned = folder_id.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.list_files(
                query_owned.as_deref(),
                page_size,
                page_token_owned.as_deref(),
                folder_id_owned.as_deref(),
            ).await
        })?;

        Ok(json!(result))
    }

    fn get_file(&self, params: HashMap<String, Value>) -> Result<Value> {
        let file_id = Self::get_str(&params, "file_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_id"))?;

        let client = self.client.clone();
        let file_id = file_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_file(&file_id).await
        })?;

        Ok(json!(result))
    }

    fn download(&self, params: HashMap<String, Value>) -> Result<Value> {
        let file_id = Self::get_str(&params, "file_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_id"))?;
        let save_path = Self::get_str(&params, "save_path");

        let client = self.client.clone();
        let file_id = file_id.to_string();

        let content = self.runtime.block_on(async move {
            client.download_file(&file_id).await
        })?;

        if let Some(path) = save_path {
            let expanded = shellexpand::tilde(path).to_string();
            std::fs::write(&expanded, &content)?;
            Ok(json!({
                "saved_to": expanded,
                "size": content.len()
            }))
        } else {
            // Return base64 encoded content
            Ok(json!({
                "content_base64": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &content),
                "size": content.len()
            }))
        }
    }

    fn upload(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let mime_type = Self::get_str(&params, "mime_type")
            .unwrap_or("application/octet-stream");
        let parent_id = Self::get_str(&params, "parent_id");

        // Content can be provided as base64 or file path
        let content = if let Some(content_b64) = Self::get_str(&params, "content_base64") {
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, content_b64)
                .map_err(|e| anyhow::anyhow!("Invalid base64: {}", e))?
        } else if let Some(file_path) = Self::get_str(&params, "file_path") {
            let expanded = shellexpand::tilde(file_path).to_string();
            std::fs::read(&expanded)?
        } else {
            anyhow::bail!("Missing content_base64 or file_path parameter");
        };

        let client = self.client.clone();
        let name = name.to_string();
        let mime_type = mime_type.to_string();
        let parent_id = parent_id.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.upload_file(&name, &content, &mime_type, parent_id.as_deref()).await
        })?;

        Ok(json!(result))
    }

    fn create_folder(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let parent_id = Self::get_str(&params, "parent_id");

        let client = self.client.clone();
        let name = name.to_string();
        let parent_id = parent_id.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.create_folder(&name, parent_id.as_deref()).await
        })?;

        Ok(json!(result))
    }

    fn move_file(&self, params: HashMap<String, Value>) -> Result<Value> {
        let file_id = Self::get_str(&params, "file_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_id"))?;
        let new_parent_id = Self::get_str(&params, "new_parent_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: new_parent_id"))?;

        let client = self.client.clone();
        let file_id = file_id.to_string();
        let new_parent_id = new_parent_id.to_string();

        let result = self.runtime.block_on(async move {
            client.move_file(&file_id, &new_parent_id).await
        })?;

        Ok(json!(result))
    }

    fn copy_file(&self, params: HashMap<String, Value>) -> Result<Value> {
        let file_id = Self::get_str(&params, "file_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_id"))?;
        let new_name = Self::get_str(&params, "new_name");

        let client = self.client.clone();
        let file_id = file_id.to_string();
        let new_name = new_name.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.copy_file(&file_id, new_name.as_deref()).await
        })?;

        Ok(json!(result))
    }

    fn delete_file(&self, params: HashMap<String, Value>) -> Result<Value> {
        let file_id = Self::get_str(&params, "file_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_id"))?;
        let permanent = Self::get_bool(&params, "permanent").unwrap_or(false);

        let client = self.client.clone();
        let file_id = file_id.to_string();
        let file_id_clone = file_id.clone();

        self.runtime.block_on(async move {
            client.delete_file(&file_id_clone, permanent).await
        })?;

        Ok(json!({
            "deleted": true,
            "file_id": file_id,
            "permanent": permanent
        }))
    }

    fn share(&self, params: HashMap<String, Value>) -> Result<Value> {
        let file_id = Self::get_str(&params, "file_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_id"))?;
        let email = Self::get_str(&params, "email")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: email"))?;
        let role = Self::get_str(&params, "role").unwrap_or("reader");

        let client = self.client.clone();
        let file_id = file_id.to_string();
        let email = email.to_string();
        let role = role.to_string();

        let result = self.runtime.block_on(async move {
            client.share_file(&file_id, &email, &role).await
        })?;

        Ok(json!(result))
    }

    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_str(&params, "query")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        let page_size = Self::get_i64(&params, "page_size").map(|v| v as i32);

        let client = self.client.clone();
        let query = query.to_string();

        let result = self.runtime.block_on(async move {
            client.search(&query, page_size).await
        })?;

        Ok(json!(result))
    }

    fn about(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.about().await
        })?;

        Ok(json!(result))
    }
}

impl FgpService for DriveService {
    fn name(&self) -> &str {
        "drive"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // File operations
            "list" | "drive.list" => self.list_files(params),
            "get" | "drive.get" => self.get_file(params),
            "download" | "drive.download" => self.download(params),
            "upload" | "drive.upload" => self.upload(params),
            "create_folder" | "drive.create_folder" => self.create_folder(params),
            "move" | "drive.move" => self.move_file(params),
            "copy" | "drive.copy" => self.copy_file(params),
            "delete" | "drive.delete" => self.delete_file(params),
            "share" | "drive.share" => self.share(params),
            "search" | "drive.search" => self.search(params),
            "about" | "drive.about" => self.about(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("drive.list", "List files in Drive or folder")
                .schema(
                    SchemaBuilder::object()
                        .property("query", SchemaBuilder::string().description("Search query"))
                        .property("folder_id", SchemaBuilder::string().description("Parent folder ID"))
                        .property("page_size", SchemaBuilder::integer().description("Results per page"))
                        .property("page_token", SchemaBuilder::string().description("Pagination token"))
                        .build(),
                ),

            MethodInfo::new("drive.get", "Get file metadata")
                .schema(
                    SchemaBuilder::object()
                        .property("file_id", SchemaBuilder::string().description("File ID"))
                        .required(&["file_id"])
                        .build(),
                ),

            MethodInfo::new("drive.download", "Download file content")
                .schema(
                    SchemaBuilder::object()
                        .property("file_id", SchemaBuilder::string().description("File ID"))
                        .property("save_path", SchemaBuilder::string().description("Local path to save"))
                        .required(&["file_id"])
                        .build(),
                ),

            MethodInfo::new("drive.upload", "Upload a file")
                .schema(
                    SchemaBuilder::object()
                        .property("name", SchemaBuilder::string().description("File name"))
                        .property("content_base64", SchemaBuilder::string().description("Base64 content"))
                        .property("file_path", SchemaBuilder::string().description("Local file path"))
                        .property("mime_type", SchemaBuilder::string().description("MIME type"))
                        .property("parent_id", SchemaBuilder::string().description("Parent folder ID"))
                        .required(&["name"])
                        .build(),
                ),

            MethodInfo::new("drive.create_folder", "Create a folder")
                .schema(
                    SchemaBuilder::object()
                        .property("name", SchemaBuilder::string().description("Folder name"))
                        .property("parent_id", SchemaBuilder::string().description("Parent folder ID"))
                        .required(&["name"])
                        .build(),
                ),

            MethodInfo::new("drive.move", "Move file to new folder")
                .schema(
                    SchemaBuilder::object()
                        .property("file_id", SchemaBuilder::string().description("File ID"))
                        .property("new_parent_id", SchemaBuilder::string().description("New parent folder ID"))
                        .required(&["file_id", "new_parent_id"])
                        .build(),
                ),

            MethodInfo::new("drive.copy", "Copy a file")
                .schema(
                    SchemaBuilder::object()
                        .property("file_id", SchemaBuilder::string().description("File ID"))
                        .property("new_name", SchemaBuilder::string().description("New file name"))
                        .required(&["file_id"])
                        .build(),
                ),

            MethodInfo::new("drive.delete", "Delete (trash) a file")
                .schema(
                    SchemaBuilder::object()
                        .property("file_id", SchemaBuilder::string().description("File ID"))
                        .property("permanent", SchemaBuilder::boolean().description("Permanently delete"))
                        .required(&["file_id"])
                        .build(),
                ),

            MethodInfo::new("drive.share", "Share file with user")
                .schema(
                    SchemaBuilder::object()
                        .property("file_id", SchemaBuilder::string().description("File ID"))
                        .property("email", SchemaBuilder::string().description("User email"))
                        .property("role", SchemaBuilder::string().description("reader/writer/commenter"))
                        .required(&["file_id", "email"])
                        .build(),
                ),

            MethodInfo::new("drive.search", "Search for files")
                .schema(
                    SchemaBuilder::object()
                        .property("query", SchemaBuilder::string().description("Drive query string"))
                        .property("page_size", SchemaBuilder::integer().description("Results per page"))
                        .required(&["query"])
                        .build(),
                ),

            MethodInfo::new("drive.about", "Get user and quota info"),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Token file check
        let home = dirs::home_dir().unwrap_or_default();
        let token_path = home.join(".fgp/auth/google/token.json");

        checks.insert(
            "oauth_token".into(),
            if token_path.exists() {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("OAuth token not found - run auth flow")
            },
        );

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_service() -> DriveService {
        DriveService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("file_id".to_string(), Value::String("file".to_string()));
        params.insert("page_size".to_string(), Value::from(25));
        params.insert("permanent".to_string(), Value::Bool(true));

        assert_eq!(DriveService::get_str(&params, "file_id"), Some("file"));
        assert_eq!(DriveService::get_str(&params, "missing"), None);
        assert_eq!(DriveService::get_i64(&params, "page_size"), Some(25));
        assert_eq!(DriveService::get_i64(&params, "missing"), None);
        assert_eq!(DriveService::get_bool(&params, "permanent"), Some(true));
        assert_eq!(DriveService::get_bool(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_required_fields() {
        let service = test_service();
        let methods = service.method_list();

        let upload = methods
            .iter()
            .find(|m| m.name == "drive.upload")
            .expect("drive.upload");
        let upload_schema = upload.schema.as_ref().expect("schema");
        let required = upload_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "name"));

        let download = methods
            .iter()
            .find(|m| m.name == "drive.download")
            .expect("drive.download");
        let download_schema = download.schema.as_ref().expect("schema");
        let required = download_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "file_id"));
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("drive.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
