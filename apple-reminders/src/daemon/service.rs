//! Apple Reminders daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDate};
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;

use crate::reminder::ReminderStore;

/// Apple Reminders service.
/// Note: ReminderStore is not stored because EKEventStore is not Send/Sync.
/// We create a fresh ReminderStore for each request.
pub struct RemindersService;

impl RemindersService {
    /// Create a new reminders service.
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

    /// Helper to get a boolean parameter with default.
    fn get_param_bool(params: &HashMap<String, Value>, key: &str, default: bool) -> bool {
        params
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Helper to get list IDs from params.
    fn get_list_ids(params: &HashMap<String, Value>) -> Option<Vec<String>> {
        params.get("lists").and_then(|v| {
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
        })
    }

    /// List all reminder lists.
    fn lists(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let store = ReminderStore::new()?;
        let lists = store.lists()?;

        Ok(serde_json::json!({
            "lists": lists,
            "count": lists.len(),
        }))
    }

    /// Get all reminders.
    fn all(&self, params: HashMap<String, Value>) -> Result<Value> {
        let list_ids = Self::get_list_ids(&params);
        let limit = Self::get_param_u32(&params, "limit", 100) as usize;

        let store = ReminderStore::new()?;
        let mut reminders = store.all_reminders(list_ids)?;
        reminders.truncate(limit);

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
        }))
    }

    /// Get incomplete reminders.
    fn incomplete(&self, params: HashMap<String, Value>) -> Result<Value> {
        let list_ids = Self::get_list_ids(&params);
        let limit = Self::get_param_u32(&params, "limit", 100) as usize;

        let store = ReminderStore::new()?;
        let mut reminders = store.incomplete_reminders(list_ids, None, None)?;
        reminders.truncate(limit);

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
        }))
    }

    /// Get completed reminders.
    fn completed(&self, params: HashMap<String, Value>) -> Result<Value> {
        let list_ids = Self::get_list_ids(&params);
        let limit = Self::get_param_u32(&params, "limit", 50) as usize;
        let days = Self::get_param_u32(&params, "days", 30);

        let now = Local::now();
        let start = now - chrono::Duration::days(days as i64);

        let store = ReminderStore::new()?;
        let mut reminders = store.completed_reminders(list_ids, Some(start), Some(now))?;
        reminders.truncate(limit);

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
            "days": days,
        }))
    }

    /// Get reminders due today.
    fn due_today(&self, params: HashMap<String, Value>) -> Result<Value> {
        let list_ids = Self::get_list_ids(&params);

        let store = ReminderStore::new()?;
        let reminders = store.due_today(list_ids)?;

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
            "date": Local::now().format("%Y-%m-%d").to_string(),
        }))
    }

    /// Get overdue reminders.
    fn overdue(&self, params: HashMap<String, Value>) -> Result<Value> {
        let list_ids = Self::get_list_ids(&params);

        let store = ReminderStore::new()?;
        let reminders = store.overdue(list_ids)?;

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
        }))
    }

    /// Get upcoming reminders.
    fn upcoming(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 7);
        let list_ids = Self::get_list_ids(&params);
        let limit = Self::get_param_u32(&params, "limit", 50) as usize;

        let store = ReminderStore::new()?;
        let mut reminders = store.upcoming(days, list_ids)?;
        reminders.truncate(limit);

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
            "days": days,
        }))
    }

    /// Search reminders.
    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_param_str(&params, "query")
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;
        let include_completed = Self::get_param_bool(&params, "include_completed", false);
        let list_ids = Self::get_list_ids(&params);
        let limit = Self::get_param_u32(&params, "limit", 50) as usize;

        let store = ReminderStore::new()?;
        let mut reminders = store.search(query, include_completed, list_ids)?;
        reminders.truncate(limit);

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
            "query": query,
            "include_completed": include_completed,
        }))
    }

    /// Get reminders for a specific date.
    fn on_date(&self, params: HashMap<String, Value>) -> Result<Value> {
        let date_str = Self::get_param_str(&params, "date")
            .ok_or_else(|| anyhow!("Missing required parameter: date"))?;

        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| anyhow!("Invalid date format (use YYYY-MM-DD): {}", e))?;

        let list_ids = Self::get_list_ids(&params);

        let store = ReminderStore::new()?;
        let reminders = store.on_date(date, list_ids)?;

        Ok(serde_json::json!({
            "reminders": reminders,
            "count": reminders.len(),
            "date": date_str,
        }))
    }

    /// Get authorization status.
    fn auth_status(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let status = ReminderStore::authorization_status();
        let status_str = format!("{:?}", status);

        let authorized =
            matches!(status, objc2_event_kit::EKAuthorizationStatus::FullAccess);

        Ok(serde_json::json!({
            "status": status_str,
            "authorized": authorized,
            "message": if authorized {
                "Reminders access granted"
            } else {
                "Please grant reminders access in System Settings > Privacy > Reminders"
            },
        }))
    }
}

