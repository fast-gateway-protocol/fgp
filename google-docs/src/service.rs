//! FGP service implementation for Google Docs.
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

use crate::api::DocsClient;

/// FGP service for Google Docs operations.
pub struct DocsService {
    client: Arc<DocsClient>,
    runtime: Runtime,
}

impl DocsService {
    /// Create a new DocsService.
    pub fn new() -> Result<Self> {
        let client = DocsClient::new()?;
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
    // Document operations
    // ========================================================================

    fn get_document(&self, params: HashMap<String, Value>) -> Result<Value> {
        let document_id = Self::get_str(&params, "document_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: document_id"))?;

        let client = self.client.clone();
        let document_id = document_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_document(&document_id).await
        })?;

        Ok(json!(result))
    }

    fn get_text(&self, params: HashMap<String, Value>) -> Result<Value> {
        let document_id = Self::get_str(&params, "document_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: document_id"))?;

        let client = self.client.clone();
        let document_id = document_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_text(&document_id).await
        })?;

        Ok(json!(result))
    }

    fn create_document(&self, params: HashMap<String, Value>) -> Result<Value> {
        let title = Self::get_str(&params, "title")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;

        let client = self.client.clone();
        let title = title.to_string();

        let result = self.runtime.block_on(async move {
            client.create_document(&title).await
        })?;

        Ok(json!(result))
    }

    fn insert_text(&self, params: HashMap<String, Value>) -> Result<Value> {
        let document_id = Self::get_str(&params, "document_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: document_id"))?;
        let text = Self::get_str(&params, "text")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let index = Self::get_i64(&params, "index")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: index"))?;

        let client = self.client.clone();
        let document_id = document_id.to_string();
        let text = text.to_string();

        let result = self.runtime.block_on(async move {
            client.insert_text(&document_id, &text, index).await
        })?;

        Ok(json!(result))
    }

    fn append_text(&self, params: HashMap<String, Value>) -> Result<Value> {
        let document_id = Self::get_str(&params, "document_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: document_id"))?;
        let text = Self::get_str(&params, "text")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;

        let client = self.client.clone();
        let document_id = document_id.to_string();
        let text = text.to_string();

        let result = self.runtime.block_on(async move {
            client.append_text(&document_id, &text).await
        })?;

        Ok(json!(result))
    }

    fn delete_content(&self, params: HashMap<String, Value>) -> Result<Value> {
        let document_id = Self::get_str(&params, "document_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: document_id"))?;
        let start_index = Self::get_i64(&params, "start_index")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: start_index"))?;
        let end_index = Self::get_i64(&params, "end_index")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: end_index"))?;

        let client = self.client.clone();
        let document_id = document_id.to_string();

        let result = self.runtime.block_on(async move {
            client.delete_content(&document_id, start_index, end_index).await
        })?;

        Ok(json!(result))
    }

    fn replace_text(&self, params: HashMap<String, Value>) -> Result<Value> {
        let document_id = Self::get_str(&params, "document_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: document_id"))?;
        let find = Self::get_str(&params, "find")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: find"))?;
        let replace = Self::get_str(&params, "replace")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: replace"))?;
        let match_case = Self::get_bool(&params, "match_case").unwrap_or(false);

        let client = self.client.clone();
        let document_id = document_id.to_string();
        let find = find.to_string();
        let replace = replace.to_string();

        let result = self.runtime.block_on(async move {
            client.replace_all_text(&document_id, &find, &replace, match_case).await
        })?;

        Ok(json!(result))
    }

    fn batch_update(&self, params: HashMap<String, Value>) -> Result<Value> {
        let document_id = Self::get_str(&params, "document_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: document_id"))?;
        let requests = params.get("requests")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: requests"))?;

        let requests: Vec<Value> = requests.clone();

        let client = self.client.clone();
        let document_id = document_id.to_string();

        let result = self.runtime.block_on(async move {
            client.batch_update(&document_id, requests).await
        })?;

        Ok(json!(result))
    }
}

