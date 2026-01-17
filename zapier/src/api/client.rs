//! Zapier HTTP API client with connection pooling.
//!
//! Provides webhook triggering and NLA (Natural Language Actions) support.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;

use crate::models::{NlaAction, NlaActionsResponse, NlaResult, WebhookResponse};

const NLA_API_BASE: &str = "https://nla.zapier.com/api/v1";

/// Zapier API client with persistent connection pooling.
pub struct ZapierClient {
    client: Client,
    nla_api_key: Option<String>,
}

impl ZapierClient {
    /// Create a new Zapier client.
    ///
    /// Optionally reads NLA API key from ZAPIER_NLA_API_KEY.
    pub fn new() -> Result<Self> {
        let nla_api_key = std::env::var("ZAPIER_NLA_API_KEY").ok();

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-zapier/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, nla_api_key })
    }

    /// Check if NLA API is available.
    pub fn has_nla(&self) -> bool {
        self.nla_api_key.is_some()
    }

    /// Trigger a webhook.
    pub async fn trigger_webhook(&self, url: &str, payload: &Value) -> Result<WebhookResponse> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .context("Failed to trigger webhook")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            bail!("Webhook trigger failed: {} - {}", status, text);
        }

        // Zapier webhooks can return various responses
        let text = response.text().await.unwrap_or_default();

        // Try to parse as JSON
        if let Ok(parsed) = serde_json::from_str::<WebhookResponse>(&text) {
            return Ok(parsed);
        }

        // Fall back to generic success response
        Ok(WebhookResponse {
            status: "success".to_string(),
            id: None,
            request_id: Some(text),
        })
    }

    /// List available NLA actions.
    pub async fn nla_actions(&self) -> Result<Vec<NlaAction>> {
        let api_key = self.nla_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ZAPIER_NLA_API_KEY not set"))?;

        let url = format!("{}/exposed/", NLA_API_BASE);

        let response = self
            .client
            .get(&url)
            .header("x-api-key", api_key)
            .send()
            .await
            .context("Failed to list NLA actions")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("NLA actions failed: {} - {}", status, text);
        }

        let result: NlaActionsResponse = response.json().await.context("Failed to parse NLA actions")?;
        Ok(result.results)
    }

    /// Execute an NLA action.
    pub async fn nla_execute(&self, action_id: &str, instructions: &str) -> Result<NlaResult> {
        let api_key = self.nla_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ZAPIER_NLA_API_KEY not set"))?;

        let url = format!("{}/exposed/{}/execute/", NLA_API_BASE, action_id);

        let body = serde_json::json!({
            "instructions": instructions
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to execute NLA action")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("NLA execute failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse NLA result")?;
        Ok(result)
    }
}
