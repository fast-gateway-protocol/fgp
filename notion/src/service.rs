//! FGP service implementation for Notion.

use anyhow::Result;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::client::NotionClient;

/// FGP service for Notion operations.
pub struct NotionService {
    client: Arc<NotionClient>,
    runtime: Runtime,
}

impl NotionService {
    /// Create a new NotionService with the given API key.
    pub fn new(api_key: String) -> Result<Self> {
        let client = NotionClient::new(api_key)?;
        let runtime = Runtime::new()?;

        Ok(Self {
            client: Arc::new(client),
            runtime,
        })
    }

    fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    fn get_i32(params: &HashMap<String, Value>, key: &str, default: i32) -> i32 {
        params
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default)
    }

    fn get_bool(params: &HashMap<String, Value>, key: &str, default: bool) -> bool {
        params
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    fn health(&self) -> Result<Value> {
        let client = self.client.clone();
        let ok = self.runtime.block_on(async move { client.ping().await })?;

        Ok(serde_json::json!({
            "status": if ok { "healthy" } else { "unhealthy" },
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    fn me(&self) -> Result<Value> {
        let client = self.client.clone();
        self.runtime.block_on(async move { client.me().await })
    }

    fn users(&self) -> Result<Value> {
        let client = self.client.clone();
        let users = self.runtime.block_on(async move { client.users().await })?;
        Ok(serde_json::json!({ "users": users, "count": users.len() }))
    }

    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_str(&params, "query");
        let filter = Self::get_str(&params, "filter");
        let limit = Self::get_i32(&params, "limit", 25);

        let client = self.client.clone();
        let results = self.runtime.block_on(async move {
            client.search(query, filter, limit).await
        })?;

        Ok(serde_json::json!({ "results": results, "count": results.len() }))
    }

    fn page(&self, params: HashMap<String, Value>) -> Result<Value> {
        let page_id = Self::get_str(&params, "page_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: page_id"))?
            .to_string();

        let client = self.client.clone();
        self.runtime
            .block_on(async move { client.page(&page_id).await })
    }

    fn database(&self, params: HashMap<String, Value>) -> Result<Value> {
        let database_id = Self::get_str(&params, "database_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: database_id"))?
            .to_string();

        let client = self.client.clone();
        self.runtime
            .block_on(async move { client.database(&database_id).await })
    }

    fn query_database(&self, params: HashMap<String, Value>) -> Result<Value> {
        let database_id = Self::get_str(&params, "database_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: database_id"))?
            .to_string();
        let filter = params.get("filter").cloned();
        let sorts = params.get("sorts").cloned();
        let limit = Self::get_i32(&params, "limit", 25);

        let client = self.client.clone();
        let results = self.runtime.block_on(async move {
            client.query_database(&database_id, filter, sorts, limit).await
        })?;

        Ok(serde_json::json!({ "results": results, "count": results.len() }))
    }

    fn blocks(&self, params: HashMap<String, Value>) -> Result<Value> {
        let block_id = Self::get_str(&params, "block_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: block_id"))?
            .to_string();
        let recursive = Self::get_bool(&params, "recursive", false);

        let client = self.client.clone();
        let blocks = self.runtime.block_on(async move {
            client.blocks(&block_id, recursive).await
        })?;

        Ok(serde_json::json!({ "blocks": blocks, "count": blocks.len() }))
    }

    fn create_page(&self, params: HashMap<String, Value>) -> Result<Value> {
        let database_id = Self::get_str(&params, "database_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: database_id"))?
            .to_string();
        let properties = params
            .get("properties")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: properties"))?;
        let children = params
            .get("children")
            .and_then(|c| c.as_array())
            .cloned();

        let client = self.client.clone();
        self.runtime.block_on(async move {
            client.create_page(&database_id, properties, children).await
        })
    }

    fn update_page(&self, params: HashMap<String, Value>) -> Result<Value> {
        let page_id = Self::get_str(&params, "page_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: page_id"))?
            .to_string();
        let properties = params
            .get("properties")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: properties"))?;

        let client = self.client.clone();
        self.runtime.block_on(async move {
            client.update_page(&page_id, properties).await
        })
    }

    fn append_blocks(&self, params: HashMap<String, Value>) -> Result<Value> {
        let block_id = Self::get_str(&params, "block_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: block_id"))?
            .to_string();
        let children = params
            .get("children")
            .and_then(|c| c.as_array())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: children"))?;

        let client = self.client.clone();
        self.runtime.block_on(async move {
            client.append_blocks(&block_id, children).await
        })
    }

    fn comments(&self, params: HashMap<String, Value>) -> Result<Value> {
        let block_id = Self::get_str(&params, "block_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: block_id"))?
            .to_string();

        let client = self.client.clone();
        let comments = self.runtime.block_on(async move {
            client.comments(&block_id).await
        })?;

        Ok(serde_json::json!({ "comments": comments, "count": comments.len() }))
    }

    fn add_comment(&self, params: HashMap<String, Value>) -> Result<Value> {
        let page_id = Self::get_str(&params, "page_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: page_id"))?
            .to_string();
        let text = Self::get_str(&params, "text")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?
            .to_string();

        let client = self.client.clone();
        self.runtime.block_on(async move {
            let rich_text = crate::client::rich_text(&text);
            client.add_comment(&page_id, rich_text).await
        })
    }
}

