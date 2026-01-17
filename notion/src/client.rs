//! Notion REST API client with connection pooling.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const API_BASE: &str = "https://api.notion.com/v1";
const NOTION_VERSION: &str = "2022-06-28";

/// Notion API client with persistent connection pooling.
pub struct NotionClient {
    client: Client,
    api_key: String,
}

impl NotionClient {
    /// Create a new Notion client.
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-notion/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, api_key })
    }

    /// Execute a GET request.
    async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", API_BASE, path);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .context("Failed to send request")?;

        self.handle_response(response).await
    }

    /// Execute a POST request.
    async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let url = format!("{}{}", API_BASE, path);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request")?;

        self.handle_response(response).await
    }

    /// Execute a PATCH request.
    async fn patch(&self, path: &str, body: Value) -> Result<Value> {
        let url = format!("{}{}", API_BASE, path);

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request")?;

        self.handle_response(response).await
    }

    /// Handle API response.
    async fn handle_response(&self, response: reqwest::Response) -> Result<Value> {
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();

            // Parse Notion error format
            if let Ok(error) = serde_json::from_str::<NotionError>(&text) {
                bail!("Notion API error: {} - {}", error.code, error.message);
            }

            bail!("Notion API request failed: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse JSON")
    }

    /// Check API connectivity.
    pub async fn ping(&self) -> Result<bool> {
        // List users to verify connectivity
        let result = self.get("/users/me").await;
        Ok(result.is_ok())
    }

    /// Get current user (bot) info.
    pub async fn me(&self) -> Result<Value> {
        self.get("/users/me").await
    }

    /// List workspace users.
    pub async fn users(&self) -> Result<Vec<Value>> {
        let result = self.get("/users").await?;

        let users = result
            .get("results")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(users)
    }

    /// Search pages and databases.
    pub async fn search(
        &self,
        query: Option<&str>,
        filter: Option<&str>,
        limit: i32,
    ) -> Result<Vec<Value>> {
        let mut body = json!({
            "page_size": limit
        });

        if let Some(q) = query {
            body["query"] = json!(q);
        }

        if let Some(f) = filter {
            body["filter"] = json!({
                "value": f,
                "property": "object"
            });
        }

        let result = self.post("/search", body).await?;

        let results = result
            .get("results")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(results)
    }

    /// Get a page by ID.
    pub async fn page(&self, page_id: &str) -> Result<Value> {
        let id = normalize_id(page_id);
        self.get(&format!("/pages/{}", id)).await
    }

    /// Get a database by ID.
    pub async fn database(&self, database_id: &str) -> Result<Value> {
        let id = normalize_id(database_id);
        self.get(&format!("/databases/{}", id)).await
    }

    /// Query a database.
    pub async fn query_database(
        &self,
        database_id: &str,
        filter: Option<Value>,
        sorts: Option<Value>,
        limit: i32,
    ) -> Result<Vec<Value>> {
        let id = normalize_id(database_id);

        let mut body = json!({
            "page_size": limit
        });

        if let Some(f) = filter {
            body["filter"] = f;
        }

        if let Some(s) = sorts {
            body["sorts"] = s;
        }

        let result = self.post(&format!("/databases/{}/query", id), body).await?;

        let results = result
            .get("results")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(results)
    }

    /// Get blocks for a page/block.
    pub async fn blocks(&self, block_id: &str, recursive: bool) -> Result<Vec<Value>> {
        let id = normalize_id(block_id);
        let result = self.get(&format!("/blocks/{}/children", id)).await?;

        let mut blocks = result
            .get("results")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        // Recursively fetch children if requested
        if recursive {
            let mut all_blocks = Vec::new();
            for block in blocks {
                all_blocks.push(block.clone());

                // Check if block has children
                if block.get("has_children").and_then(|h| h.as_bool()).unwrap_or(false) {
                    if let Some(id) = block.get("id").and_then(|i| i.as_str()) {
                        if let Ok(children) = Box::pin(self.blocks(id, true)).await {
                            all_blocks.extend(children);
                        }
                    }
                }
            }
            blocks = all_blocks;
        }

        Ok(blocks)
    }

    /// Create a page in a database.
    pub async fn create_page(
        &self,
        database_id: &str,
        properties: Value,
        children: Option<Vec<Value>>,
    ) -> Result<Value> {
        let db_id = normalize_id(database_id);

        let mut body = json!({
            "parent": {
                "database_id": db_id
            },
            "properties": properties
        });

        if let Some(c) = children {
            body["children"] = json!(c);
        }

        self.post("/pages", body).await
    }

    /// Update a page's properties.
    pub async fn update_page(&self, page_id: &str, properties: Value) -> Result<Value> {
        let id = normalize_id(page_id);

        let body = json!({
            "properties": properties
        });

        self.patch(&format!("/pages/{}", id), body).await
    }

    /// Append blocks to a page.
    pub async fn append_blocks(&self, block_id: &str, children: Vec<Value>) -> Result<Value> {
        let id = normalize_id(block_id);

        let body = json!({
            "children": children
        });

        self.patch(&format!("/blocks/{}/children", id), body).await
    }

    /// Get a single block.
    pub async fn block(&self, block_id: &str) -> Result<Value> {
        let id = normalize_id(block_id);
        self.get(&format!("/blocks/{}", id)).await
    }

    /// Delete (archive) a block.
    pub async fn delete_block(&self, block_id: &str) -> Result<Value> {
        let id = normalize_id(block_id);
        let url = format!("{}/blocks/{}", API_BASE, id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .context("Failed to send request")?;

        self.handle_response(response).await
    }

    /// Get comments on a page or block.
    pub async fn comments(&self, block_id: &str) -> Result<Vec<Value>> {
        let id = normalize_id(block_id);
        let result = self.get(&format!("/comments?block_id={}", id)).await?;

        let comments = result
            .get("results")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(comments)
    }

    /// Add a comment to a page.
    pub async fn add_comment(&self, page_id: &str, rich_text: Vec<Value>) -> Result<Value> {
        let id = normalize_id(page_id);

        let body = json!({
            "parent": {
                "page_id": id
            },
            "rich_text": rich_text
        });

        self.post("/comments", body).await
    }
}

