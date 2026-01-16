//! FGP service implementation for Linear.

use anyhow::Result;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::client::LinearClient;

/// FGP service for Linear operations.
pub struct LinearService {
    client: Arc<LinearClient>,
    runtime: Runtime,
}

impl LinearService {
    /// Create a new LinearService with the given API key.
    pub fn new(api_key: String) -> Result<Self> {
        let client = LinearClient::new(api_key)?;
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

    fn teams(&self) -> Result<Value> {
        let client = self.client.clone();
        let teams = self.runtime.block_on(async move { client.teams().await })?;
        Ok(serde_json::json!({ "teams": teams, "count": teams.len() }))
    }

    fn issues(&self, params: HashMap<String, Value>) -> Result<Value> {
        let team = Self::get_str(&params, "team");
        let state = Self::get_str(&params, "state");
        let assignee = Self::get_str(&params, "assignee");
        let limit = Self::get_i32(&params, "limit", 25);

        let client = self.client.clone();
        let issues = self.runtime.block_on(async move {
            client.issues(team, state, assignee, limit).await
        })?;

        Ok(serde_json::json!({ "issues": issues, "count": issues.len() }))
    }

    fn issue(&self, params: HashMap<String, Value>) -> Result<Value> {
        let id = Self::get_str(&params, "id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: id"))?
            .to_string();

        let client = self.client.clone();
        self.runtime
            .block_on(async move { client.issue(&id).await })
    }

    fn create_issue(&self, params: HashMap<String, Value>) -> Result<Value> {
        let team_id = Self::get_str(&params, "team_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: team_id"))?
            .to_string();
        let title = Self::get_str(&params, "title")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?
            .to_string();
        let description = Self::get_str(&params, "description").map(|s| s.to_string());
        let priority = params.get("priority").and_then(|v| v.as_i64()).map(|v| v as i32);
        let assignee_id = Self::get_str(&params, "assignee_id").map(|s| s.to_string());

        let client = self.client.clone();
        self.runtime.block_on(async move {
            client
                .create_issue(
                    &team_id,
                    &title,
                    description.as_deref(),
                    priority,
                    assignee_id.as_deref(),
                )
                .await
        })
    }

    fn update_issue(&self, params: HashMap<String, Value>) -> Result<Value> {
        let id = Self::get_str(&params, "id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: id"))?
            .to_string();
        let title = Self::get_str(&params, "title").map(|s| s.to_string());
        let description = Self::get_str(&params, "description").map(|s| s.to_string());
        let state_id = Self::get_str(&params, "state_id").map(|s| s.to_string());
        let priority = params.get("priority").and_then(|v| v.as_i64()).map(|v| v as i32);
        let assignee_id = Self::get_str(&params, "assignee_id").map(|s| s.to_string());

        let client = self.client.clone();
        self.runtime.block_on(async move {
            client
                .update_issue(
                    &id,
                    title.as_deref(),
                    description.as_deref(),
                    state_id.as_deref(),
                    priority,
                    assignee_id.as_deref(),
                )
                .await
        })
    }

    fn comments(&self, params: HashMap<String, Value>) -> Result<Value> {
        let issue_id = Self::get_str(&params, "issue_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: issue_id"))?
            .to_string();

        let client = self.client.clone();
        let comments = self
            .runtime
            .block_on(async move { client.comments(&issue_id).await })?;

        Ok(serde_json::json!({ "comments": comments, "count": comments.len() }))
    }

    fn add_comment(&self, params: HashMap<String, Value>) -> Result<Value> {
        let issue_id = Self::get_str(&params, "issue_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: issue_id"))?
            .to_string();
        let body = Self::get_str(&params, "body")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: body"))?
            .to_string();

        let client = self.client.clone();
        self.runtime
            .block_on(async move { client.add_comment(&issue_id, &body).await })
    }

    fn projects(&self, params: HashMap<String, Value>) -> Result<Value> {
        let team = Self::get_str(&params, "team");
        let limit = Self::get_i32(&params, "limit", 25);

        let client = self.client.clone();
        let projects = self
            .runtime
            .block_on(async move { client.projects(team, limit).await })?;

        Ok(serde_json::json!({ "projects": projects, "count": projects.len() }))
    }

    fn cycles(&self, params: HashMap<String, Value>) -> Result<Value> {
        let team = Self::get_str(&params, "team");
        let limit = Self::get_i32(&params, "limit", 10);

        let client = self.client.clone();
        let cycles = self
            .runtime
            .block_on(async move { client.cycles(team, limit).await })?;

        Ok(serde_json::json!({ "cycles": cycles, "count": cycles.len() }))
    }

    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_str(&params, "query")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?
            .to_string();
        let limit = Self::get_i32(&params, "limit", 25);

        let client = self.client.clone();
        let results = self
            .runtime
            .block_on(async move { client.search(&query, limit).await })?;

        Ok(serde_json::json!({ "issues": results, "count": results.len() }))
    }

    fn states(&self, params: HashMap<String, Value>) -> Result<Value> {
        let team_key = Self::get_str(&params, "team_key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: team_key"))?
            .to_string();

        let client = self.client.clone();
        let states = self
            .runtime
            .block_on(async move { client.states(&team_key).await })?;

        Ok(serde_json::json!({ "states": states, "count": states.len() }))
    }
}