impl FgpService for DocsService {
    fn name(&self) -> &str {
        "docs"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "get" | "docs.get" => self.get_document(params),
            "text" | "docs.text" => self.get_text(params),
            "create" | "docs.create" => self.create_document(params),
            "insert" | "docs.insert" => self.insert_text(params),
            "append" | "docs.append" => self.append_text(params),
            "delete" | "docs.delete" => self.delete_content(params),
            "replace" | "docs.replace" => self.replace_text(params),
            "batch_update" | "docs.batch_update" => self.batch_update(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("docs.get", "Get document with full structure")
                .schema(
                    SchemaBuilder::object()
                        .property("document_id", SchemaBuilder::string().description("Document ID"))
                        .required(&["document_id"])
                        .build(),
                ),

            MethodInfo::new("docs.text", "Get document as plain text")
                .schema(
                    SchemaBuilder::object()
                        .property("document_id", SchemaBuilder::string().description("Document ID"))
                        .required(&["document_id"])
                        .build(),
                ),

            MethodInfo::new("docs.create", "Create a new document")
                .schema(
                    SchemaBuilder::object()
                        .property("title", SchemaBuilder::string().description("Document title"))
                        .required(&["title"])
                        .build(),
                ),

            MethodInfo::new("docs.insert", "Insert text at index")
                .schema(
                    SchemaBuilder::object()
                        .property("document_id", SchemaBuilder::string().description("Document ID"))
                        .property("text", SchemaBuilder::string().description("Text to insert"))
                        .property("index", SchemaBuilder::integer().description("Character index (1 = start)"))
                        .required(&["document_id", "text", "index"])
                        .build(),
                ),

            MethodInfo::new("docs.append", "Append text to end of document")
                .schema(
                    SchemaBuilder::object()
                        .property("document_id", SchemaBuilder::string().description("Document ID"))
                        .property("text", SchemaBuilder::string().description("Text to append"))
                        .required(&["document_id", "text"])
                        .build(),
                ),

            MethodInfo::new("docs.delete", "Delete content in range")
                .schema(
                    SchemaBuilder::object()
                        .property("document_id", SchemaBuilder::string().description("Document ID"))
                        .property("start_index", SchemaBuilder::integer().description("Start index"))
                        .property("end_index", SchemaBuilder::integer().description("End index"))
                        .required(&["document_id", "start_index", "end_index"])
                        .build(),
                ),

            MethodInfo::new("docs.replace", "Find and replace text")
                .schema(
                    SchemaBuilder::object()
                        .property("document_id", SchemaBuilder::string().description("Document ID"))
                        .property("find", SchemaBuilder::string().description("Text to find"))
                        .property("replace", SchemaBuilder::string().description("Replacement text"))
                        .property("match_case", SchemaBuilder::boolean().description("Case sensitive"))
                        .required(&["document_id", "find", "replace"])
                        .build(),
                ),

            MethodInfo::new("docs.batch_update", "Raw batch update API")
                .schema(
                    SchemaBuilder::object()
                        .property("document_id", SchemaBuilder::string().description("Document ID"))
                        .property("requests", SchemaBuilder::array().description("Array of update requests"))
                        .required(&["document_id", "requests"])
                        .build(),
                ),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

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

    fn test_service() -> DocsService {
        DocsService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("document_id".to_string(), Value::String("doc".to_string()));
        params.insert("index".to_string(), Value::from(12));
        params.insert("match_case".to_string(), Value::Bool(true));

        assert_eq!(DocsService::get_str(&params, "document_id"), Some("doc"));
        assert_eq!(DocsService::get_str(&params, "missing"), None);
        assert_eq!(DocsService::get_i64(&params, "index"), Some(12));
        assert_eq!(DocsService::get_i64(&params, "missing"), None);
        assert_eq!(DocsService::get_bool(&params, "match_case"), Some(true));
        assert_eq!(DocsService::get_bool(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_schema_required_fields() {
        let service = test_service();
        let methods = service.method_list();

        let insert = methods
            .iter()
            .find(|m| m.name == "docs.insert")
            .expect("docs.insert");
        let insert_schema = insert.schema.as_ref().expect("schema");
        let required = insert_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "document_id"));
        assert!(required.iter().any(|v| v == "text"));
        assert!(required.iter().any(|v| v == "index"));
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("docs.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