impl Default for RemindersService {
    fn default() -> Self {
        Self::new().expect("Failed to create RemindersService")
    }
}

impl FgpService for RemindersService {
    fn name(&self) -> &str {
        "apple-reminders"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "apple-reminders.lists" | "lists" => self.lists(params),
            "apple-reminders.all" | "all" => self.all(params),
            "apple-reminders.incomplete" | "incomplete" => self.incomplete(params),
            "apple-reminders.completed" | "completed" => self.completed(params),
            "apple-reminders.due_today" | "due_today" => self.due_today(params),
            "apple-reminders.overdue" | "overdue" => self.overdue(params),
            "apple-reminders.upcoming" | "upcoming" => self.upcoming(params),
            "apple-reminders.search" | "search" => self.search(params),
            "apple-reminders.on_date" | "on_date" => self.on_date(params),
            "apple-reminders.auth" | "auth" => self.auth_status(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("lists", "List all reminder lists"),
            MethodInfo::new("all", "Get all reminders")
                .param(ParamInfo {
                    name: "lists".into(),
                    param_type: "array".into(),
                    required: false,
                    default: None,
                })
                .param(ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(100.into())),
                }),
            MethodInfo::new("incomplete", "Get incomplete reminders")
                .param(ParamInfo {
                    name: "lists".into(),
                    param_type: "array".into(),
                    required: false,
                    default: None,
                })
                .param(ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(100.into())),
                }),
            MethodInfo::new("completed", "Get completed reminders")
                .param(ParamInfo {
                    name: "lists".into(),
                    param_type: "array".into(),
                    required: false,
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
                    default: Some(Value::Number(50.into())),
                }),
            MethodInfo::new("due_today", "Get reminders due today").param(ParamInfo {
                name: "lists".into(),
                param_type: "array".into(),
                required: false,
                default: None,
            }),
            MethodInfo::new("overdue", "Get overdue reminders").param(ParamInfo {
                name: "lists".into(),
                param_type: "array".into(),
                required: false,
                default: None,
            }),
            MethodInfo::new("upcoming", "Get upcoming reminders")
                .param(ParamInfo {
                    name: "days".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(7.into())),
                })
                .param(ParamInfo {
                    name: "lists".into(),
                    param_type: "array".into(),
                    required: false,
                    default: None,
                })
                .param(ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(50.into())),
                }),
            MethodInfo::new("search", "Search reminders by title/notes")
                .param(ParamInfo {
                    name: "query".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "include_completed".into(),
                    param_type: "boolean".into(),
                    required: false,
                    default: Some(Value::Bool(false)),
                })
                .param(ParamInfo {
                    name: "lists".into(),
                    param_type: "array".into(),
                    required: false,
                    default: None,
                })
                .param(ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(50.into())),
                }),
            MethodInfo::new("on_date", "Get reminders due on a specific date")
                .param(ParamInfo {
                    name: "date".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "lists".into(),
                    param_type: "array".into(),
                    required: false,
                    default: None,
                }),
            MethodInfo::new("auth", "Check authorization status"),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Authorization check
        let status = ReminderStore::authorization_status();
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

        // Lists access check
        if let Ok(store) = ReminderStore::new() {
            let lists_check = store.lists();
            checks.insert(
                "lists".into(),
                HealthStatus {
                    ok: lists_check.is_ok(),
                    latency_ms: None,
                    message: lists_check
                        .map(|l| format!("{} lists available", l.len()))
                        .ok(),
                },
            );
        }

        checks
    }

    fn on_start(&self) -> Result<()> {
        let status = ReminderStore::authorization_status();
        tracing::info!("Authorization status: {:?}", status);

        match status {
            objc2_event_kit::EKAuthorizationStatus::FullAccess => {
                if let Ok(store) = ReminderStore::new() {
                    if let Ok(lists) = store.lists() {
                        tracing::info!("Found {} reminder lists", lists.len());
                    }
                }
            }
            _ => {
                tracing::warn!(
                    "Reminders access not granted. Please enable in System Settings > Privacy > Reminders"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RemindersService;
    use fgp_daemon::service::FgpService;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn get_param_helpers_read_values() {
        let mut params = HashMap::new();
        params.insert("days".to_string(), json!(5));
        params.insert("query".to_string(), json!("walk"));
        params.insert("include_completed".to_string(), json!(true));

        assert_eq!(RemindersService::get_param_u32(&params, "days", 7), 5);
        assert_eq!(RemindersService::get_param_u32(&params, "missing", 7), 7);
        assert_eq!(RemindersService::get_param_str(&params, "query"), Some("walk"));
        assert_eq!(RemindersService::get_param_str(&params, "missing"), None);
        assert!(RemindersService::get_param_bool(&params, "include_completed", false));
        assert!(!RemindersService::get_param_bool(&params, "missing", false));
    }

    #[test]
    fn get_list_ids_parses_arrays_and_csv() {
        let mut params = HashMap::new();
        params.insert("lists".to_string(), json!(["one", "two"]));
        assert_eq!(
            RemindersService::get_list_ids(&params),
            Some(vec!["one".to_string(), "two".to_string()])
        );

        let mut params = HashMap::new();
        params.insert("lists".to_string(), json!("one, two"));
        assert_eq!(
            RemindersService::get_list_ids(&params),
            Some(vec!["one".to_string(), "two".to_string()])
        );

        let mut params = HashMap::new();
        params.insert("lists".to_string(), json!(5));
        assert_eq!(RemindersService::get_list_ids(&params), None);
    }

    #[test]
    fn method_list_includes_defaults_and_required_fields() {
        let methods = RemindersService::new().expect("service").method_list();

        let all_method = methods.iter().find(|m| m.name == "all").expect("all");
        let limit_param = all_method
            .params
            .iter()
            .find(|p| p.name == "limit")
            .expect("limit");
        assert_eq!(limit_param.default, Some(json!(100)));

        let completed_method = methods
            .iter()
            .find(|m| m.name == "completed")
            .expect("completed");
        let days_param = completed_method
            .params
            .iter()
            .find(|p| p.name == "days")
            .expect("days");
        let completed_limit = completed_method
            .params
            .iter()
            .find(|p| p.name == "limit")
            .expect("limit");
        assert_eq!(days_param.default, Some(json!(30)));
        assert_eq!(completed_limit.default, Some(json!(50)));

        let search_method = methods.iter().find(|m| m.name == "search").expect("search");
        let query_param = search_method
            .params
            .iter()
            .find(|p| p.name == "query")
            .expect("query");
        let include_param = search_method
            .params
            .iter()
            .find(|p| p.name == "include_completed")
            .expect("include_completed");
        assert!(query_param.required);
        assert_eq!(include_param.default, Some(json!(false)));
    }

    #[test]
    fn dispatch_rejects_unknown_method() {
        let service = RemindersService::new().expect("service");
        let err = service
            .dispatch("apple-reminders.nope", HashMap::new())
            .expect_err("unknown method");
        assert!(err.to_string().contains("Unknown method"));
    }
}
