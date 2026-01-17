//! FGP service implementation for Composio.
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

use crate::api::ComposioClient;

/// FGP service for Composio operations.
pub struct ComposioService {
    client: Arc<ComposioClient>,
    runtime: Runtime,
}

impl ComposioService {
    /// Create a new ComposioService.
    pub fn new() -> Result<Self> {
        let client = ComposioClient::new()?;
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

    /// Helper to get string array.
    fn get_str_array(params: &HashMap<String, Value>, key: &str) -> Option<Vec<String>> {
        params.get(key).and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
    }

    // ========================================================================
    // Tool methods
    // ========================================================================

    fn search_tools(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_str(&params, "query")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        let limit = Self::get_i32(&params, "limit", 10);
        let apps = Self::get_str_array(&params, "apps");
        let tags = Self::get_str_array(&params, "tags");

        let client = self.client.clone();
        let query = query.to_string();

        let result = self.runtime.block_on(async move {
            client
                .search_tools(&query, apps.as_deref(), tags.as_deref(), limit)
                .await
        })?;

        Ok(json!({
            "tools": result,
            "count": result.len()
        }))
    }

    fn get_tool(&self, params: HashMap<String, Value>) -> Result<Value> {
        let action_name = Self::get_str(&params, "action")
            .or_else(|| Self::get_str(&params, "name"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;

        let client = self.client.clone();
        let action_name = action_name.to_string();

        let result = self.runtime.block_on(async move {
            client.get_tool(&action_name).await
        })?;

        Ok(json!(result))
    }

    fn execute(&self, params: HashMap<String, Value>) -> Result<Value> {
        let action_name = Self::get_str(&params, "action")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;
        let connected_account_id = Self::get_str(&params, "connected_account_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: connected_account_id"))?;

        let input = params.get("input").cloned().unwrap_or(json!({}));

        let client = self.client.clone();
        let action_name = action_name.to_string();
        let connected_account_id = connected_account_id.to_string();

        let result = self.runtime.block_on(async move {
            client
                .execute(&action_name, &connected_account_id, &input)
                .await
        })?;

        Ok(json!(result))
    }

    fn execute_batch(&self, params: HashMap<String, Value>) -> Result<Value> {
        let executions = params
            .get("executions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: executions"))?;

        let batch: Vec<(String, String, Value)> = executions
            .iter()
            .filter_map(|exec| {
                let action = exec.get("action")?.as_str()?.to_string();
                let account = exec.get("connected_account_id")?.as_str()?.to_string();
                let input = exec.get("input").cloned().unwrap_or(json!({}));
                Some((action, account, input))
            })
            .collect();

        if batch.is_empty() {
            return Err(anyhow::anyhow!("No valid executions provided"));
        }

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.execute_batch(&batch).await
        })?;

