//! FGP service implementation for Zapier.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;

use crate::api::ZapierClient;
use crate::models::{Webhook, WebhookRegistry};

/// FGP service for Zapier operations.
pub struct ZapierService {
    client: Arc<ZapierClient>,
    runtime: Runtime,
    registry: Arc<RwLock<WebhookRegistry>>,
    registry_path: PathBuf,
}

impl ZapierService {
    /// Create a new ZapierService.
    pub fn new() -> Result<Self> {
        let client = ZapierClient::new()?;
        let runtime = Runtime::new()?;

        // Load webhook registry from disk
        let registry_path = Self::registry_path()?;
        let registry = Self::load_registry(&registry_path);

        Ok(Self {
            client: Arc::new(client),
            runtime,
            registry: Arc::new(RwLock::new(registry)),
            registry_path,
        })
    }

    /// Get registry file path.
    fn registry_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let path = home.join(".fgp/services/zapier/webhooks.json");

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(path)
    }

    /// Load registry from disk.
    fn load_registry(path: &PathBuf) -> WebhookRegistry {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(registry) = serde_json::from_str(&content) {
                    return registry;
                }
            }
        }
        WebhookRegistry::default()
    }

    /// Save registry to disk.
    fn save_registry(&self) -> Result<()> {
        let registry = self.registry.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let content = serde_json::to_string_pretty(&*registry)?;
        std::fs::write(&self.registry_path, content)?;
        Ok(())
    }

    /// Helper to get a string parameter.
    fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Get current timestamp as ISO 8601 string.
    fn timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("{}", secs)
    }

    // ========================================================================
    // Webhook methods
    // ========================================================================

    fn trigger(&self, params: HashMap<String, Value>) -> Result<Value> {
        let url = Self::get_str(&params, "url")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

        let payload = params.get("payload").cloned().unwrap_or(json!({}));

        let client = self.client.clone();
        let url = url.to_string();

        let result = self.runtime.block_on(async move {
            client.trigger_webhook(&url, &payload).await
        })?;

        Ok(json!(result))
    }

    fn trigger_named(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

        let registry = self.registry.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let webhook = registry.webhooks.get(name)
            .ok_or_else(|| anyhow::anyhow!("Webhook '{}' not found", name))?;
        let url = webhook.url.clone();
        drop(registry);

        let payload = params.get("payload").cloned().unwrap_or(json!({}));

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.trigger_webhook(&url, &payload).await
        })?;

        Ok(json!(result))
    }

    fn register_webhook(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let url = Self::get_str(&params, "url")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;
        let description = Self::get_str(&params, "description");

        let webhook = Webhook {
            name: name.to_string(),
            url: url.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: Some(Self::timestamp()),
        };

        {
            let mut registry = self.registry.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            registry.webhooks.insert(name.to_string(), webhook.clone());
        }

        self.save_registry()?;

        Ok(json!({
            "registered": true,
            "webhook": webhook
        }))
    }

    fn list_webhooks(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let registry = self.registry.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        let webhooks: Vec<&Webhook> = registry.webhooks.values().collect();

        Ok(json!({
            "webhooks": webhooks,
            "count": webhooks.len()
        }))
    }

    fn remove_webhook(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

        let removed = {
            let mut registry = self.registry.write().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            registry.webhooks.remove(name).is_some()
        };

        if removed {
            self.save_registry()?;
        }

        Ok(json!({
            "removed": removed,
            "name": name
        }))
    }

    // ========================================================================
    // NLA methods
    // ========================================================================

    fn nla_actions(&self, _params: HashMap<String, Value>) -> Result<Value> {
        if !self.client.has_nla() {
            return Err(anyhow::anyhow!("ZAPIER_NLA_API_KEY not set"));
        }

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.nla_actions().await
        })?;

        Ok(json!({
            "actions": result,
            "count": result.len()
        }))
    }

    fn nla_execute(&self, params: HashMap<String, Value>) -> Result<Value> {
        if !self.client.has_nla() {
            return Err(anyhow::anyhow!("ZAPIER_NLA_API_KEY not set"));
        }

        let action_id = Self::get_str(&params, "action_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action_id"))?;
        let instructions = Self::get_str(&params, "instructions")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: instructions"))?;

        let client = self.client.clone();
        let action_id = action_id.to_string();
        let instructions = instructions.to_string();

        let result = self.runtime.block_on(async move {
            client.nla_execute(&action_id, &instructions).await
        })?;

        Ok(json!(result))
    }
}

