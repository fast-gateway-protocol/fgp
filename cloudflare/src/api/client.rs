//! Cloudflare HTTP API client with connection pooling.
//!
//! Uses CLOUDFLARE_API_TOKEN env var for authentication.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;

use crate::models::{ApiResponse, DnsRecord, KvKey, KvNamespace, Worker, WorkerRoute, Zone};

const API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare API client with persistent connection pooling.
pub struct CloudflareClient {
    client: Client,
    api_token: Option<String>,
    account_id: Option<String>,
}

impl CloudflareClient {
    /// Create a new Cloudflare client.
    pub fn new() -> Result<Self> {
        let api_token = std::env::var("CLOUDFLARE_API_TOKEN").ok();
        let account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").ok();

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("fgp-cloudflare/1.0.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_token,
            account_id,
        })
    }

    /// Check if API token is available.
    pub fn has_token(&self) -> bool {
        self.api_token.is_some()
    }

    /// Get API token.
    fn get_token(&self) -> Result<&str> {
        self.api_token
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("CLOUDFLARE_API_TOKEN not set"))
    }

    /// Get account ID.
    fn get_account_id(&self) -> Result<&str> {
        self.account_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("CLOUDFLARE_ACCOUNT_ID not set"))
    }

    // ========================================================================
    // Zones
    // ========================================================================

    /// List all zones.
    pub async fn list_zones(&self) -> Result<Vec<Zone>> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/zones", API_BASE))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list zones")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<Vec<Zone>> = response.json().await.context("Failed to parse zones")?;

        if !result.success {
            bail!("API error: {:?}", result.errors);
        }

        Ok(result.result.unwrap_or_default())
    }

    /// Get zone by ID.
    pub async fn get_zone(&self, zone_id: &str) -> Result<Zone> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/zones/{}", API_BASE, zone_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to get zone")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<Zone> = response.json().await.context("Failed to parse zone")?;

        result.result.ok_or_else(|| anyhow::anyhow!("Zone not found"))
    }

    // ========================================================================
    // DNS Records
    // ========================================================================

    /// List DNS records for a zone.
    pub async fn list_dns_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/zones/{}/dns_records", API_BASE, zone_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list DNS records")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<Vec<DnsRecord>> = response.json().await.context("Failed to parse DNS records")?;

        Ok(result.result.unwrap_or_default())
    }

    /// Create a DNS record.
    pub async fn create_dns_record(
        &self,
        zone_id: &str,
        record_type: &str,
        name: &str,
        content: &str,
        ttl: Option<i64>,
        proxied: Option<bool>,
        priority: Option<i64>,
    ) -> Result<DnsRecord> {
        let token = self.get_token()?;

        let mut body = serde_json::json!({
            "type": record_type,
            "name": name,
            "content": content
        });

        if let Some(t) = ttl {
            body["ttl"] = serde_json::json!(t);
        }
        if let Some(p) = proxied {
            body["proxied"] = serde_json::json!(p);
        }
        if let Some(pr) = priority {
            body["priority"] = serde_json::json!(pr);
        }

        let response = self
            .client
            .post(format!("{}/zones/{}/dns_records", API_BASE, zone_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to create DNS record")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<DnsRecord> = response.json().await.context("Failed to parse DNS record")?;

        result.result.ok_or_else(|| anyhow::anyhow!("Failed to create record"))
    }

    /// Update a DNS record.
    pub async fn update_dns_record(
        &self,
        zone_id: &str,
        record_id: &str,
        record_type: &str,
        name: &str,
        content: &str,
        ttl: Option<i64>,
        proxied: Option<bool>,
    ) -> Result<DnsRecord> {
        let token = self.get_token()?;

        let mut body = serde_json::json!({
            "type": record_type,
            "name": name,
            "content": content
        });

        if let Some(t) = ttl {
            body["ttl"] = serde_json::json!(t);
        }
        if let Some(p) = proxied {
            body["proxied"] = serde_json::json!(p);
        }

        let response = self
            .client
            .put(format!("{}/zones/{}/dns_records/{}", API_BASE, zone_id, record_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to update DNS record")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<DnsRecord> = response.json().await.context("Failed to parse DNS record")?;

        result.result.ok_or_else(|| anyhow::anyhow!("Failed to update record"))
    }

    /// Delete a DNS record.
    pub async fn delete_dns_record(&self, zone_id: &str, record_id: &str) -> Result<()> {
        let token = self.get_token()?;

        let response = self
            .client
            .delete(format!("{}/zones/{}/dns_records/{}", API_BASE, zone_id, record_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to delete DNS record")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        Ok(())
    }

    // ========================================================================
    // KV
    // ========================================================================

    /// List KV namespaces.
    pub async fn list_kv_namespaces(&self) -> Result<Vec<KvNamespace>> {
        let token = self.get_token()?;
        let account_id = self.get_account_id()?;

        let response = self
            .client
            .get(format!("{}/accounts/{}/storage/kv/namespaces", API_BASE, account_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list KV namespaces")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<Vec<KvNamespace>> = response.json().await.context("Failed to parse namespaces")?;

        Ok(result.result.unwrap_or_default())
    }

    /// List keys in a KV namespace.
    pub async fn list_kv_keys(&self, namespace_id: &str) -> Result<Vec<KvKey>> {
        let token = self.get_token()?;
        let account_id = self.get_account_id()?;

        let response = self
            .client
            .get(format!(
                "{}/accounts/{}/storage/kv/namespaces/{}/keys",
                API_BASE, account_id, namespace_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list KV keys")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<Vec<KvKey>> = response.json().await.context("Failed to parse keys")?;

        Ok(result.result.unwrap_or_default())
    }

    /// Read a KV value.
    pub async fn read_kv(&self, namespace_id: &str, key: &str) -> Result<String> {
        let token = self.get_token()?;
        let account_id = self.get_account_id()?;

        let response = self
            .client
            .get(format!(
                "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
                API_BASE, account_id, namespace_id, key
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to read KV value")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        response.text().await.context("Failed to read KV value")
    }

    /// Write a KV value.
    pub async fn write_kv(&self, namespace_id: &str, key: &str, value: &str) -> Result<()> {
        let token = self.get_token()?;
        let account_id = self.get_account_id()?;

        let response = self
            .client
            .put(format!(
                "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
                API_BASE, account_id, namespace_id, key
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "text/plain")
            .body(value.to_string())
            .send()
            .await
            .context("Failed to write KV value")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        Ok(())
    }

    /// Delete a KV key.
    pub async fn delete_kv(&self, namespace_id: &str, key: &str) -> Result<()> {
        let token = self.get_token()?;
        let account_id = self.get_account_id()?;

        let response = self
            .client
            .delete(format!(
                "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
                API_BASE, account_id, namespace_id, key
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to delete KV key")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        Ok(())
    }

    // ========================================================================
    // Workers
    // ========================================================================

    /// List Workers.
    pub async fn list_workers(&self) -> Result<Vec<Worker>> {
        let token = self.get_token()?;
        let account_id = self.get_account_id()?;

        let response = self
            .client
            .get(format!("{}/accounts/{}/workers/scripts", API_BASE, account_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list Workers")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<Vec<Worker>> = response.json().await.context("Failed to parse Workers")?;

        Ok(result.result.unwrap_or_default())
    }

    /// List Worker routes for a zone.
    pub async fn list_worker_routes(&self, zone_id: &str) -> Result<Vec<WorkerRoute>> {
        let token = self.get_token()?;

        let response = self
            .client
            .get(format!("{}/zones/{}/workers/routes", API_BASE, zone_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list Worker routes")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        let result: ApiResponse<Vec<WorkerRoute>> = response.json().await.context("Failed to parse routes")?;

        Ok(result.result.unwrap_or_default())
    }

    /// Purge cache for a zone.
    pub async fn purge_cache(&self, zone_id: &str, purge_everything: bool) -> Result<Value> {
        let token = self.get_token()?;

        let body = if purge_everything {
            serde_json::json!({"purge_everything": true})
        } else {
            serde_json::json!({})
        };

        let response = self
            .client
            .post(format!("{}/zones/{}/purge_cache", API_BASE, zone_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to purge cache")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Cloudflare API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse response")
    }
}
