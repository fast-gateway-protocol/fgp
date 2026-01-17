//! FGP service implementation for Cloudflare.
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

use crate::api::CloudflareClient;

/// FGP service for Cloudflare operations.
pub struct CloudflareService {
    client: Arc<CloudflareClient>,
    runtime: Runtime,
}

impl CloudflareService {
    /// Create a new CloudflareService.
    pub fn new() -> Result<Self> {
        let client = CloudflareClient::new()?;
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
    // Zone operations
    // ========================================================================

    fn list_zones(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.list_zones().await
        })?;

        Ok(json!({
            "zones": result,
            "count": result.len()
        }))
    }

    fn get_zone(&self, params: HashMap<String, Value>) -> Result<Value> {
        let zone_id = Self::get_str(&params, "zone_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: zone_id"))?;

        let client = self.client.clone();
        let zone_id = zone_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_zone(&zone_id).await
        })?;

        Ok(json!(result))
    }

    // ========================================================================
    // DNS operations
    // ========================================================================

    fn list_dns_records(&self, params: HashMap<String, Value>) -> Result<Value> {
        let zone_id = Self::get_str(&params, "zone_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: zone_id"))?;

        let client = self.client.clone();
        let zone_id = zone_id.to_string();

        let result = self.runtime.block_on(async move {
            client.list_dns_records(&zone_id).await
        })?;

        Ok(json!({
            "records": result,
            "count": result.len()
        }))
    }

    fn create_dns_record(&self, params: HashMap<String, Value>) -> Result<Value> {
        let zone_id = Self::get_str(&params, "zone_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: zone_id"))?;
        let record_type = Self::get_str(&params, "type")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: type"))?;
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let content = Self::get_str(&params, "content")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        let ttl = Self::get_i64(&params, "ttl");
        let proxied = Self::get_bool(&params, "proxied");
        let priority = Self::get_i64(&params, "priority");

        let client = self.client.clone();
        let zone_id = zone_id.to_string();
        let record_type = record_type.to_string();
        let name = name.to_string();
        let content = content.to_string();

        let result = self.runtime.block_on(async move {
            client.create_dns_record(&zone_id, &record_type, &name, &content, ttl, proxied, priority).await
        })?;

        Ok(json!(result))
    }

    fn update_dns_record(&self, params: HashMap<String, Value>) -> Result<Value> {
        let zone_id = Self::get_str(&params, "zone_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: zone_id"))?;
        let record_id = Self::get_str(&params, "record_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: record_id"))?;
        let record_type = Self::get_str(&params, "type")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: type"))?;
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let content = Self::get_str(&params, "content")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        let ttl = Self::get_i64(&params, "ttl");
        let proxied = Self::get_bool(&params, "proxied");

        let client = self.client.clone();
        let zone_id = zone_id.to_string();
        let record_id = record_id.to_string();
        let record_type = record_type.to_string();
        let name = name.to_string();
        let content = content.to_string();

        let result = self.runtime.block_on(async move {
            client.update_dns_record(&zone_id, &record_id, &record_type, &name, &content, ttl, proxied).await
        })?;

        Ok(json!(result))
    }

    fn delete_dns_record(&self, params: HashMap<String, Value>) -> Result<Value> {
        let zone_id = Self::get_str(&params, "zone_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: zone_id"))?;
        let record_id = Self::get_str(&params, "record_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: record_id"))?;

        let client = self.client.clone();
        let zone_id = zone_id.to_string();
        let record_id = record_id.to_string();
        let record_id_clone = record_id.clone();

        self.runtime.block_on(async move {
            client.delete_dns_record(&zone_id, &record_id_clone).await
        })?;

        Ok(json!({"deleted": true, "record_id": record_id}))
    }

    // ========================================================================
    // KV operations
    // ========================================================================

    fn list_kv_namespaces(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.list_kv_namespaces().await
        })?;

        Ok(json!({
            "namespaces": result,
            "count": result.len()
        }))
    }

    fn list_kv_keys(&self, params: HashMap<String, Value>) -> Result<Value> {
        let namespace_id = Self::get_str(&params, "namespace_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: namespace_id"))?;

        let client = self.client.clone();
        let namespace_id = namespace_id.to_string();

        let result = self.runtime.block_on(async move {
            client.list_kv_keys(&namespace_id).await
        })?;

        Ok(json!({
            "keys": result,
            "count": result.len()
        }))
    }

    fn read_kv(&self, params: HashMap<String, Value>) -> Result<Value> {
        let namespace_id = Self::get_str(&params, "namespace_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: namespace_id"))?;
        let key = Self::get_str(&params, "key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;

        let client = self.client.clone();
        let namespace_id = namespace_id.to_string();
        let key = key.to_string();
        let key_clone = key.clone();

        let result = self.runtime.block_on(async move {
            client.read_kv(&namespace_id, &key_clone).await
        })?;

        Ok(json!({"key": key, "value": result}))
    }

    fn write_kv(&self, params: HashMap<String, Value>) -> Result<Value> {
        let namespace_id = Self::get_str(&params, "namespace_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: namespace_id"))?;
        let key = Self::get_str(&params, "key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        let value = Self::get_str(&params, "value")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: value"))?;

        let client = self.client.clone();
        let namespace_id = namespace_id.to_string();
        let key = key.to_string();
        let key_clone = key.clone();
        let value = value.to_string();

        self.runtime.block_on(async move {
            client.write_kv(&namespace_id, &key_clone, &value).await
        })?;

        Ok(json!({"written": true, "key": key}))
    }

    fn delete_kv(&self, params: HashMap<String, Value>) -> Result<Value> {
        let namespace_id = Self::get_str(&params, "namespace_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: namespace_id"))?;
        let key = Self::get_str(&params, "key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;

        let client = self.client.clone();
        let namespace_id = namespace_id.to_string();
        let key = key.to_string();
        let key_clone = key.clone();

        self.runtime.block_on(async move {
            client.delete_kv(&namespace_id, &key_clone).await
        })?;

        Ok(json!({"deleted": true, "key": key}))
    }

    // ========================================================================
    // Worker operations
    // ========================================================================

    fn list_workers(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.list_workers().await
        })?;

        Ok(json!({
            "workers": result,
            "count": result.len()
        }))
    }

    fn list_worker_routes(&self, params: HashMap<String, Value>) -> Result<Value> {
        let zone_id = Self::get_str(&params, "zone_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: zone_id"))?;

        let client = self.client.clone();
        let zone_id = zone_id.to_string();

        let result = self.runtime.block_on(async move {
            client.list_worker_routes(&zone_id).await
        })?;

        Ok(json!({
            "routes": result,
            "count": result.len()
        }))
    }

    fn purge_cache(&self, params: HashMap<String, Value>) -> Result<Value> {
        let zone_id = Self::get_str(&params, "zone_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: zone_id"))?;
        let purge_everything = Self::get_bool(&params, "purge_everything").unwrap_or(true);

        let client = self.client.clone();
        let zone_id = zone_id.to_string();

        let result = self.runtime.block_on(async move {
            client.purge_cache(&zone_id, purge_everything).await
        })?;

        Ok(result)
    }
}

