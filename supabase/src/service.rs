//! FGP service implementation for Supabase.
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

use crate::api::SupabaseClient;

/// FGP service for Supabase operations.
pub struct SupabaseService {
    client: Arc<SupabaseClient>,
    runtime: Runtime,
}

impl SupabaseService {
    /// Create a new SupabaseService.
    pub fn new() -> Result<Self> {
        let client = SupabaseClient::new()?;
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

    /// Helper to get an i32 parameter with default.
    fn get_i32(params: &HashMap<String, Value>, key: &str, default: i32) -> i32 {
        params
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default)
    }

    /// Helper to get bool parameter.
    fn get_bool(params: &HashMap<String, Value>, key: &str, default: bool) -> bool {
        params
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    // ========================================================================
    // Database methods
    // ========================================================================

    fn db_sql(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_str(&params, "query")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;

        let sql_params = params.get("params").and_then(|v| v.as_array()).map(|a| a.as_slice());

        let client = self.client.clone();
        let query = query.to_string();

        let result = self.runtime.block_on(async move {
            client.sql(&query, sql_params).await
        })?;

        Ok(result)
    }

    fn db_select(&self, params: HashMap<String, Value>) -> Result<Value> {
        let table = Self::get_str(&params, "table")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: table"))?;
        let columns = Self::get_str(&params, "columns");
        let order = Self::get_str(&params, "order");
        let limit = params.get("limit").and_then(|v| v.as_i64()).map(|v| v as i32);
        let offset = params.get("offset").and_then(|v| v.as_i64()).map(|v| v as i32);

        // Parse filters object
        let filters: Option<HashMap<String, Value>> = params
            .get("filters")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

        let client = self.client.clone();
        let table = table.to_string();
        let columns = columns.map(|s| s.to_string());
        let order = order.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client
                .select(
                    &table,
                    columns.as_deref(),
                    filters.as_ref(),
                    order.as_deref(),
                    limit,
                    offset,
                )
                .await
        })?;