/// Normalize a Notion ID (remove dashes if present).
fn normalize_id(id: &str) -> String {
    id.replace('-', "")
}

/// Notion API error response.
#[derive(Deserialize)]
struct NotionError {
    code: String,
    message: String,
}

/// Helper to create a simple text block.
pub fn text_block(text: &str) -> Value {
    json!({
        "object": "block",
        "type": "paragraph",
        "paragraph": {
            "rich_text": [{
                "type": "text",
                "text": { "content": text }
            }]
        }
    })
}

/// Helper to create rich text array from plain string.
pub fn rich_text(text: &str) -> Vec<Value> {
    vec![json!({
        "type": "text",
        "text": { "content": text }
    })]
}

#[cfg(test)]
mod tests {
    use super::{normalize_id, rich_text, text_block};
    use serde_json::Value;

    #[test]
    fn normalize_id_strips_dashes() {
        let id = normalize_id("1234-5678-90ab-cdef");
        assert_eq!(id, "1234567890abcdef");
    }

    #[test]
    fn text_block_contains_content() {
        let block = text_block("Hello");
        let content = block
            .get("paragraph")
            .and_then(|p| p.get("rich_text"))
            .and_then(|r| r.get(0))
            .and_then(|t| t.get("text"))
            .and_then(|t| t.get("content"))
            .and_then(Value::as_str);

        assert_eq!(content, Some("Hello"));
    }

    #[test]
    fn rich_text_builds_array() {
        let items = rich_text("Hi");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0]
                .get("text")
                .and_then(|t| t.get("content"))
                .and_then(Value::as_str),
            Some("Hi")
        );
    }
}