impl FgpService for NotionService {
    fn name(&self) -> &str {
        "notion"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.health(),
            "me" | "notion.me" => self.me(),
            "users" | "notion.users" => self.users(),
            "search" | "notion.search" => self.search(params),
            "page" | "notion.page" => self.page(params),
            "database" | "notion.database" => self.database(params),
            "query_database" | "notion.query_database" => self.query_database(params),
            "blocks" | "notion.blocks" => self.blocks(params),
            "create_page" | "notion.create_page" => self.create_page(params),
            "update_page" | "notion.update_page" => self.update_page(params),
            "append_blocks" | "notion.append_blocks" => self.append_blocks(params),
            "comments" | "notion.comments" => self.comments(params),
            "add_comment" | "notion.add_comment" => self.add_comment(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("notion.me", "Get current bot/integration info"),
            MethodInfo::new("notion.users", "List workspace users"),
            MethodInfo::new("notion.search", "Search pages and databases")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "filter": { "type": "string", "enum": ["page", "database"], "description": "Object type filter" },
                        "limit": { "type": "integer", "default": 25 }
                    }
                })),
            MethodInfo::new("notion.page", "Get a page by ID")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "page_id": { "type": "string", "description": "Page ID (with or without dashes)" }
                    },
                    "required": ["page_id"]
                })),
            MethodInfo::new("notion.database", "Get a database schema by ID")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "database_id": { "type": "string", "description": "Database ID" }
                    },
                    "required": ["database_id"]
                })),
            MethodInfo::new("notion.query_database", "Query database rows")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "database_id": { "type": "string", "description": "Database ID" },
                        "filter": { "type": "object", "description": "Notion filter object" },
                        "sorts": { "type": "array", "description": "Sort configuration" },
                        "limit": { "type": "integer", "default": 25 }
                    },
                    "required": ["database_id"]
                })),
            MethodInfo::new("notion.blocks", "Get page/block children")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "block_id": { "type": "string", "description": "Block or page ID" },
                        "recursive": { "type": "boolean", "default": false, "description": "Fetch nested blocks" }
                    },
                    "required": ["block_id"]
                })),
            MethodInfo::new("notion.create_page", "Create a page in a database")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "database_id": { "type": "string", "description": "Parent database ID" },
                        "properties": { "type": "object", "description": "Page properties" },
                        "children": { "type": "array", "description": "Initial content blocks" }
                    },
                    "required": ["database_id", "properties"]
                })),
            MethodInfo::new("notion.update_page", "Update page properties")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "page_id": { "type": "string", "description": "Page ID" },
                        "properties": { "type": "object", "description": "Properties to update" }
                    },
                    "required": ["page_id", "properties"]
                })),
            MethodInfo::new("notion.append_blocks", "Append blocks to a page")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "block_id": { "type": "string", "description": "Page or block ID" },
                        "children": { "type": "array", "description": "Block objects to append" }
                    },
                    "required": ["block_id", "children"]
                })),
            MethodInfo::new("notion.comments", "Get comments on a page/block")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "block_id": { "type": "string", "description": "Page or block ID" }
                    },
                    "required": ["block_id"]
                })),
            MethodInfo::new("notion.add_comment", "Add a comment to a page")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "page_id": { "type": "string", "description": "Page ID" },
                        "text": { "type": "string", "description": "Comment text" }
                    },
                    "required": ["page_id", "text"]
                })),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("NotionService starting, verifying API connection...");

        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Notion API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Notion API returned unsuccessful response");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Notion API: {}", e);
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
                    "notion_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "notion_api".into(),
                    HealthStatus::unhealthy("API returned error"),
                );
            }
            Err(e) => {
                checks.insert("notion_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}