        Ok(json!({
            "rows": result,
            "count": result.len()
        }))
    }

    fn db_insert(&self, params: HashMap<String, Value>) -> Result<Value> {
        let table = Self::get_str(&params, "table")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: table"))?;
        let upsert = Self::get_bool(&params, "upsert", false);

        let rows = params
            .get("rows")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: rows"))?;

        let rows_vec: Vec<Value> = if rows.is_array() {
            rows.as_array().unwrap().clone()
        } else {
            vec![rows.clone()]
        };

        let client = self.client.clone();
        let table = table.to_string();

        let result = self.runtime.block_on(async move {
            client.insert(&table, &rows_vec, upsert).await
        })?;

        Ok(json!({
            "inserted": result,
            "count": result.len()
        }))
    }

    fn db_update(&self, params: HashMap<String, Value>) -> Result<Value> {
        let table = Self::get_str(&params, "table")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: table"))?;

        let values = params
            .get("values")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: values"))?;

        let filters: HashMap<String, Value> = params
            .get("filters")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: filters"))?;

        let client = self.client.clone();
        let table = table.to_string();
        let values = values.clone();

        let result = self.runtime.block_on(async move {
            client.update(&table, &values, &filters).await
        })?;

        Ok(json!({
            "updated": result,
            "count": result.len()
        }))
    }

    fn db_delete(&self, params: HashMap<String, Value>) -> Result<Value> {
        let table = Self::get_str(&params, "table")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: table"))?;

        let filters: HashMap<String, Value> = params
            .get("filters")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: filters"))?;

        let client = self.client.clone();
        let table = table.to_string();

        let result = self.runtime.block_on(async move {
            client.delete(&table, &filters).await
        })?;

        Ok(json!({
            "deleted": result,
            "count": result.len()
        }))
    }

    fn db_rpc(&self, params: HashMap<String, Value>) -> Result<Value> {
        let function = Self::get_str(&params, "function")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: function"))?;

        let rpc_params = params.get("params");

        let client = self.client.clone();
        let function = function.to_string();
        let rpc_params = rpc_params.cloned();

        let result = self.runtime.block_on(async move {
            client.rpc(&function, rpc_params.as_ref()).await
        })?;

        Ok(result)
    }

    // ========================================================================
    // Auth methods
    // ========================================================================

    fn auth_signup(&self, params: HashMap<String, Value>) -> Result<Value> {
        let email = Self::get_str(&params, "email")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: email"))?;
        let password = Self::get_str(&params, "password")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: password"))?;
        let data = params.get("data");

        let client = self.client.clone();
        let email = email.to_string();
        let password = password.to_string();
        let data = data.cloned();

        let result = self.runtime.block_on(async move {
            client.auth_signup(&email, &password, data.as_ref()).await
        })?;

        Ok(json!(result))
    }

    fn auth_signin(&self, params: HashMap<String, Value>) -> Result<Value> {
        let email = Self::get_str(&params, "email")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: email"))?;
        let password = Self::get_str(&params, "password")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: password"))?;

        let client = self.client.clone();
        let email = email.to_string();
        let password = password.to_string();

        let result = self.runtime.block_on(async move {
            client.auth_signin(&email, &password).await
        })?;

        Ok(json!(result))
    }

    fn auth_signout(&self, params: HashMap<String, Value>) -> Result<Value> {
        let token = Self::get_str(&params, "access_token")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: access_token"))?;

        let client = self.client.clone();
        let token = token.to_string();

        self.runtime.block_on(async move {
            client.auth_signout(&token).await
        })?;

        Ok(json!({"success": true}))
    }

    fn auth_user(&self, params: HashMap<String, Value>) -> Result<Value> {
        let token = Self::get_str(&params, "access_token")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: access_token"))?;

        let client = self.client.clone();
        let token = token.to_string();

        let result = self.runtime.block_on(async move {
            client.auth_user(&token).await
        })?;

        Ok(json!(result))
    }

    fn auth_users(&self, params: HashMap<String, Value>) -> Result<Value> {
        let page = Self::get_i32(&params, "page", 1);
        let per_page = Self::get_i32(&params, "per_page", 50);

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.auth_users(page, per_page).await
        })?;

        Ok(json!({
            "users": result,
            "count": result.len()
        }))
    }

    // ========================================================================
    // Storage methods
    // ========================================================================

    fn storage_buckets(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.storage_buckets().await
        })?;

        Ok(json!({
            "buckets": result,
            "count": result.len()
        }))
    }

    fn storage_create_bucket(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let public = Self::get_bool(&params, "public", false);

        let client = self.client.clone();
        let name = name.to_string();

        let result = self.runtime.block_on(async move {
            client.storage_create_bucket(&name, public).await
        })?;

        Ok(json!(result))
    }

    fn storage_list(&self, params: HashMap<String, Value>) -> Result<Value> {
        let bucket = Self::get_str(&params, "bucket")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: bucket"))?;
        let path = Self::get_str(&params, "path");
        let limit = params.get("limit").and_then(|v| v.as_i64()).map(|v| v as i32);
        let offset = params.get("offset").and_then(|v| v.as_i64()).map(|v| v as i32);

        let client = self.client.clone();
        let bucket = bucket.to_string();
        let path = path.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.storage_list(&bucket, path.as_deref(), limit, offset).await
        })?;

        Ok(json!({
            "files": result,
            "count": result.len()
        }))
    }

    fn storage_upload(&self, params: HashMap<String, Value>) -> Result<Value> {
        let bucket = Self::get_str(&params, "bucket")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: bucket"))?;
        let path = Self::get_str(&params, "path")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        let content_type = Self::get_str(&params, "content_type").unwrap_or("application/octet-stream");
        let upsert = Self::get_bool(&params, "upsert", false);

        // Get file data (base64 encoded)
        let data_b64 = Self::get_str(&params, "data")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: data (base64)"))?;

        use base64::Engine;
        let data = base64::engine::general_purpose::STANDARD
            .decode(data_b64)
            .map_err(|e| anyhow::anyhow!("Invalid base64 data: {}", e))?;

        let client = self.client.clone();
        let bucket = bucket.to_string();
        let path = path.to_string();
        let content_type = content_type.to_string();

        let result = self.runtime.block_on(async move {
            client.storage_upload(&bucket, &path, &data, &content_type, upsert).await
        })?;

        Ok(json!(result))
    }

    fn storage_download(&self, params: HashMap<String, Value>) -> Result<Value> {
        let bucket = Self::get_str(&params, "bucket")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: bucket"))?;
        let path = Self::get_str(&params, "path")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;

        let client = self.client.clone();
        let bucket = bucket.to_string();
        let path = path.to_string();

        let result = self.runtime.block_on(async move {
            client.storage_download(&bucket, &path).await
        })?;

        // Return as base64
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&result);

        Ok(json!({
            "data": b64,
            "size": result.len()
        }))
    }

    fn storage_delete(&self, params: HashMap<String, Value>) -> Result<Value> {
        let bucket = Self::get_str(&params, "bucket")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: bucket"))?;

        let paths: Vec<String> = if let Some(path) = Self::get_str(&params, "path") {
            vec![path.to_string()]
        } else if let Some(paths_array) = params.get("paths").and_then(|v| v.as_array()) {
            paths_array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            return Err(anyhow::anyhow!("Missing required parameter: path or paths"));
        };

        let client = self.client.clone();
        let bucket = bucket.to_string();
        let paths_clone = paths.clone();

        self.runtime.block_on(async move {
            client.storage_delete(&bucket, &paths_clone).await
        })?;

        Ok(json!({"success": true, "deleted": paths}))
    }

    fn storage_signed_url(&self, params: HashMap<String, Value>) -> Result<Value> {
        let bucket = Self::get_str(&params, "bucket")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: bucket"))?;
        let path = Self::get_str(&params, "path")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        let expires_in = Self::get_i32(&params, "expires_in", 3600);

        let client = self.client.clone();
        let bucket = bucket.to_string();
        let path = path.to_string();

        let result = self.runtime.block_on(async move {
            client.storage_signed_url(&bucket, &path, expires_in).await
        })?;

        Ok(json!({"signed_url": result}))
    }

    fn storage_public_url(&self, params: HashMap<String, Value>) -> Result<Value> {
        let bucket = Self::get_str(&params, "bucket")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: bucket"))?;
        let path = Self::get_str(&params, "path")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;

        let url = self.client.storage_public_url(bucket, path);

        Ok(json!({"public_url": url}))
    }

    // ========================================================================
    // Functions methods
    // ========================================================================

    fn functions_invoke(&self, params: HashMap<String, Value>) -> Result<Value> {
        let function_name = Self::get_str(&params, "function")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: function"))?;

        let body = params.get("body");
        let headers: Option<HashMap<String, String>> = params
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            });

        let client = self.client.clone();
        let function_name = function_name.to_string();
        let body = body.cloned();

        let result = self.runtime.block_on(async move {
            client.functions_invoke(&function_name, body.as_ref(), headers.as_ref()).await
        })?;

        Ok(result)
    }

    // ========================================================================
    // Vector methods
    // ========================================================================

    fn vectors_search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let table = Self::get_str(&params, "table")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: table"))?;
        let embedding_column = Self::get_str(&params, "embedding_column").unwrap_or("embedding");
        let match_count = Self::get_i32(&params, "match_count", 10);
        let match_threshold = params.get("match_threshold").and_then(|v| v.as_f64());

        let query_embedding: Vec<f64> = params
            .get("query_embedding")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query_embedding"))?;

        let client = self.client.clone();
        let table = table.to_string();
        let embedding_column = embedding_column.to_string();

        let result = self.runtime.block_on(async move {
            client.vectors_search(&table, &embedding_column, &query_embedding, match_count, match_threshold).await
        })?;

        Ok(json!({
            "matches": result,
            "count": result.len()
        }))
    }
}

