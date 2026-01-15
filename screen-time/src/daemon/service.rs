//! Screen Time daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use chrono::{Duration, NaiveDate, Utc};
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;

use crate::screen_time::ScreenTimeStore;

/// Screen Time service.
pub struct ScreenTimeService;

impl ScreenTimeService {
    /// Create a new Screen Time service.
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Helper to get an optional string parameter.
    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Helper to get an optional integer parameter.
    fn get_param_i64(params: &HashMap<String, Value>, key: &str) -> Option<i64> {
        params.get(key).and_then(|v| v.as_i64())
    }

    /// Get daily total screen time.
    fn daily_total(&self, params: HashMap<String, Value>) -> Result<Value> {
        let date = if let Some(date_str) = Self::get_param_str(&params, "date") {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| anyhow!("Invalid date format (use YYYY-MM-DD): {}", e))?
        } else {
            chrono::Local::now().date_naive()
        };

        let store = ScreenTimeStore::new()?;
        let daily = store.daily_total(date)?;

        Ok(serde_json::to_value(daily)?)
    }

    /// Get app usage for a specific bundle ID.
    fn app_usage(&self, params: HashMap<String, Value>) -> Result<Value> {
        let bundle_id = Self::get_param_str(&params, "bundle_id")
            .ok_or_else(|| anyhow!("Missing required parameter: bundle_id"))?;

        let days = Self::get_param_i64(&params, "days").unwrap_or(7);

        let end = Utc::now();
        let start = end - Duration::days(days);

        let store = ScreenTimeStore::new()?;
        let sessions = store.app_usage(bundle_id, start, end)?;

        let total_seconds: i64 = sessions.iter().map(|s| s.duration_seconds).sum();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let total_formatted = if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        };

        Ok(serde_json::json!({
            "bundle_id": bundle_id,
            "days": days,
            "total_seconds": total_seconds,
            "total_formatted": total_formatted,
            "session_count": sessions.len(),
            "sessions": sessions,
        }))
    }

    /// Get weekly summary.
    fn weekly_summary(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let store = ScreenTimeStore::new()?;
        let summary = store.weekly_summary()?;

        let total_seconds: i64 = summary.iter().map(|d| d.total_seconds).sum();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;

        Ok(serde_json::json!({
            "total_seconds": total_seconds,
            "total_formatted": format!("{}h {}m", hours, minutes),
            "daily_average_seconds": total_seconds / 7,
            "days": summary,
        }))
    }

    /// Get most used apps.
    fn most_used(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_i64(&params, "limit").unwrap_or(10) as usize;
        let days = Self::get_param_i64(&params, "days").unwrap_or(7);

        let end = Utc::now();
        let start = end - Duration::days(days);

        let store = ScreenTimeStore::new()?;
        let apps = store.most_used(start, end, limit)?;

        Ok(serde_json::json!({
            "period_days": days,
            "limit": limit,
            "apps": apps,
        }))
    }

    /// Get hourly usage timeline.
    fn usage_timeline(&self, params: HashMap<String, Value>) -> Result<Value> {
        let date = if let Some(date_str) = Self::get_param_str(&params, "date") {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| anyhow!("Invalid date format (use YYYY-MM-DD): {}", e))?
        } else {
            chrono::Local::now().date_naive()
        };

        let store = ScreenTimeStore::new()?;
        let hourly = store.usage_timeline(date)?;

        // Convert to array format for easier consumption
        let mut timeline: Vec<serde_json::Value> = (0..24)
            .map(|hour| {
                let seconds = hourly.get(&hour).copied().unwrap_or(0);
                let minutes = seconds / 60;
                serde_json::json!({
                    "hour": hour,
                    "seconds": seconds,
                    "formatted": format!("{}m", minutes),
                })
            })
            .collect();

        // Sort by hour
        timeline.sort_by_key(|v| v["hour"].as_u64().unwrap_or(0));

        Ok(serde_json::json!({
            "date": date.format("%Y-%m-%d").to_string(),
            "timeline": timeline,
        }))
    }

    /// Check authorization status.
    fn auth_status(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let accessible = ScreenTimeStore::check_access();

        Ok(serde_json::json!({
            "accessible": accessible,
            "message": if accessible {
                "Screen Time data accessible via Full Disk Access."
            } else {
                "Screen Time data not accessible. Grant Full Disk Access in System Settings > Privacy & Security."
            },
        }))
    }
}

impl Default for ScreenTimeService {
    fn default() -> Self {
        Self::new().expect("Failed to create ScreenTimeService")
    }
}

impl FgpService for ScreenTimeService {
    fn name(&self) -> &str {
        "screen-time"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "screen-time.daily_total" | "daily_total" => self.daily_total(params),
            "screen-time.app_usage" | "app_usage" => self.app_usage(params),
            "screen-time.weekly_summary" | "weekly_summary" => self.weekly_summary(params),
            "screen-time.most_used" | "most_used" => self.most_used(params),
            "screen-time.usage_timeline" | "usage_timeline" => self.usage_timeline(params),
            "screen-time.auth" | "auth" => self.auth_status(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("daily_total", "Get total screen time for a day with per-app breakdown")
                .param(ParamInfo {
                    name: "date".into(),
                    param_type: "string".into(),
                    required: false,
                    default: Some("today".into()),
                }),
            MethodInfo::new("app_usage", "Get usage for a specific app")
                .param(ParamInfo {
                    name: "bundle_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "days".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some("7".into()),
                }),
            MethodInfo::new("weekly_summary", "Get 7-day summary with daily breakdown"),
            MethodInfo::new("most_used", "Get top N most used apps")
                .param(ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some("10".into()),
                })
                .param(ParamInfo {
                    name: "days".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some("7".into()),
                }),
            MethodInfo::new("usage_timeline", "Get hourly breakdown for a day")
                .param(ParamInfo {
                    name: "date".into(),
                    param_type: "string".into(),
                    required: false,
                    default: Some("today".into()),
                }),
            MethodInfo::new("auth", "Check Screen Time data access status"),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let accessible = ScreenTimeStore::check_access();
        checks.insert(
            "access".into(),
            HealthStatus {
                ok: accessible,
                latency_ms: None,
                message: Some(if accessible {
                    "knowledgeC.db accessible".to_string()
                } else {
                    "knowledgeC.db not accessible - check Full Disk Access".to_string()
                }),
            },
        );

        checks
    }

    fn on_start(&self) -> Result<()> {
        let accessible = ScreenTimeStore::check_access();
        if accessible {
            tracing::info!("Screen Time data accessible");
        } else {
            tracing::warn!(
                "Screen Time data may not be accessible. Grant Full Disk Access in System Settings."
            );
        }
        Ok(())
    }
}
