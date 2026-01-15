//! Safari FGP daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use fgp_daemon::service::{FgpService, HealthStatus, MethodInfo, ParamInfo};
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::connection::{open_cloud_tabs_db, open_history_db};
use crate::db::queries;

/// Safari daemon service with hot database connections.
pub struct SafariService {
    history_conn: Mutex<Connection>,
    cloud_tabs_conn: Option<Mutex<Connection>>,
    started_at: String,
}

impl SafariService {
    /// Create new Safari service with hot connections.
    pub fn new() -> Result<Self> {
        let history_conn = Mutex::new(open_history_db()?);

        // CloudTabs is optional (may not exist if iCloud not enabled)
        let cloud_tabs_conn = open_cloud_tabs_db().ok().map(Mutex::new);

        let started_at = chrono::Utc::now().to_rfc3339();

        Ok(Self {
            history_conn,
            cloud_tabs_conn,
            started_at,
        })
    }

    // ========================================================================
    // Parameter Helpers
    // ========================================================================

    fn get_param_u32(params: &HashMap<String, Value>, key: &str, default: u32) -> u32 {
        params
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(default)
    }

    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    // ========================================================================
    // Handlers
    // ========================================================================

    /// Recent history handler.
    /// Params: days (default 7), limit (default 50)
    fn history(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 7);
        let limit = Self::get_param_u32(&params, "limit", 50);

        let items = queries::query_recent_history(&self.history_conn.lock().unwrap(), days, limit)?;

        Ok(serde_json::json!({
            "items": items,
            "count": items.len(),
            "days": days,
        }))
    }

    /// Search history handler.
    /// Params: query (required), days (default 30), limit (default 50)
    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_param_str(&params, "query")
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;
        let days = Self::get_param_u32(&params, "days", 30);
        let limit = Self::get_param_u32(&params, "limit", 50);

        let items =
            queries::query_search_history(&self.history_conn.lock().unwrap(), query, days, limit)?;

        Ok(serde_json::json!({
            "items": items,
            "count": items.len(),
            "query": query,
            "days": days,
        }))
    }

    /// Top sites handler.
    /// Params: days (default 30), limit (default 20)
    fn top_sites(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 30);
        let limit = Self::get_param_u32(&params, "limit", 20);

        let sites = queries::query_top_sites(&self.history_conn.lock().unwrap(), days, limit)?;

        Ok(serde_json::json!({
            "sites": sites,
            "count": sites.len(),
            "days": days,
        }))
    }

    /// History stats handler.
    /// Params: days (default 30)
    fn stats(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 30);

        let (total_visits, unique_pages, active_days) =
            queries::query_history_stats(&self.history_conn.lock().unwrap(), days)?;

        Ok(serde_json::json!({
            "total_visits": total_visits,
            "unique_pages": unique_pages,
            "active_days": active_days,
            "period_days": days,
            "avg_visits_per_day": if active_days > 0 { total_visits / active_days } else { 0 },
        }))
    }

    /// Cloud tabs handler.
    /// Returns tabs from all synced devices.
    fn cloud_tabs(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let conn = self
            .cloud_tabs_conn
            .as_ref()
            .ok_or_else(|| anyhow!("CloudTabs not available (iCloud sync may be disabled)"))?;

        let tabs = queries::query_cloud_tabs(&conn.lock().unwrap())?;
        let devices = queries::query_cloud_devices(&conn.lock().unwrap())?;

        Ok(serde_json::json!({
            "tabs": tabs,
            "devices": devices,
            "total_tabs": tabs.len(),
            "total_devices": devices.len(),
        }))
    }

    /// Bundle handler - combines multiple queries.
    /// Params: include (comma-separated: history,top_sites,stats,cloud_tabs)
    fn bundle(&self, params: HashMap<String, Value>) -> Result<Value> {
        let include = Self::get_param_str(&params, "include").unwrap_or("history,top_sites");
        let sections: Vec<&str> = include.split(',').map(|s| s.trim()).collect();
        let mut result = serde_json::Map::new();

        for section in sections {
            match section {
                "history" => {
                    let limit = Self::get_param_u32(&params, "history_limit", 20);
                    let days = Self::get_param_u32(&params, "history_days", 7);
                    let items =
                        queries::query_recent_history(&self.history_conn.lock().unwrap(), days, limit)?;
                    result.insert("history".to_string(), serde_json::json!(items));
                }
                "top_sites" => {
                    let limit = Self::get_param_u32(&params, "top_sites_limit", 10);
                    let days = Self::get_param_u32(&params, "top_sites_days", 30);
                    let sites =
                        queries::query_top_sites(&self.history_conn.lock().unwrap(), days, limit)?;
                    result.insert("top_sites".to_string(), serde_json::json!(sites));
                }
                "stats" => {
                    let days = Self::get_param_u32(&params, "stats_days", 30);
                    let (total, unique, active) =
                        queries::query_history_stats(&self.history_conn.lock().unwrap(), days)?;
                    result.insert(
                        "stats".to_string(),
                        serde_json::json!({
                            "total_visits": total,
                            "unique_pages": unique,
                            "active_days": active,
                        }),
                    );
                }
                "cloud_tabs" => {
                    if let Some(ref conn) = self.cloud_tabs_conn {
                        let tabs = queries::query_cloud_tabs(&conn.lock().unwrap())?;
                        result.insert(
                            "cloud_tabs".to_string(),
                            serde_json::json!({
                                "tabs": tabs,
                                "count": tabs.len(),
                            }),
                        );
                    }
                }
                _ => {
                    // Unknown section, skip silently
                }
            }
        }

        Ok(Value::Object(result))
    }
}