impl FgpService for SupabaseService {
    fn name(&self) -> &str {
        "supabase"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // Database
            "sql" | "supabase.sql" => self.db_sql(params),
            "select" | "supabase.select" => self.db_select(params),
            "insert" | "supabase.insert" => self.db_insert(params),
            "update" | "supabase.update" => self.db_update(params),
            "delete" | "supabase.delete" => self.db_delete(params),
            "rpc" | "supabase.rpc" => self.db_rpc(params),

            // Auth
            "auth.signup" | "supabase.auth.signup" => self.auth_signup(params),
            "auth.signin" | "supabase.auth.signin" => self.auth_signin(params),
            "auth.signout" | "supabase.auth.signout" => self.auth_signout(params),
            "auth.user" | "supabase.auth.user" => self.auth_user(params),
            "auth.users" | "supabase.auth.users" => self.auth_users(params),

            // Storage
            "storage.buckets" | "supabase.storage.buckets" => self.storage_buckets(params),
            "storage.create_bucket" | "supabase.storage.create_bucket" => self.storage_create_bucket(params),
            "storage.list" | "supabase.storage.list" => self.storage_list(params),
            "storage.upload" | "supabase.storage.upload" => self.storage_upload(params),
            "storage.download" | "supabase.storage.download" => self.storage_download(params),
            "storage.delete" | "supabase.storage.delete" => self.storage_delete(params),
            "storage.signed_url" | "supabase.storage.signed_url" => self.storage_signed_url(params),
            "storage.public_url" | "supabase.storage.public_url" => self.storage_public_url(params),

            // Functions
            "functions.invoke" | "supabase.functions.invoke" => self.functions_invoke(params),

            // Vectors
            "vectors.search" | "supabase.vectors.search" => self.vectors_search(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // Database methods
            MethodInfo::new("supabase.sql", "Execute raw SQL query")
                .schema(
                    SchemaBuilder::object()
                        .property("query", SchemaBuilder::string().description("SQL query to execute"))
                        .property("params", SchemaBuilder::array().description("Query parameters"))
                        .required(&["query"])
                        .build(),
                ),
            MethodInfo::new("supabase.select", "Select rows from a table")
                .schema(
                    SchemaBuilder::object()
                        .property("table", SchemaBuilder::string().description("Table name"))
                        .property("columns", SchemaBuilder::string().description("Columns to select (default: *)"))
                        .property("filters", SchemaBuilder::object().description("Filter conditions"))
                        .property("order", SchemaBuilder::string().description("Order by clause"))
                        .property("limit", SchemaBuilder::integer().description("Maximum rows"))
                        .property("offset", SchemaBuilder::integer().description("Rows to skip"))
                        .required(&["table"])
                        .build(),
                ),
            MethodInfo::new("supabase.insert", "Insert rows into a table")
                .schema(
                    SchemaBuilder::object()
                        .property("table", SchemaBuilder::string())
                        .property("rows", SchemaBuilder::array().description("Row(s) to insert"))
                        .property("upsert", SchemaBuilder::boolean().default_value(json!(false)))
                        .required(&["table", "rows"])
                        .build(),
                ),
            MethodInfo::new("supabase.update", "Update rows in a table")
                .schema(
                    SchemaBuilder::object()
                        .property("table", SchemaBuilder::string())
                        .property("values", SchemaBuilder::object().description("Values to update"))
                        .property("filters", SchemaBuilder::object().description("Filter conditions"))
                        .required(&["table", "values", "filters"])
                        .build(),
                ),
            MethodInfo::new("supabase.delete", "Delete rows from a table")
                .schema(
                    SchemaBuilder::object()
                        .property("table", SchemaBuilder::string())
                        .property("filters", SchemaBuilder::object().description("Filter conditions (required)"))
                        .required(&["table", "filters"])
                        .build(),
                ),
            MethodInfo::new("supabase.rpc", "Call a database function")
                .schema(
                    SchemaBuilder::object()
                        .property("function", SchemaBuilder::string().description("Function name"))
                        .property("params", SchemaBuilder::object().description("Function parameters"))
                        .required(&["function"])
                        .build(),
                ),

            // Auth methods
            MethodInfo::new("supabase.auth.signup", "Create a new user")
                .schema(
                    SchemaBuilder::object()
                        .property("email", SchemaBuilder::string().format("email"))
                        .property("password", SchemaBuilder::string())
                        .property("data", SchemaBuilder::object().description("User metadata"))
                        .required(&["email", "password"])
                        .build(),
                ),
            MethodInfo::new("supabase.auth.signin", "Sign in with email and password")
                .schema(
                    SchemaBuilder::object()
                        .property("email", SchemaBuilder::string().format("email"))
                        .property("password", SchemaBuilder::string())
                        .required(&["email", "password"])
                        .build(),
                ),
            MethodInfo::new("supabase.auth.signout", "Sign out a user")
                .schema(
                    SchemaBuilder::object()
                        .property("access_token", SchemaBuilder::string())
                        .required(&["access_token"])
                        .build(),
                ),
            MethodInfo::new("supabase.auth.user", "Get user by access token")
                .schema(
                    SchemaBuilder::object()
                        .property("access_token", SchemaBuilder::string())
                        .required(&["access_token"])
                        .build(),
                ),
            MethodInfo::new("supabase.auth.users", "List all users (admin)")
                .schema(
                    SchemaBuilder::object()
                        .property("page", SchemaBuilder::integer().default_value(json!(1)))
                        .property("per_page", SchemaBuilder::integer().default_value(json!(50)))
                        .build(),
                ),

            // Storage methods
            MethodInfo::new("supabase.storage.buckets", "List all storage buckets"),
            MethodInfo::new("supabase.storage.create_bucket", "Create a storage bucket")
                .schema(
                    SchemaBuilder::object()
                        .property("name", SchemaBuilder::string())
                        .property("public", SchemaBuilder::boolean().default_value(json!(false)))
                        .required(&["name"])
                        .build(),
                ),
            MethodInfo::new("supabase.storage.list", "List files in a bucket")
                .schema(
                    SchemaBuilder::object()
                        .property("bucket", SchemaBuilder::string())
                        .property("path", SchemaBuilder::string().description("Folder path prefix"))
                        .property("limit", SchemaBuilder::integer())
                        .property("offset", SchemaBuilder::integer())
                        .required(&["bucket"])
                        .build(),
                ),
            MethodInfo::new("supabase.storage.upload", "Upload a file")
                .schema(
                    SchemaBuilder::object()
                        .property("bucket", SchemaBuilder::string())
                        .property("path", SchemaBuilder::string())
                        .property("data", SchemaBuilder::string().description("Base64-encoded file data"))
                        .property("content_type", SchemaBuilder::string())
                        .property("upsert", SchemaBuilder::boolean())
                        .required(&["bucket", "path", "data"])
                        .build(),
                ),
            MethodInfo::new("supabase.storage.download", "Download a file")
                .schema(
                    SchemaBuilder::object()
                        .property("bucket", SchemaBuilder::string())
                        .property("path", SchemaBuilder::string())
                        .required(&["bucket", "path"])
                        .build(),
                ),
            MethodInfo::new("supabase.storage.delete", "Delete file(s)")
                .schema(
                    SchemaBuilder::object()
                        .property("bucket", SchemaBuilder::string())
                        .property("path", SchemaBuilder::string().description("Single file path"))
                        .property("paths", SchemaBuilder::array().description("Multiple file paths"))
                        .required(&["bucket"])
                        .build(),
                ),
            MethodInfo::new("supabase.storage.signed_url", "Get a signed URL")
                .schema(
                    SchemaBuilder::object()
                        .property("bucket", SchemaBuilder::string())
                        .property("path", SchemaBuilder::string())
                        .property("expires_in", SchemaBuilder::integer().default_value(json!(3600)))
                        .required(&["bucket", "path"])
                        .build(),
                ),
            MethodInfo::new("supabase.storage.public_url", "Get public URL for a file")
                .schema(
                    SchemaBuilder::object()
                        .property("bucket", SchemaBuilder::string())
                        .property("path", SchemaBuilder::string())
                        .required(&["bucket", "path"])
                        .build(),
                ),

            // Functions
            MethodInfo::new("supabase.functions.invoke", "Invoke an edge function")
                .schema(
                    SchemaBuilder::object()
                        .property("function", SchemaBuilder::string().description("Function name"))
                        .property("body", SchemaBuilder::object().description("Request body"))
                        .property("headers", SchemaBuilder::object().description("Custom headers"))
                        .required(&["function"])
                        .build(),
                ),

            // Vectors
            MethodInfo::new("supabase.vectors.search", "Vector similarity search")
                .schema(
                    SchemaBuilder::object()
                        .property("table", SchemaBuilder::string())
                        .property("query_embedding", SchemaBuilder::array().description("Query vector"))
                        .property("embedding_column", SchemaBuilder::string().default_value(json!("embedding")))
                        .property("match_count", SchemaBuilder::integer().default_value(json!(10)))
                        .property("match_threshold", SchemaBuilder::number())
                        .required(&["table", "query_embedding"])
                        .build(),
                ),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("SupabaseService starting, verifying API connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Supabase API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Supabase API returned unexpected response");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Supabase API: {}", e);
                    Err(e)
                }
            }
        })
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let client = self.client.clone();
        let start = std::time::Instant::now();
        let result = self.runtime.block_on(async move { client.ping().await });

        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(true) => {
                checks.insert(
                    "supabase_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "supabase_api".into(),
                    HealthStatus::unhealthy("Unexpected response"),
                );
            }
            Err(e) => {
                checks.insert("supabase_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_env() {
        INIT.call_once(|| {
            std::env::set_var("SUPABASE_URL", "https://example.supabase.co");
            std::env::set_var("SUPABASE_KEY", "test-anon-key");
        });
    }

    fn test_service() -> SupabaseService {
        ensure_env();
        SupabaseService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), Value::String("Ada".to_string()));
        params.insert("count".to_string(), Value::from(7));
        params.insert("enabled".to_string(), Value::Bool(true));

        assert_eq!(SupabaseService::get_str(&params, "name"), Some("Ada"));
        assert_eq!(SupabaseService::get_str(&params, "missing"), None);
        assert_eq!(SupabaseService::get_i32(&params, "count", 3), 7);
        assert_eq!(SupabaseService::get_i32(&params, "missing", 3), 3);
        assert!(SupabaseService::get_bool(&params, "enabled", false));
        assert!(!SupabaseService::get_bool(&params, "missing", false));
    }

    #[test]
    fn test_method_list_defaults() {
        let service = test_service();
        let methods = service.method_list();

        let auth_users = methods
            .iter()
            .find(|m| m.name == "supabase.auth.users")
            .expect("auth users");
        let auth_schema = auth_users.schema.as_ref().expect("schema");
        let auth_props = auth_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(
            auth_props
                .get("page")
                .and_then(Value::as_object)
                .and_then(|p| p.get("default")),
            Some(&json!(1))
        );
        assert_eq!(
            auth_props
                .get("per_page")
                .and_then(Value::as_object)
                .and_then(|p| p.get("default")),
            Some(&json!(50))
        );

        let signed_url = methods
            .iter()
            .find(|m| m.name == "supabase.storage.signed_url")
            .expect("signed_url");
        let signed_schema = signed_url.schema.as_ref().expect("schema");
        let signed_props = signed_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(
            signed_props
                .get("expires_in")
                .and_then(Value::as_object)
                .and_then(|p| p.get("default")),
            Some(&json!(3600))
        );

        let vectors = methods
            .iter()
            .find(|m| m.name == "supabase.vectors.search")
            .expect("vectors");
        let vectors_schema = vectors.schema.as_ref().expect("schema");
        let vectors_props = vectors_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(
            vectors_props
                .get("embedding_column")
                .and_then(Value::as_object)
                .and_then(|p| p.get("default")),
            Some(&json!("embedding"))
        );
        assert_eq!(
            vectors_props
                .get("match_count")
                .and_then(Value::as_object)
                .and_then(|p| p.get("default")),
            Some(&json!(10))
        );
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("supabase.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