        Ok(json!({
            "results": result,
            "count": result.len()
        }))
    }

    // ========================================================================
    // Connection methods
    // ========================================================================

    fn list_connections(&self, params: HashMap<String, Value>) -> Result<Value> {
        let app_name = Self::get_str(&params, "app");
        let status = Self::get_str(&params, "status");

        let client = self.client.clone();
        let app_name = app_name.map(|s| s.to_string());
        let status = status.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client
                .list_connections(app_name.as_deref(), status.as_deref())
                .await
        })?;

        Ok(json!({
            "connections": result,
            "count": result.len()
        }))
    }

    fn get_connection(&self, params: HashMap<String, Value>) -> Result<Value> {
        let connection_id = Self::get_str(&params, "connection_id")
            .or_else(|| Self::get_str(&params, "id"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: connection_id"))?;

        let client = self.client.clone();
        let connection_id = connection_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_connection(&connection_id).await
        })?;

        Ok(json!(result))
    }

    fn initiate_connection(&self, params: HashMap<String, Value>) -> Result<Value> {
        let app_name = Self::get_str(&params, "app")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: app"))?;
        let redirect_url = Self::get_str(&params, "redirect_url");
        let entity_id = Self::get_str(&params, "entity_id");

        let client = self.client.clone();
        let app_name = app_name.to_string();
        let redirect_url = redirect_url.map(|s| s.to_string());
        let entity_id = entity_id.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client
                .initiate_connection(&app_name, redirect_url.as_deref(), entity_id.as_deref())
                .await
        })?;

        Ok(json!(result))
    }

    // ========================================================================
    // App methods
    // ========================================================================

    fn list_apps(&self, params: HashMap<String, Value>) -> Result<Value> {
        let category = Self::get_str(&params, "category");
        let limit = Self::get_i32(&params, "limit", 50);

        let client = self.client.clone();
        let category = category.map(|s| s.to_string());

        let result = self.runtime.block_on(async move {
            client.list_apps(category.as_deref(), limit).await
        })?;

        Ok(json!({
            "apps": result,
            "count": result.len()
        }))
    }

    fn get_app(&self, params: HashMap<String, Value>) -> Result<Value> {
        let app_name = Self::get_str(&params, "app")
            .or_else(|| Self::get_str(&params, "name"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: app"))?;

        let client = self.client.clone();
        let app_name = app_name.to_string();

        let result = self.runtime.block_on(async move {
            client.get_app(&app_name).await
        })?;

        Ok(json!(result))
    }
}

impl FgpService for ComposioService {
    fn name(&self) -> &str {
        "composio"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // Tools
            "search_tools" | "composio.search_tools" => self.search_tools(params),
            "get_tool" | "composio.get_tool" => self.get_tool(params),
            "execute" | "composio.execute" => self.execute(params),
            "execute_batch" | "composio.execute_batch" => self.execute_batch(params),

            // Connections
            "list_connections" | "composio.list_connections" => self.list_connections(params),
            "get_connection" | "composio.get_connection" => self.get_connection(params),
            "initiate_connection" | "composio.initiate_connection" => self.initiate_connection(params),

            // Apps
            "list_apps" | "composio.list_apps" => self.list_apps(params),
            "get_app" | "composio.get_app" => self.get_app(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // Tools
            MethodInfo::new("composio.search_tools", "Search available tools/actions by query")
                .schema(
                    SchemaBuilder::object()
                        .property("query", SchemaBuilder::string().description("Search query (e.g., 'send email')"))
                        .property("apps", SchemaBuilder::array().description("Filter by app names"))
                        .property("tags", SchemaBuilder::array().description("Filter by tags"))
                        .property("limit", SchemaBuilder::integer().default_value(json!(10)))
                        .required(&["query"])
                        .build(),
                )
                .example("Search email tools", json!({"query": "send email", "limit": 5})),

            MethodInfo::new("composio.get_tool", "Get tool schema and metadata")
                .schema(
                    SchemaBuilder::object()
                        .property("action", SchemaBuilder::string().description("Tool/action name"))
                        .required(&["action"])
                        .build(),
                )
                .example("Get Gmail send tool", json!({"action": "GMAIL_SEND_EMAIL"})),

            MethodInfo::new("composio.execute", "Execute a single tool")
                .schema(
                    SchemaBuilder::object()
                        .property("action", SchemaBuilder::string().description("Tool name"))
                        .property("connected_account_id", SchemaBuilder::string().description("OAuth connection ID"))
                        .property("input", SchemaBuilder::object().description("Tool input parameters"))
                        .required(&["action", "connected_account_id"])
                        .build(),
                )
                .example(
                    "Send Slack message",
                    json!({
                        "action": "SLACK_SEND_MESSAGE",
                        "connected_account_id": "conn_xxx",
                        "input": {"channel": "#general", "text": "Hello!"}
                    }),
                ),

            MethodInfo::new("composio.execute_batch", "Execute multiple tools in parallel")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "executions",
                            SchemaBuilder::array().description("Array of {action, connected_account_id, input}"),
                        )
                        .required(&["executions"])
                        .build(),
                ),

            // Connections
            MethodInfo::new("composio.list_connections", "List OAuth connected accounts")
                .schema(
                    SchemaBuilder::object()
                        .property("app", SchemaBuilder::string().description("Filter by app name"))
                        .property("status", SchemaBuilder::string().description("Filter by status"))
                        .build(),
                )
                .example("List Slack connections", json!({"app": "slack"})),

            MethodInfo::new("composio.get_connection", "Get connection details")
                .schema(
                    SchemaBuilder::object()
                        .property("connection_id", SchemaBuilder::string())
                        .required(&["connection_id"])
                        .build(),
                ),

            MethodInfo::new("composio.initiate_connection", "Start OAuth flow for an app")
                .schema(
                    SchemaBuilder::object()
                        .property("app", SchemaBuilder::string().description("App name (e.g., 'gmail', 'slack')"))
                        .property("redirect_url", SchemaBuilder::string().description("OAuth callback URL"))
                        .property("entity_id", SchemaBuilder::string().description("Entity/user ID"))
                        .required(&["app"])
                        .build(),
                )
                .example("Connect to Gmail", json!({"app": "gmail"})),

            // Apps
            MethodInfo::new("composio.list_apps", "List available apps/integrations")
                .schema(
                    SchemaBuilder::object()
                        .property("category", SchemaBuilder::string().description("Filter by category"))
                        .property("limit", SchemaBuilder::integer().default_value(json!(50)))
                        .build(),
                ),

            MethodInfo::new("composio.get_app", "Get app details")
                .schema(
                    SchemaBuilder::object()
                        .property("app", SchemaBuilder::string().description("App name"))
                        .required(&["app"])
                        .build(),
                ),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("ComposioService starting, verifying API connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Composio API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Composio API returned unexpected response");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Composio API: {}", e);
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
                    "composio_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "composio_api".into(),
                    HealthStatus::unhealthy("Unexpected response"),
                );
            }
            Err(e) => {
                checks.insert("composio_api".into(), HealthStatus::unhealthy(e.to_string()));
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
            std::env::set_var("COMPOSIO_API_KEY", "test-key");
        });
    }

    fn test_service() -> ComposioService {
        ensure_env();
        ComposioService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("query".to_string(), Value::String("send email".to_string()));
        params.insert("limit".to_string(), Value::from(25));
        params.insert(
            "apps".to_string(),
            Value::Array(vec![
                Value::String("slack".to_string()),
                Value::Bool(true),
                Value::String("gmail".to_string()),
            ]),
        );

        assert_eq!(ComposioService::get_str(&params, "query"), Some("send email"));
        assert_eq!(ComposioService::get_str(&params, "missing"), None);
        assert_eq!(ComposioService::get_i32(&params, "limit", 10), 25);
        assert_eq!(ComposioService::get_i32(&params, "missing", 10), 10);
        assert_eq!(
            ComposioService::get_str_array(&params, "apps"),
            Some(vec!["slack".to_string(), "gmail".to_string()])
        );
        assert_eq!(ComposioService::get_str_array(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_required_fields() {
        let service = test_service();
        let methods = service.method_list();

        let execute = methods
            .iter()
            .find(|m| m.name == "composio.execute")
            .expect("execute");
        let execute_schema = execute.schema.as_ref().expect("schema");
        let execute_required = execute_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(execute_required.iter().any(|v| v == "action"));
        assert!(execute_required.iter().any(|v| v == "connected_account_id"));

        let search = methods
            .iter()
            .find(|m| m.name == "composio.search_tools")
            .expect("search_tools");
        let search_schema = search.schema.as_ref().expect("schema");
        let search_required = search_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(search_required.iter().any(|v| v == "query"));
    }

    #[test]
    fn test_method_list_defaults() {
        let service = test_service();
        let methods = service.method_list();

        let search = methods
            .iter()
            .find(|m| m.name == "composio.search_tools")
            .expect("search_tools");
        let search_schema = search.schema.as_ref().expect("schema");
        let search_props = search_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(
            search_props
                .get("limit")
                .and_then(Value::as_object)
                .and_then(|p| p.get("default")),
            Some(&json!(10))
        );

        let list_apps = methods
            .iter()
            .find(|m| m.name == "composio.list_apps")
            .expect("list_apps");
        let list_schema = list_apps.schema.as_ref().expect("schema");
        let list_props = list_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        assert_eq!(
            list_props
                .get("limit")
                .and_then(Value::as_object)
                .and_then(|p| p.get("default")),
            Some(&json!(50))
        );
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("composio.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