impl FgpService for ZapierService {
    fn name(&self) -> &str {
        "zapier"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            // Webhooks
            "trigger" | "zapier.trigger" => self.trigger(params),
            "trigger_named" | "zapier.trigger_named" => self.trigger_named(params),
            "register_webhook" | "zapier.register_webhook" => self.register_webhook(params),
            "list_webhooks" | "zapier.list_webhooks" => self.list_webhooks(params),
            "remove_webhook" | "zapier.remove_webhook" => self.remove_webhook(params),

            // NLA
            "nla_actions" | "zapier.nla_actions" => self.nla_actions(params),
            "nla_execute" | "zapier.nla_execute" => self.nla_execute(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // Webhooks
            MethodInfo::new("zapier.trigger", "Trigger a Zap via webhook URL")
                .schema(
                    SchemaBuilder::object()
                        .property("url", SchemaBuilder::string().description("Webhook URL"))
                        .property("payload", SchemaBuilder::object().description("Data to send"))
                        .required(&["url"])
                        .build(),
                )
                .example(
                    "Trigger webhook",
                    json!({
                        "url": "https://hooks.zapier.com/hooks/catch/xxx/yyy/",
                        "payload": {"name": "John", "email": "john@example.com"}
                    }),
                ),

            MethodInfo::new("zapier.trigger_named", "Trigger registered Zap by name")
                .schema(
                    SchemaBuilder::object()
                        .property("name", SchemaBuilder::string().description("Registered webhook name"))
                        .property("payload", SchemaBuilder::object().description("Data to send"))
                        .required(&["name"])
                        .build(),
                )
                .example("Trigger by name", json!({"name": "notify-slack", "payload": {"message": "Hello"}})),

            MethodInfo::new("zapier.register_webhook", "Register webhook URL with alias")
                .schema(
                    SchemaBuilder::object()
                        .property("name", SchemaBuilder::string().description("Alias for the webhook"))
                        .property("url", SchemaBuilder::string().description("Webhook URL"))
                        .property("description", SchemaBuilder::string().description("Optional description"))
                        .required(&["name", "url"])
                        .build(),
                ),

            MethodInfo::new("zapier.list_webhooks", "List registered webhooks"),

            MethodInfo::new("zapier.remove_webhook", "Remove registered webhook")
                .schema(
                    SchemaBuilder::object()
                        .property("name", SchemaBuilder::string())
                        .required(&["name"])
                        .build(),
                ),

            // NLA
            MethodInfo::new("zapier.nla_actions", "List available NLA actions (requires API key)"),

            MethodInfo::new("zapier.nla_execute", "Execute NLA action")
                .schema(
                    SchemaBuilder::object()
                        .property("action_id", SchemaBuilder::string().description("NLA action ID"))
                        .property("instructions", SchemaBuilder::string().description("Natural language instructions"))
                        .required(&["action_id", "instructions"])
                        .build(),
                ),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Webhook registry check
        checks.insert(
            "webhook_registry".into(),
            if self.registry_path.exists() {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("Registry file not found")
            },
        );

        // NLA API check
        checks.insert(
            "nla_api".into(),
            if self.client.has_nla() {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("ZAPIER_NLA_API_KEY not set")
            },
        );

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::ZapierService;
    use crate::api::ZapierClient;
    use crate::models::WebhookRegistry;
    use fgp_daemon::FgpService;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use tokio::runtime::Runtime;

    fn test_service() -> ZapierService {
        let client = ZapierClient::new().expect("client");
        let runtime = Runtime::new().expect("runtime");
        ZapierService {
            client: Arc::new(client),
            runtime,
            registry: Arc::new(RwLock::new(WebhookRegistry::default())),
            registry_path: std::env::temp_dir().join("fgp-zapier-tests.json"),
        }
    }

    #[test]
    fn get_str_and_timestamp_helpers() {
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));

        assert_eq!(ZapierService::get_str(&params, "url"), Some("https://example.com"));
        assert_eq!(ZapierService::get_str(&params, "missing"), None);

        let stamp = ZapierService::timestamp();
        assert!(!stamp.is_empty());
        assert!(stamp.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn method_list_includes_required_fields() {
        let methods = test_service().method_list();

        let trigger = methods
            .iter()
            .find(|m| m.name == "zapier.trigger")
            .expect("trigger");
        let trigger_schema = trigger.schema.as_ref().expect("schema");
        let trigger_required = trigger_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(trigger_required.iter().any(|v| v == "url"));

        let register = methods
            .iter()
            .find(|m| m.name == "zapier.register_webhook")
            .expect("register_webhook");
        let register_schema = register.schema.as_ref().expect("schema");
        let register_required = register_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(register_required.iter().any(|v| v == "name"));
        assert!(register_required.iter().any(|v| v == "url"));

        let nla_execute = methods
            .iter()
            .find(|m| m.name == "zapier.nla_execute")
            .expect("nla_execute");
        let nla_schema = nla_execute.schema.as_ref().expect("schema");
        let nla_required = nla_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(nla_required.iter().any(|v| v == "action_id"));
        assert!(nla_required.iter().any(|v| v == "instructions"));
    }

    #[test]
    fn dispatch_rejects_unknown_method() {
        let service = test_service();
        let err = service
            .dispatch("zapier.nope", HashMap::new())
            .expect_err("unknown method");
        assert!(err.to_string().contains("Unknown method"));
    }
}