impl FgpService for CloudflareService {
    fn name(&self) -> &str {
        "cloudflare"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // Zones
            "zones" | "cloudflare.zones" => self.list_zones(params),
            "zone" | "cloudflare.zone" => self.get_zone(params),

            // DNS
            "dns.list" | "cloudflare.dns.list" => self.list_dns_records(params),
            "dns.create" | "cloudflare.dns.create" => self.create_dns_record(params),
            "dns.update" | "cloudflare.dns.update" => self.update_dns_record(params),
            "dns.delete" | "cloudflare.dns.delete" => self.delete_dns_record(params),

            // KV
            "kv.namespaces" | "cloudflare.kv.namespaces" => self.list_kv_namespaces(params),
            "kv.keys" | "cloudflare.kv.keys" => self.list_kv_keys(params),
            "kv.read" | "cloudflare.kv.read" => self.read_kv(params),
            "kv.write" | "cloudflare.kv.write" => self.write_kv(params),
            "kv.delete" | "cloudflare.kv.delete" => self.delete_kv(params),

            // Workers
            "workers" | "cloudflare.workers" => self.list_workers(params),
            "workers.routes" | "cloudflare.workers.routes" => self.list_worker_routes(params),

            // Cache
            "purge_cache" | "cloudflare.purge_cache" => self.purge_cache(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // Zones
            MethodInfo::new("cloudflare.zones", "List all zones"),
            MethodInfo::new("cloudflare.zone", "Get zone by ID")
                .schema(
                    SchemaBuilder::object()
                        .property("zone_id", SchemaBuilder::string().description("Zone ID"))
                        .required(&["zone_id"])
                        .build(),
                ),

            // DNS
            MethodInfo::new("cloudflare.dns.list", "List DNS records for a zone")
                .schema(
                    SchemaBuilder::object()
                        .property("zone_id", SchemaBuilder::string().description("Zone ID"))
                        .required(&["zone_id"])
                        .build(),
                ),
            MethodInfo::new("cloudflare.dns.create", "Create a DNS record")
                .schema(
                    SchemaBuilder::object()
                        .property("zone_id", SchemaBuilder::string().description("Zone ID"))
                        .property("type", SchemaBuilder::string().description("Record type (A, AAAA, CNAME, MX, TXT, etc.)"))
                        .property("name", SchemaBuilder::string().description("Record name"))
                        .property("content", SchemaBuilder::string().description("Record content/value"))
                        .property("ttl", SchemaBuilder::integer().description("TTL in seconds (1 = auto)"))
                        .property("proxied", SchemaBuilder::boolean().description("Proxy through Cloudflare"))
                        .property("priority", SchemaBuilder::integer().description("Priority (for MX records)"))
                        .required(&["zone_id", "type", "name", "content"])
                        .build(),
                ),
            MethodInfo::new("cloudflare.dns.update", "Update a DNS record")
                .schema(
                    SchemaBuilder::object()
                        .property("zone_id", SchemaBuilder::string().description("Zone ID"))
                        .property("record_id", SchemaBuilder::string().description("Record ID"))
                        .property("type", SchemaBuilder::string().description("Record type"))
                        .property("name", SchemaBuilder::string().description("Record name"))
                        .property("content", SchemaBuilder::string().description("Record content/value"))
                        .property("ttl", SchemaBuilder::integer().description("TTL in seconds"))
                        .property("proxied", SchemaBuilder::boolean().description("Proxy through Cloudflare"))
                        .required(&["zone_id", "record_id", "type", "name", "content"])
                        .build(),
                ),
            MethodInfo::new("cloudflare.dns.delete", "Delete a DNS record")
                .schema(
                    SchemaBuilder::object()
                        .property("zone_id", SchemaBuilder::string().description("Zone ID"))
                        .property("record_id", SchemaBuilder::string().description("Record ID"))
                        .required(&["zone_id", "record_id"])
                        .build(),
                ),

            // KV
            MethodInfo::new("cloudflare.kv.namespaces", "List KV namespaces"),
            MethodInfo::new("cloudflare.kv.keys", "List keys in a KV namespace")
                .schema(
                    SchemaBuilder::object()
                        .property("namespace_id", SchemaBuilder::string().description("Namespace ID"))
                        .required(&["namespace_id"])
                        .build(),
                ),
            MethodInfo::new("cloudflare.kv.read", "Read a KV value")
                .schema(
                    SchemaBuilder::object()
                        .property("namespace_id", SchemaBuilder::string().description("Namespace ID"))
                        .property("key", SchemaBuilder::string().description("Key name"))
                        .required(&["namespace_id", "key"])
                        .build(),
                ),
            MethodInfo::new("cloudflare.kv.write", "Write a KV value")
                .schema(
                    SchemaBuilder::object()
                        .property("namespace_id", SchemaBuilder::string().description("Namespace ID"))
                        .property("key", SchemaBuilder::string().description("Key name"))
                        .property("value", SchemaBuilder::string().description("Value to write"))
                        .required(&["namespace_id", "key", "value"])
                        .build(),
                ),
            MethodInfo::new("cloudflare.kv.delete", "Delete a KV key")
                .schema(
                    SchemaBuilder::object()
                        .property("namespace_id", SchemaBuilder::string().description("Namespace ID"))
                        .property("key", SchemaBuilder::string().description("Key name"))
                        .required(&["namespace_id", "key"])
                        .build(),
                ),

            // Workers
            MethodInfo::new("cloudflare.workers", "List Workers"),
            MethodInfo::new("cloudflare.workers.routes", "List Worker routes for a zone")
                .schema(
                    SchemaBuilder::object()
                        .property("zone_id", SchemaBuilder::string().description("Zone ID"))
                        .required(&["zone_id"])
                        .build(),
                ),

            // Cache
            MethodInfo::new("cloudflare.purge_cache", "Purge cache for a zone")
                .schema(
                    SchemaBuilder::object()
                        .property("zone_id", SchemaBuilder::string().description("Zone ID"))
                        .property("purge_everything", SchemaBuilder::boolean().description("Purge all cached content"))
                        .required(&["zone_id"])
                        .build(),
                ),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        checks.insert(
            "api_token".into(),
            if self.client.has_token() {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("CLOUDFLARE_API_TOKEN not set")
            },
        );

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_env() {
        INIT.call_once(|| {
            std::env::set_var("CLOUDFLARE_API_TOKEN", "test-token");
            std::env::set_var("CLOUDFLARE_ACCOUNT_ID", "test-account");
        });
    }

    fn test_service() -> CloudflareService {
        ensure_env();
        CloudflareService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("zone_id".to_string(), Value::String("zone".to_string()));
        params.insert("ttl".to_string(), Value::from(60));
        params.insert("proxied".to_string(), Value::Bool(true));

        assert_eq!(CloudflareService::get_str(&params, "zone_id"), Some("zone"));
        assert_eq!(CloudflareService::get_str(&params, "missing"), None);
        assert_eq!(CloudflareService::get_i64(&params, "ttl"), Some(60));
        assert_eq!(CloudflareService::get_i64(&params, "missing"), None);
        assert_eq!(CloudflareService::get_bool(&params, "proxied"), Some(true));
        assert_eq!(CloudflareService::get_bool(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_defaults() {
        let service = test_service();
        let methods = service.method_list();

        let purge = methods
            .iter()
            .find(|m| m.name == "cloudflare.purge_cache")
            .expect("purge_cache");
        let purge_schema = purge.schema.as_ref().expect("schema");
        let purge_props = purge_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        let purge_default = purge_props
            .get("purge_everything")
            .and_then(Value::as_object)
            .and_then(|p| p.get("default"));
        assert!(purge_default.is_none());

        let dns_create = methods
            .iter()
            .find(|m| m.name == "cloudflare.dns.create")
            .expect("dns.create");
        let dns_schema = dns_create.schema.as_ref().expect("schema");
        let dns_props = dns_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        let ttl = dns_props.get("ttl").and_then(Value::as_object);
        assert!(ttl.is_some());
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("cloudflare.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
