//! Apple Calendar daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDate, TimeZone};
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;

use crate::calendar::CalendarStore;

/// Apple Calendar service.
/// Note: CalendarStore is not stored because EKEventStore is not Send/Sync.
/// We create a fresh CalendarStore for each request.
pub struct CalendarService;

impl CalendarService {
    /// Create a new calendar service.
    pub fn new() -> Result<Self> {
        Ok(Self)
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

    /// List all calendars.
    fn calendars(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let store = CalendarStore::new()?;
        let calendars = store.calendars()?;

        Ok(serde_json::json!({
            "calendars": calendars,
            "count": calendars.len(),
        }))
    }

    /// Get today's events.
    fn today(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let store = CalendarStore::new()?;
        let events = store.today()?;

        Ok(serde_json::json!({
            "events": events,
            "count": events.len(),
            "date": Local::now().format("%Y-%m-%d").to_string(),
        }))
    }

    /// Get upcoming events.
    fn upcoming(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 7);
        let limit = Self::get_param_u32(&params, "limit", 50) as usize;

        let store = CalendarStore::new()?;
        let mut events = store.upcoming(days)?;
        events.truncate(limit);

        Ok(serde_json::json!({
            "events": events,
            "count": events.len(),
            "days": days,
        }))
    }

    /// Get events in a date range.
    fn events(&self, params: HashMap<String, Value>) -> Result<Value> {
        let start_str = Self::get_param_str(&params, "start")
            .ok_or_else(|| anyhow!("Missing required parameter: start"))?;
        let end_str = Self::get_param_str(&params, "end")
            .ok_or_else(|| anyhow!("Missing required parameter: end"))?;

        // Parse dates (YYYY-MM-DD format)
        let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .map_err(|e| anyhow!("Invalid start date format (use YYYY-MM-DD): {}", e))?;
        let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .map_err(|e| anyhow!("Invalid end date format (use YYYY-MM-DD): {}", e))?;

        let start = Local
            .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let end = Local
            .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap())
            .unwrap();

        // Get calendar IDs if specified
        let calendar_ids: Option<Vec<String>> = params.get("calendars").and_then(|v| {
            if let Some(arr) = v.as_array() {
                Some(
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                )
            } else if let Some(s) = v.as_str() {
                Some(s.split(',').map(|s| s.trim().to_string()).collect())
            } else {
                None
            }
        });

        let store = CalendarStore::new()?;
        let events = store.events(start, end, calendar_ids)?;

        Ok(serde_json::json!({
            "events": events,
            "count": events.len(),
            "start": start_str,
            "end": end_str,
        }))
    }

    /// Search events by title/location.
    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_param_str(&params, "query")
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;
        let days = Self::get_param_u32(&params, "days", 30);
        let limit = Self::get_param_u32(&params, "limit", 20) as usize;

        let store = CalendarStore::new()?;
        let mut events = store.search(query, days)?;
        events.truncate(limit);

        Ok(serde_json::json!({
            "events": events,
            "count": events.len(),
            "query": query,
            "days": days,
        }))
    }

    /// Get events for a specific date.
    fn on_date(&self, params: HashMap<String, Value>) -> Result<Value> {
        let date_str = Self::get_param_str(&params, "date")
            .ok_or_else(|| anyhow!("Missing required parameter: date"))?;

        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| anyhow!("Invalid date format (use YYYY-MM-DD): {}", e))?;

        let store = CalendarStore::new()?;
        let events = store.events_on_date(date)?;

        Ok(serde_json::json!({
            "events": events,
            "count": events.len(),
            "date": date_str,
        }))
    }

    /// Get authorization status.
    fn auth_status(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let status = CalendarStore::authorization_status();
        let status_str = format!("{:?}", status);

        let authorized =
            matches!(status, objc2_event_kit::EKAuthorizationStatus::FullAccess);

        Ok(serde_json::json!({
            "status": status_str,
            "authorized": authorized,
            "message": if authorized {
                "Calendar access granted"
            } else {
                "Please grant calendar access in System Settings > Privacy > Calendars"
            },
        }))
    }
}

impl Default for CalendarService {
    fn default() -> Self {
        Self::new().expect("Failed to create CalendarService")
    }
}

impl FgpService for CalendarService {
    fn name(&self) -> &str {
        "apple-calendar"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "apple-calendar.calendars" | "calendars" => self.calendars(params),
            "apple-calendar.today" | "today" => self.today(params),
            "apple-calendar.upcoming" | "upcoming" => self.upcoming(params),
            "apple-calendar.events" | "events" => self.events(params),
            "apple-calendar.search" | "search" => self.search(params),
            "apple-calendar.on_date" | "on_date" => self.on_date(params),
            "apple-calendar.auth" | "auth" => self.auth_status(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("calendars", "List all calendars"),
            MethodInfo::new("today", "Get today's events"),
            MethodInfo::new("upcoming", "Get upcoming events")
                .param(ParamInfo {
                    name: "days".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(7.into())),
                })
                .param(ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(50.into())),
                }),
            MethodInfo::new("events", "Get events in a date range")
                .param(ParamInfo {
                    name: "start".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "end".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "calendars".into(),
                    param_type: "array".into(),
                    required: false,
                    default: None,
                }),
            MethodInfo::new("search", "Search events by title/location")
                .param(ParamInfo {
                    name: "query".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "days".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(30.into())),
                })
                .param(ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(20.into())),
                }),
            MethodInfo::new("on_date", "Get events for a specific date").param(ParamInfo {
                name: "date".into(),
                param_type: "string".into(),
                required: true,
                default: None,
            }),
            MethodInfo::new("auth", "Check authorization status"),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Authorization check
        let status = CalendarStore::authorization_status();
        let authorized =
            matches!(status, objc2_event_kit::EKAuthorizationStatus::FullAccess);

        checks.insert(
            "authorization".into(),
            HealthStatus {
                ok: authorized,
                latency_ms: None,
                message: Some(format!("{:?}", status)),
            },
        );

        // Calendar access check
        if let Ok(store) = CalendarStore::new() {
            let calendar_check = store.calendars();
            checks.insert(
                "calendars".into(),
                HealthStatus {
                    ok: calendar_check.is_ok(),
                    latency_ms: None,
                    message: calendar_check
                        .map(|c| format!("{} calendars available", c.len()))
                        .ok(),
                },
            );
        }

        checks
    }

    fn on_start(&self) -> Result<()> {
        let status = CalendarStore::authorization_status();
        tracing::info!("Authorization status: {:?}", status);

        match status {
            objc2_event_kit::EKAuthorizationStatus::FullAccess => {
                if let Ok(store) = CalendarStore::new() {
                    if let Ok(calendars) = store.calendars() {
                        tracing::info!("Found {} calendars", calendars.len());
                    }
                }
            }
            _ => {
                tracing::warn!(
                    "Calendar access not granted. Please enable in System Settings > Privacy > Calendars"
                );
            }
        }

        Ok(())
    }
}
