//! Composio HTTP API client with connection pooling.
//!
//! Provides access to 500+ SaaS integrations through Composio's API.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;

use crate::models::{App, Connection, ConnectionInitiation, ExecutionResult, Tool, ToolsResponse};

const API_BASE: &str = "https://backend.composio.dev/api/v3";

/// Composio API client with persistent connection pooling.
pub struct ComposioClient {
    client: Client,
    api_key: String,
}

impl ComposioClient {
    /// Create a new Composio client.
    ///
    /// Reads API key from COMPOSIO_API_KEY environment variable.
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("COMPOSIO_API_KEY")
            .context("COMPOSIO_API_KEY environment variable not set")?;

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("fgp-composio/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, api_key })
    }

    /// Check if client can connect to Composio API.
    pub async fn ping(&self) -> Result<bool> {
        let url = format!("{}/apps", API_BASE);
        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .query(&[("limit", "1")])
            .send()
            .await
            .context("Failed to ping Composio")?;

        Ok(response.status().is_success())
    }

    /// Search for tools by query.
    pub async fn search_tools(
        &self,
        query: &str,
        apps: Option<&[String]>,
        tags: Option<&[String]>,
        limit: i32,
    ) -> Result<Vec<Tool>> {
        let url = format!("{}/actions", API_BASE);

        let mut request = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .query(&[("useCase", query), ("limit", &limit.to_string())]);

        if let Some(app_list) = apps {
            for app in app_list {
                request = request.query(&[("apps", app)]);
            }
        }

        if let Some(tag_list) = tags {
            for tag in tag_list {
                request = request.query(&[("tags", tag)]);
            }
        }

        let response = request.send().await.context("Failed to search tools")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Search tools failed: {} - {}", status, text);
        }

        let result: ToolsResponse = response.json().await.context("Failed to parse tools")?;
        Ok(result.items)
    }

    /// Get a specific tool by name/slug.
    pub async fn get_tool(&self, action_name: &str) -> Result<Tool> {
        let url = format!("{}/actions/{}", API_BASE, action_name);

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Failed to get tool")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Get tool failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse tool")?;
        Ok(result)
    }

    /// Execute a tool.
    pub async fn execute(
        &self,
        action_name: &str,
        connected_account_id: &str,
        input: &Value,
    ) -> Result<ExecutionResult> {
        let url = format!("{}/actions/{}/execute", API_BASE, action_name);

        let body = serde_json::json!({
            "connectedAccountId": connected_account_id,
            "input": input
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to execute tool")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Execute failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse execution result")?;
        Ok(result)
    }

    /// Execute multiple tools in batch.
    pub async fn execute_batch(
        &self,
        executions: &[(String, String, Value)],
    ) -> Result<Vec<ExecutionResult>> {
        // Execute in parallel using tokio tasks
        let mut handles = Vec::new();

        for (action_name, connected_account_id, input) in executions {
            let client = self.client.clone();
            let api_key = self.api_key.clone();
            let action = action_name.clone();
            let account = connected_account_id.clone();
            let params = input.clone();

            handles.push(tokio::spawn(async move {
                let url = format!("{}/actions/{}/execute", API_BASE, action);
                let body = serde_json::json!({
                    "connectedAccountId": account,
                    "input": params
                });

                let response = client
                    .post(&url)
                    .header("x-api-key", &api_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.status().is_success() => {
                        resp.json::<ExecutionResult>().await.unwrap_or(ExecutionResult {
                            data: Value::Null,
                            error: Some("Failed to parse response".to_string()),
                            successful: false,
                        })
                    }
                    Ok(resp) => ExecutionResult {
                        data: Value::Null,
                        error: Some(format!("HTTP {}", resp.status())),
                        successful: false,
                    },
                    Err(e) => ExecutionResult {
                        data: Value::Null,
                        error: Some(e.to_string()),
                        successful: false,
                    },
                }
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(ExecutionResult {
                    data: Value::Null,
                    error: Some(e.to_string()),
                    successful: false,
                }),
            }
        }

        Ok(results)
    }

    /// List connected accounts.
    pub async fn list_connections(
        &self,
        app_name: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<Connection>> {
        let url = format!("{}/connectedAccounts", API_BASE);

        let mut request = self.client.get(&url).header("x-api-key", &self.api_key);

        if let Some(app) = app_name {
            request = request.query(&[("appNames", app)]);
        }

        if let Some(s) = status {
            request = request.query(&[("status", s)]);
        }

        let response = request.send().await.context("Failed to list connections")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("List connections failed: {} - {}", status, text);
        }

        #[derive(serde::Deserialize)]
        struct ConnectionsResponse {
            #[serde(default)]
            items: Vec<Connection>,
        }

        let result: ConnectionsResponse = response.json().await.context("Failed to parse connections")?;
        Ok(result.items)
    }

    /// Get a specific connection.
    pub async fn get_connection(&self, connection_id: &str) -> Result<Connection> {
        let url = format!("{}/connectedAccounts/{}", API_BASE, connection_id);

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Failed to get connection")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Get connection failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse connection")?;
        Ok(result)
    }

    /// Initiate a new OAuth connection.
    pub async fn initiate_connection(
        &self,
        app_name: &str,
        redirect_url: Option<&str>,
        entity_id: Option<&str>,
    ) -> Result<ConnectionInitiation> {
        let url = format!("{}/connectedAccounts", API_BASE);

        let mut body = serde_json::json!({
            "integrationId": app_name
        });

        if let Some(redirect) = redirect_url {
            body["redirectUri"] = Value::String(redirect.to_string());
        }

        if let Some(entity) = entity_id {
            body["entityId"] = Value::String(entity.to_string());
        }

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to initiate connection")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Initiate connection failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse connection initiation")?;
        Ok(result)
    }

    /// List available apps/integrations.
    pub async fn list_apps(&self, category: Option<&str>, limit: i32) -> Result<Vec<App>> {
        let url = format!("{}/apps", API_BASE);

        let mut request = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .query(&[("limit", &limit.to_string())]);

        if let Some(cat) = category {
            request = request.query(&[("category", cat)]);
        }

        let response = request.send().await.context("Failed to list apps")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("List apps failed: {} - {}", status, text);
        }

        #[derive(serde::Deserialize)]
        struct AppsResponse {
            #[serde(default)]
            items: Vec<App>,
        }

        let result: AppsResponse = response.json().await.context("Failed to parse apps")?;
        Ok(result.items)
    }

    /// Get app details.
    pub async fn get_app(&self, app_name: &str) -> Result<App> {
        let url = format!("{}/apps/{}", API_BASE, app_name);

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Failed to get app")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Get app failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse app")?;
        Ok(result)
    }
}
