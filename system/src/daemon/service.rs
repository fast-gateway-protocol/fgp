//! System daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use fgp_daemon::FgpService;
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::system::SystemCache;

/// System information service.
pub struct SystemService {
    cache: Arc<SystemCache>,
}

impl SystemService {
    /// Create a new system service.
    pub fn new() -> Result<Self> {
        Ok(Self {
            cache: Arc::new(SystemCache::new()),
        })
    }

    /// Helper to get a string parameter.
    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Helper to get an integer parameter with default.
    fn get_param_u32(params: &HashMap<String, Value>, key: &str, default: u32) -> u32 {
        params
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(default)
    }

    /// Get hardware information.
    fn hardware(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let info = self.cache.hardware()?;

        Ok(serde_json::json!({
            "hardware": info,
        }))
    }

    /// Get disk information.
    fn disks(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let disks = self.cache.disks()?;

        Ok(serde_json::json!({
            "disks": disks,
            "count": disks.len(),
        }))
    }

    /// Get network interfaces.
    fn network(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let interfaces = self.cache.network()?;

        Ok(serde_json::json!({
            "interfaces": interfaces,
            "count": interfaces.len(),
        }))
    }

    /// Get running processes.
    fn processes(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 20);
        let processes = self.cache.processes(limit)?;

        Ok(serde_json::json!({
            "processes": processes,
            "count": processes.len(),
        }))
    }

    /// Get installed applications.
    fn apps(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_param_str(&params, "query");
        let limit = Self::get_param_u32(&params, "limit", 100) as usize;

        let mut apps = self.cache.apps()?;

        // Filter by query if provided
        if let Some(q) = query {
            let q_lower = q.to_lowercase();
            apps.retain(|a| {
                a.name.to_lowercase().contains(&q_lower)
                    || a.bundle_id
                        .as_ref()
                        .map(|b| b.to_lowercase().contains(&q_lower))
                        .unwrap_or(false)
            });
        }

        // Apply limit
        apps.truncate(limit);

        Ok(serde_json::json!({
            "apps": apps,
            "count": apps.len(),
        }))
    }

    /// Get battery information.
    fn battery(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let info = self.cache.battery()?;

        Ok(serde_json::json!({
            "battery": info,
        }))
    }

    /// Get system statistics.
    fn stats(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let stats = self.cache.stats()?;

        Ok(serde_json::json!({
            "stats": stats,
        }))
    }

    /// Invalidate all caches.
    fn invalidate(&self, _params: HashMap<String, Value>) -> Result<Value> {
        self.cache.invalidate_all();

        Ok(serde_json::json!({
            "status": "ok",
            "message": "All caches invalidated",
        }))
    }

    /// Get cache statistics.
    fn cache_info(&self, _params: HashMap<String, Value>) -> Result<Value> {
        Ok(serde_json::json!({
            "cache": self.cache.cache_stats(),
        }))
    }

    /// Bundle multiple queries for dashboard.
    fn bundle(&self, params: HashMap<String, Value>) -> Result<Value> {
        let include = Self::get_param_str(&params, "include").unwrap_or("hardware,stats,battery");

        let mut result = serde_json::Map::new();

        for query in include.split(',') {
            match query.trim() {
                "hardware" => {
                    if let Ok(hw) = self.cache.hardware() {
                        result.insert("hardware".into(), serde_json::to_value(hw)?);
                    }
                }
                "disks" => {
                    if let Ok(disks) = self.cache.disks() {
                        result.insert("disks".into(), serde_json::to_value(disks)?);
                    }
                }
                "network" => {
                    if let Ok(net) = self.cache.network() {
                        result.insert("network".into(), serde_json::to_value(net)?);
                    }
                }
                "processes" => {
                    if let Ok(procs) = self.cache.processes(10) {
                        result.insert("processes".into(), serde_json::to_value(procs)?);
                    }
                }
                "apps" => {
                    if let Ok(apps) = self.cache.apps() {
                        result.insert("apps_count".into(), apps.len().into());
                    }
                }
                "battery" => {
                    if let Ok(batt) = self.cache.battery() {
                        result.insert("battery".into(), serde_json::to_value(batt)?);
                    }
                }
                "stats" => {
                    if let Ok(stats) = self.cache.stats() {
                        result.insert("stats".into(), serde_json::to_value(stats)?);
                    }
                }
                _ => {}
            }
        }

        Ok(Value::Object(result))
    }
}

impl Default for SystemService {
    fn default() -> Self {
        Self::new().expect("Failed to create SystemService")
    }
}

impl FgpService for SystemService {
    fn name(&self) -> &str {
        "system"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "system.hardware" | "hardware" => self.hardware(params),
            "system.disks" | "disks" => self.disks(params),
            "system.network" | "network" => self.network(params),
            "system.processes" | "processes" => self.processes(params),
            "system.apps" | "apps" => self.apps(params),
            "system.battery" | "battery" => self.battery(params),
            "system.stats" | "stats" => self.stats(params),
            "system.invalidate" | "invalidate" => self.invalidate(params),
            "system.cache" | "cache" => self.cache_info(params),
            "system.bundle" | "bundle" => self.bundle(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("hardware", "Get hardware information (CPU, RAM, model)")
                .param(ParamInfo { name: "force".into(), param_type: "boolean".into(), required: false, default: Some(Value::Bool(false)) }),
            MethodInfo::new("disks", "Get disk usage information"),
            MethodInfo::new("network", "Get network interface information"),
            MethodInfo::new("processes", "Get running processes sorted by CPU usage")
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(20.into())) }),
            MethodInfo::new("apps", "List installed applications")
                .param(ParamInfo { name: "query".into(), param_type: "string".into(), required: false, default: None })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(100.into())) }),
            MethodInfo::new("battery", "Get battery status and health"),
            MethodInfo::new("stats", "Get system statistics (uptime, load, memory)"),
            MethodInfo::new("invalidate", "Invalidate all caches"),
            MethodInfo::new("cache", "Get cache statistics and TTL info"),
            MethodInfo::new("bundle", "Get multiple system info in one call")
                .param(ParamInfo { name: "include".into(), param_type: "string".into(), required: false, default: Some(Value::String("hardware,stats,battery".into())) }),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Hardware check
        let hw_ok = self.cache.hardware().is_ok();
        checks.insert(
            "hardware".into(),
            HealthStatus {
                ok: hw_ok,
                latency_ms: None,
                message: if hw_ok {
                    Some("Hardware info available".into())
                } else {
                    Some("Failed to query hardware".into())
                },
            },
        );

        // Battery check
        let battery_result = self.cache.battery();
        checks.insert(
            "battery".into(),
            HealthStatus {
                ok: battery_result.is_ok(),
                latency_ms: None,
                message: battery_result.ok().map(|b| format!("{}%, {}", b.percent, if b.charging { "charging" } else { "discharging" })),
            },
        );

        // Stats check
        let stats_ok = self.cache.stats().is_ok();
        checks.insert(
            "stats".into(),
            HealthStatus {
                ok: stats_ok,
                latency_ms: None,
                message: if stats_ok {
                    Some("System stats available".into())
                } else {
                    Some("Failed to query stats".into())
                },
            },
        );

        checks
    }

    fn on_start(&self) -> Result<()> {
        // Pre-warm caches
        tracing::info!("Pre-warming system caches...");

        let _ = self.cache.hardware();
        let _ = self.cache.stats();
        let _ = self.cache.battery();

        tracing::info!("System daemon ready");
        Ok(())
    }
}
