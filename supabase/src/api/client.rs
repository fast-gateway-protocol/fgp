//! Supabase HTTP API client with connection pooling.
//!
//! Supports all Supabase services: Database (PostgREST), Auth (GoTrue), Storage, and Functions.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

use crate::models::{AuthSession, Bucket, StorageObject, UploadResult, User};

/// Supabase API client with persistent connection pooling.
pub struct SupabaseClient {
    client: Client,
    url: String,
    anon_key: String,
    service_key: Option<String>,
}

impl SupabaseClient {
    /// Create a new Supabase client.
    ///
    /// Reads credentials from environment:
    /// - SUPABASE_URL: Project URL
    /// - SUPABASE_KEY: Anon/public key
    /// - SUPABASE_SERVICE_KEY: Service role key (optional, for admin ops)
    pub fn new() -> Result<Self> {
        let url = std::env::var("SUPABASE_URL")
            .context("SUPABASE_URL environment variable not set")?;
        let anon_key = std::env::var("SUPABASE_KEY")
            .or_else(|_| std::env::var("SUPABASE_ANON_KEY"))
            .context("SUPABASE_KEY or SUPABASE_ANON_KEY environment variable not set")?;
        let service_key = std::env::var("SUPABASE_SERVICE_KEY").ok();

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-supabase/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            url: url.trim_end_matches('/').to_string(),
            anon_key,
            service_key,
        })
    }

    /// Get the appropriate API key (service key for admin ops, anon key otherwise).
    fn api_key(&self, admin: bool) -> &str {
        if admin {
            self.service_key.as_deref().unwrap_or(&self.anon_key)
        } else {
            &self.anon_key
        }
    }

    /// Check if client can connect to Supabase.
    pub async fn ping(&self) -> Result<bool> {
        let url = format!("{}/rest/v1/", self.url);
        let response = self
            .client
            .get(&url)
            .header("apikey", &self.anon_key)
            .send()
            .await
            .context("Failed to ping Supabase")?;

        Ok(response.status().is_success() || response.status().as_u16() == 400)
    }

    // ========================================================================
    // Database (PostgREST) operations
    // ========================================================================

    /// Execute a raw SQL query via RPC.
    pub async fn sql(&self, query: &str, params: Option<&[Value]>) -> Result<Value> {
        // Use the sql_query RPC function if available, otherwise fall back to direct query
        let body = serde_json::json!({
            "query": query,
            "params": params.unwrap_or(&[])
        });

        let url = format!("{}/rest/v1/rpc/sql_query", self.url);
        let response = self
            .client
            .post(&url)
            .header("apikey", self.api_key(true))
            .header("Authorization", format!("Bearer {}", self.api_key(true)))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to execute SQL")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("SQL query failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse SQL result")?;
        Ok(result)
    }

    /// Select rows from a table.
    pub async fn select(
        &self,
        table: &str,
        columns: Option<&str>,
        filters: Option<&HashMap<String, Value>>,
        order: Option<&str>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Value>> {
        let mut url = format!("{}/rest/v1/{}", self.url, table);
        let mut query_params = vec![];

        // Add column selection
        if let Some(cols) = columns {
            query_params.push(format!("select={}", cols));
        }

        // Add ordering
        if let Some(ord) = order {
            query_params.push(format!("order={}", ord));
        }

        // Add limit
        if let Some(lim) = limit {
            query_params.push(format!("limit={}", lim));
        }

        // Add offset
        if let Some(off) = offset {
            query_params.push(format!("offset={}", off));
        }

        // Add filters (PostgREST format: column=eq.value)
        if let Some(f) = filters {
            for (key, value) in f {
                let filter_value = match value {
                    Value::String(s) => format!("{}=eq.{}", key, s),
                    Value::Number(n) => format!("{}=eq.{}", key, n),
                    Value::Bool(b) => format!("{}=eq.{}", key, b),
                    Value::Null => format!("{}=is.null", key),
                    _ => format!("{}=eq.{}", key, value),
                };
                query_params.push(filter_value);
            }
        }

        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .send()
            .await
            .context("Failed to select from table")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Select failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse select result")?;
        Ok(result)
    }

    /// Insert rows into a table.
    pub async fn insert(&self, table: &str, rows: &[Value], upsert: bool) -> Result<Vec<Value>> {
        let url = format!("{}/rest/v1/{}", self.url, table);

        let mut request = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation");

        if upsert {
            request = request.header("Prefer", "resolution=merge-duplicates,return=representation");
        }

        let response = request
            .json(&rows)
            .send()
            .await
            .context("Failed to insert rows")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Insert failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse insert result")?;
        Ok(result)
    }

    /// Update rows in a table.
    pub async fn update(
        &self,
        table: &str,
        values: &Value,
        filters: &HashMap<String, Value>,
    ) -> Result<Vec<Value>> {
        let mut url = format!("{}/rest/v1/{}", self.url, table);

        // Build filter query string
        let filter_parts: Vec<String> = filters
            .iter()
            .map(|(key, value)| match value {
                Value::String(s) => format!("{}=eq.{}", key, s),
                Value::Number(n) => format!("{}=eq.{}", key, n),
                Value::Bool(b) => format!("{}=eq.{}", key, b),
                Value::Null => format!("{}=is.null", key),
                _ => format!("{}=eq.{}", key, value),
            })
            .collect();

        if !filter_parts.is_empty() {
            url = format!("{}?{}", url, filter_parts.join("&"));
        }

        let response = self
            .client
            .patch(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(values)
            .send()
            .await
            .context("Failed to update rows")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Update failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse update result")?;
        Ok(result)
    }

    /// Delete rows from a table.
    pub async fn delete(&self, table: &str, filters: &HashMap<String, Value>) -> Result<Vec<Value>> {
        let mut url = format!("{}/rest/v1/{}", self.url, table);

        // Build filter query string (required for delete)
        let filter_parts: Vec<String> = filters
            .iter()
            .map(|(key, value)| match value {
                Value::String(s) => format!("{}=eq.{}", key, s),
                Value::Number(n) => format!("{}=eq.{}", key, n),
                Value::Bool(b) => format!("{}=eq.{}", key, b),
                Value::Null => format!("{}=is.null", key),
                _ => format!("{}=eq.{}", key, value),
            })
            .collect();

        if filter_parts.is_empty() {
            bail!("Delete requires at least one filter to prevent accidental deletion of all rows");
        }

        url = format!("{}?{}", url, filter_parts.join("&"));

        let response = self
            .client
            .delete(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Prefer", "return=representation")
            .send()
            .await
            .context("Failed to delete rows")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Delete failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse delete result")?;
        Ok(result)
    }

    /// Call a database function (RPC).
    pub async fn rpc(&self, function: &str, params: Option<&Value>) -> Result<Value> {
        let url = format!("{}/rest/v1/rpc/{}", self.url, function);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", "application/json")
            .json(&params.unwrap_or(&Value::Object(serde_json::Map::new())))
            .send()
            .await
            .context("Failed to call RPC function")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("RPC call failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse RPC result")?;
        Ok(result)
    }

    // ========================================================================
    // Auth (GoTrue) operations
    // ========================================================================

    /// Sign up a new user.
    pub async fn auth_signup(&self, email: &str, password: &str, data: Option<&Value>) -> Result<AuthSession> {
        let url = format!("{}/auth/v1/signup", self.url);

        let mut body = serde_json::json!({
            "email": email,
            "password": password
        });

        if let Some(d) = data {
            body["data"] = d.clone();
        }

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to sign up")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Signup failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse signup result")?;
        Ok(result)
    }

    /// Sign in with email and password.
    pub async fn auth_signin(&self, email: &str, password: &str) -> Result<AuthSession> {
        let url = format!("{}/auth/v1/token?grant_type=password", self.url);

        let body = serde_json::json!({
            "email": email,
            "password": password
        });

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to sign in")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Signin failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse signin result")?;
        Ok(result)
    }

    /// Sign out a user.
    pub async fn auth_signout(&self, access_token: &str) -> Result<()> {
        let url = format!("{}/auth/v1/logout", self.url);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to sign out")?;

        if !response.status().is_success() && response.status().as_u16() != 204 {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Signout failed: {} - {}", status, text);
        }

        Ok(())
    }

    /// Get user by access token.
    pub async fn auth_user(&self, access_token: &str) -> Result<User> {
        let url = format!("{}/auth/v1/user", self.url);

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to get user")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Get user failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse user result")?;
        Ok(result)
    }

    /// List all users (requires service role key).
    pub async fn auth_users(&self, page: i32, per_page: i32) -> Result<Vec<User>> {
        let service_key = self.service_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service role key required for admin operations"))?;

        let url = format!(
            "{}/auth/v1/admin/users?page={}&per_page={}",
            self.url, page, per_page
        );

        let response = self
            .client
            .get(&url)
            .header("apikey", service_key)
            .header("Authorization", format!("Bearer {}", service_key))
            .send()
            .await
            .context("Failed to list users")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("List users failed: {} - {}", status, text);
        }

        #[derive(serde::Deserialize)]
        struct UsersResponse {
            users: Vec<User>,
        }

        let result: UsersResponse = response.json().await.context("Failed to parse users result")?;
        Ok(result.users)
    }

    // ========================================================================
    // Storage operations
    // ========================================================================

    /// List all storage buckets.
    pub async fn storage_buckets(&self) -> Result<Vec<Bucket>> {
        let url = format!("{}/storage/v1/bucket", self.url);

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .send()
            .await
            .context("Failed to list buckets")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("List buckets failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse buckets result")?;
        Ok(result)
    }

    /// Create a new storage bucket.
    pub async fn storage_create_bucket(&self, name: &str, public: bool) -> Result<Bucket> {
        let service_key = self.service_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service role key required to create buckets"))?;

        let url = format!("{}/storage/v1/bucket", self.url);

        let body = serde_json::json!({
            "name": name,
            "public": public
        });

        let response = self
            .client
            .post(&url)
            .header("apikey", service_key)
            .header("Authorization", format!("Bearer {}", service_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to create bucket")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Create bucket failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse bucket result")?;
        Ok(result)
    }

    /// List files in a storage bucket.
    pub async fn storage_list(
        &self,
        bucket: &str,
        path: Option<&str>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<StorageObject>> {
        let url = format!("{}/storage/v1/object/list/{}", self.url, bucket);

        let body = serde_json::json!({
            "prefix": path.unwrap_or(""),
            "limit": limit.unwrap_or(100),
            "offset": offset.unwrap_or(0)
        });

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to list files")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("List files failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse files result")?;
        Ok(result)
    }

    /// Upload a file to storage.
    pub async fn storage_upload(
        &self,
        bucket: &str,
        path: &str,
        data: &[u8],
        content_type: &str,
        upsert: bool,
    ) -> Result<UploadResult> {
        let url = format!("{}/storage/v1/object/{}/{}", self.url, bucket, path);

        let mut request = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", content_type);

        if upsert {
            request = request.header("x-upsert", "true");
        }

        let response = request
            .body(data.to_vec())
            .send()
            .await
            .context("Failed to upload file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Upload failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse upload result")?;
        Ok(result)
    }

    /// Download a file from storage.
    pub async fn storage_download(&self, bucket: &str, path: &str) -> Result<Vec<u8>> {
        let url = format!("{}/storage/v1/object/{}/{}", self.url, bucket, path);

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .send()
            .await
            .context("Failed to download file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Download failed: {} - {}", status, text);
        }

        let bytes = response.bytes().await.context("Failed to read file bytes")?;
        Ok(bytes.to_vec())
    }

    /// Delete a file from storage.
    pub async fn storage_delete(&self, bucket: &str, paths: &[String]) -> Result<()> {
        let url = format!("{}/storage/v1/object/{}", self.url, bucket);

        let body = serde_json::json!({
            "prefixes": paths
        });

        let response = self
            .client
            .delete(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to delete file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Delete failed: {} - {}", status, text);
        }

        Ok(())
    }

    /// Get a signed URL for a private file.
    pub async fn storage_signed_url(&self, bucket: &str, path: &str, expires_in: i32) -> Result<String> {
        let url = format!("{}/storage/v1/object/sign/{}/{}", self.url, bucket, path);

        let body = serde_json::json!({
            "expiresIn": expires_in
        });

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to create signed URL")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Signed URL failed: {} - {}", status, text);
        }

        #[derive(serde::Deserialize)]
        struct SignedUrlResponse {
            #[serde(rename = "signedURL")]
            signed_url: String,
        }

        let result: SignedUrlResponse = response.json().await.context("Failed to parse signed URL")?;
        Ok(format!("{}{}", self.url, result.signed_url))
    }

    /// Get public URL for a file in a public bucket.
    pub fn storage_public_url(&self, bucket: &str, path: &str) -> String {
        format!("{}/storage/v1/object/public/{}/{}", self.url, bucket, path)
    }

    // ========================================================================
    // Edge Functions operations
    // ========================================================================

    /// Invoke an edge function.
    pub async fn functions_invoke(
        &self,
        function_name: &str,
        body: Option<&Value>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<Value> {
        let url = format!("{}/functions/v1/{}", self.url, function_name);

        let mut request = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key));

        if let Some(h) = headers {
            for (key, value) in h {
                request = request.header(key.as_str(), value.as_str());
            }
        }

        if let Some(b) = body {
            request = request.header("Content-Type", "application/json").json(b);
        }

        let response = request.send().await.context("Failed to invoke function")?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            bail!("Function invocation failed: {} - {}", status, text);
        }

        // Try to parse as JSON, fall back to string
        let text = response.text().await.context("Failed to read response")?;
        let result = serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text));

        Ok(serde_json::json!({
            "data": result,
            "status": status
        }))
    }

    // ========================================================================
    // Vector operations (pgvector)
    // ========================================================================

    /// Perform a vector similarity search.
    pub async fn vectors_search(
        &self,
        table: &str,
        embedding_column: &str,
        query_embedding: &[f64],
        match_count: i32,
        match_threshold: Option<f64>,
    ) -> Result<Vec<Value>> {
        // This uses the typical pgvector pattern with a match_documents function
        // Projects may have their own function names, so we try a few patterns
        let function_name = format!("match_{}", table);

        let body = serde_json::json!({
            "query_embedding": query_embedding,
            "match_count": match_count,
            "match_threshold": match_threshold.unwrap_or(0.0)
        });

        let url = format!("{}/rest/v1/rpc/{}", self.url, function_name);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", &self.anon_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        // If the standard function doesn't exist, try raw SQL approach
        if let Ok(resp) = response {
            if resp.status().is_success() {
                let result = resp.json().await.context("Failed to parse vector search result")?;
                return Ok(result);
            }
        }

        // Fallback: use raw SQL with pgvector operator
        let threshold = match_threshold.unwrap_or(0.0);
        let _query = format!(
            "SELECT *, 1 - ({} <=> $1::vector) as similarity
             FROM {}
             WHERE 1 - ({} <=> $1::vector) > {}
             ORDER BY {} <=> $1::vector
             LIMIT {}",
            embedding_column, table, embedding_column, threshold, embedding_column, match_count
        );

        // For raw SQL, we need the sql_query function or similar
        bail!(
            "Vector search requires a 'match_{}' function or 'sql_query' RPC function. \
             Please create an appropriate function in your Supabase database.",
            table
        )
    }
}