impl FgpService for LinearService {
    fn name(&self) -> &str {
        "linear"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.health(),
            "me" | "linear.me" => self.me(),
            "teams" | "linear.teams" => self.teams(),
            "issues" | "linear.issues" => self.issues(params),
            "issue" | "linear.issue" => self.issue(params),
            "create_issue" | "linear.create_issue" => self.create_issue(params),
            "update_issue" | "linear.update_issue" => self.update_issue(params),
            "comments" | "linear.comments" => self.comments(params),
            "add_comment" | "linear.add_comment" => self.add_comment(params),
            "projects" | "linear.projects" => self.projects(params),
            "cycles" | "linear.cycles" => self.cycles(params),
            "search" | "linear.search" => self.search(params),
            "states" | "linear.states" => self.states(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("linear.me", "Get current user info"),
            MethodInfo::new("linear.teams", "List all teams"),
            MethodInfo::new("linear.issues", "List issues with optional filters")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "team": { "type": "string", "description": "Team key filter" },
                        "state": { "type": "string", "description": "State name filter" },
                        "assignee": { "type": "string", "description": "Assignee email filter" },
                        "limit": { "type": "integer", "default": 25 }
                    }
                })),
            MethodInfo::new("linear.issue", "Get a single issue")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Issue ID or identifier" }
                    },
                    "required": ["id"]
                })),
            MethodInfo::new("linear.create_issue", "Create a new issue")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "team_id": { "type": "string", "description": "Team ID" },
                        "title": { "type": "string", "description": "Issue title" },
                        "description": { "type": "string", "description": "Issue description" },
                        "priority": { "type": "integer", "description": "Priority 0-4 (0=none, 1=urgent, 4=low)" },
                        "assignee_id": { "type": "string", "description": "Assignee user ID" }
                    },
                    "required": ["team_id", "title"]
                })),
            MethodInfo::new("linear.update_issue", "Update an issue")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Issue ID" },
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "state_id": { "type": "string" },
                        "priority": { "type": "integer" },
                        "assignee_id": { "type": "string" }
                    },
                    "required": ["id"]
                })),
            MethodInfo::new("linear.comments", "Get comments for an issue")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "issue_id": { "type": "string", "description": "Issue ID" }
                    },
                    "required": ["issue_id"]
                })),
            MethodInfo::new("linear.add_comment", "Add a comment to an issue")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "issue_id": { "type": "string", "description": "Issue ID" },
                        "body": { "type": "string", "description": "Comment body (markdown)" }
                    },
                    "required": ["issue_id", "body"]
                })),
            MethodInfo::new("linear.projects", "List projects")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "team": { "type": "string", "description": "Team key filter" },
                        "limit": { "type": "integer", "default": 25 }
                    }
                })),
            MethodInfo::new("linear.cycles", "List cycles/sprints")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "team": { "type": "string", "description": "Team key filter" },
                        "limit": { "type": "integer", "default": 10 }
                    }
                })),
            MethodInfo::new("linear.search", "Search issues")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "limit": { "type": "integer", "default": 25 }
                    },
                    "required": ["query"]
                })),
            MethodInfo::new("linear.states", "Get workflow states for a team")
                .schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "team_key": { "type": "string", "description": "Team key" }
                    },
                    "required": ["team_key"]
                })),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("LinearService starting, verifying API connection...");

        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Linear API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Linear API returned unsuccessful response");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Linear API: {}", e);
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
                    "linear_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "linear_api".into(),
                    HealthStatus::unhealthy("API returned error"),
                );
            }
            Err(e) => {
                checks.insert("linear_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}