impl FgpService for SafariService {
    fn name(&self) -> &str {
        "safari"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "safari.history" | "history" => self.history(params),
            "safari.search" | "search" => self.search(params),
            "safari.top_sites" | "top_sites" => self.top_sites(params),
            "safari.stats" | "stats" => self.stats(params),
            "safari.cloud_tabs" | "cloud_tabs" => self.cloud_tabs(params),
            "safari.bundle" | "bundle" => self.bundle(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo {
                name: "history".into(),
                description: "Get recent browser history".into(),
                params: vec![
                    ParamInfo {
                        name: "days".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(7.into())),
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(50.into())),
                    },
                ],
            },
            MethodInfo {
                name: "search".into(),
                description: "Search history by URL or title".into(),
                params: vec![
                    ParamInfo {
                        name: "query".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "days".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(30.into())),
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(50.into())),
                    },
                ],
            },
            MethodInfo {
                name: "top_sites".into(),
                description: "Get most visited sites".into(),
                params: vec![
                    ParamInfo {
                        name: "days".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(30.into())),
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(20.into())),
                    },
                ],
            },
            MethodInfo {
                name: "stats".into(),
                description: "Get browsing statistics".into(),
                params: vec![ParamInfo {
                    name: "days".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(30.into())),
                }],
            },
            MethodInfo {
                name: "cloud_tabs".into(),
                description: "Get tabs from other devices via iCloud".into(),
                params: vec![],
            },
            MethodInfo {
                name: "bundle".into(),
                description: "Bundle multiple queries for dashboard".into(),
                params: vec![ParamInfo {
                    name: "include".into(),
                    param_type: "string".into(),
                    required: false,
                    default: Some(Value::String("history,top_sites".into())),
                }],
            },
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // History database check
        checks.insert(
            "history_db".into(),
            HealthStatus {
                ok: true,
                latency_ms: None,
                message: Some("Connected to History.db".into()),
            },
        );

        // CloudTabs check
        let cloud_tabs_ok = self.cloud_tabs_conn.is_some();
        checks.insert(
            "cloud_tabs_db".into(),
            HealthStatus {
                ok: cloud_tabs_ok,
                latency_ms: None,
                message: Some(if cloud_tabs_ok {
                    "Connected to CloudTabs.db".into()
                } else {
                    "CloudTabs.db not available".into()
                }),
            },
        );

        checks
    }

    fn on_start(&self) -> Result<()> {
        let cloud_tabs_status = if self.cloud_tabs_conn.is_some() {
            "available"
        } else {
            "not available"
        };
        tracing::info!(
            cloud_tabs = cloud_tabs_status,
            "Safari daemon starting - databases loaded"
        );
        Ok(())
    }
}
