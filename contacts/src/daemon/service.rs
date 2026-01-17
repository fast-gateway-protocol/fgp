//! Contacts FGP daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use fgp_daemon::service::{FgpService, HealthStatus, MethodInfo, ParamInfo};
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::connection::open_addressbook_db;
use crate::db::queries;

/// Contacts daemon service with hot database connection.
pub struct ContactsService {
    conn: Mutex<Connection>,
    started_at: String,
}

impl ContactsService {
    /// Create new Contacts service with hot connection.
    pub fn new() -> Result<Self> {
        let conn = Mutex::new(open_addressbook_db()?);
        let started_at = chrono::Utc::now().to_rfc3339();

        Ok(Self { conn, started_at })
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

    /// List all contacts.
    /// Params: limit (default 100)
    fn list(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 100);

        let contacts = queries::query_contacts_list(&self.conn.lock().unwrap(), limit)?;

        let contacts_json: Vec<Value> = contacts
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.display_name(),
                    "first_name": c.first_name,
                    "last_name": c.last_name,
                    "organization": c.organization,
                    "job_title": c.job_title,
                    "emails": c.emails,
                    "phones": c.phones,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "contacts": contacts_json,
            "count": contacts_json.len(),
        }))
    }

    /// Search contacts by name.
    /// Params: query (required), limit (default 20)
    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_param_str(&params, "query")
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;
        let limit = Self::get_param_u32(&params, "limit", 20);

        let contacts = queries::query_search_contacts(&self.conn.lock().unwrap(), query, limit)?;

        let contacts_json: Vec<Value> = contacts
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.display_name(),
                    "first_name": c.first_name,
                    "last_name": c.last_name,
                    "organization": c.organization,
                    "emails": c.emails,
                    "phones": c.phones,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "contacts": contacts_json,
            "count": contacts_json.len(),
            "query": query,
        }))
    }

    /// Find contact by email address.
    /// Params: email (required)
    fn by_email(&self, params: HashMap<String, Value>) -> Result<Value> {
        let email = Self::get_param_str(&params, "email")
            .ok_or_else(|| anyhow!("Missing required parameter: email"))?;

        let contact = queries::query_contact_by_email(&self.conn.lock().unwrap(), email)?;

        match contact {
            Some(c) => Ok(serde_json::json!({
                "found": true,
                "contact": {
                    "id": c.id,
                    "name": c.display_name(),
                    "first_name": c.first_name,
                    "last_name": c.last_name,
                    "organization": c.organization,
                    "job_title": c.job_title,
                    "emails": c.emails,
                    "phones": c.phones,
                },
            })),
            None => Ok(serde_json::json!({
                "found": false,
                "email": email,
            })),
        }
    }

    /// Find contact by phone number.
    /// Params: phone (required)
    fn by_phone(&self, params: HashMap<String, Value>) -> Result<Value> {
        let phone = Self::get_param_str(&params, "phone")
            .ok_or_else(|| anyhow!("Missing required parameter: phone"))?;

        let contact = queries::query_contact_by_phone(&self.conn.lock().unwrap(), phone)?;

        match contact {
            Some(c) => Ok(serde_json::json!({
                "found": true,
                "contact": {
                    "id": c.id,
                    "name": c.display_name(),
                    "first_name": c.first_name,
                    "last_name": c.last_name,
                    "organization": c.organization,
                    "job_title": c.job_title,
                    "emails": c.emails,
                    "phones": c.phones,
                },
            })),
            None => Ok(serde_json::json!({
                "found": false,
                "phone": phone,
            })),
        }
    }

    /// Get recently modified contacts.
    /// Params: days (default 30), limit (default 20)
    fn recent(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 30);
        let limit = Self::get_param_u32(&params, "limit", 20);

        let contacts = queries::query_recent_contacts(&self.conn.lock().unwrap(), days, limit)?;

        let contacts_json: Vec<Value> = contacts
            .iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.display_name(),
                    "first_name": c.first_name,
                    "last_name": c.last_name,
                    "organization": c.organization,
                    "emails": c.emails,
                    "phones": c.phones,
                    "modification_date": c.modification_date,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "contacts": contacts_json,
            "count": contacts_json.len(),
            "days": days,
        }))
    }

    /// Get contact statistics.
    fn stats(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let stats = queries::query_contact_stats(&self.conn.lock().unwrap())?;

        Ok(serde_json::json!({
            "total_contacts": stats.total_contacts,
            "with_email": stats.with_email,
            "with_phone": stats.with_phone,
            "with_organization": stats.with_organization,
            "total_groups": stats.total_groups,
        }))
    }
}

impl FgpService for ContactsService {
    fn name(&self) -> &str {
        "contacts"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "contacts.list" | "list" => self.list(params),
            "contacts.search" | "search" => self.search(params),
            "contacts.by_email" | "by_email" => self.by_email(params),
            "contacts.by_phone" | "by_phone" => self.by_phone(params),
            "contacts.recent" | "recent" => self.recent(params),
            "contacts.stats" | "stats" => self.stats(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("list", "List all contacts")
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(100.into())) }),
            MethodInfo::new("search", "Search contacts by name")
                .param(ParamInfo { name: "query".into(), param_type: "string".into(), required: true, default: None })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(20.into())) }),
            MethodInfo::new("by_email", "Find contact by email address")
                .param(ParamInfo { name: "email".into(), param_type: "string".into(), required: true, default: None }),
            MethodInfo::new("by_phone", "Find contact by phone number")
                .param(ParamInfo { name: "phone".into(), param_type: "string".into(), required: true, default: None }),
            MethodInfo::new("recent", "Get recently modified contacts")
                .param(ParamInfo { name: "days".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(30.into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(20.into())) }),
            MethodInfo::new("stats", "Get contact statistics"),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Database check
        let stats = queries::query_contact_stats(&self.conn.lock().unwrap());
        let (ok, msg) = match stats {
            Ok(s) => (true, format!("{} contacts in database", s.total_contacts)),
            Err(e) => (false, format!("Database error: {}", e)),
        };

        checks.insert(
            "database".into(),
            HealthStatus {
                ok,
                latency_ms: None,
                message: Some(msg),
            },
        );

        checks
    }

    fn on_start(&self) -> Result<()> {
        let stats = queries::query_contact_stats(&self.conn.lock().unwrap())?;
        tracing::info!(
            contacts = stats.total_contacts,
            "Contacts daemon starting - database loaded"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ContactsService;
    use fgp_daemon::service::FgpService;
    use rusqlite::Connection;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn test_service() -> ContactsService {
        let conn = Connection::open_in_memory().expect("in memory db");
        ContactsService {
            conn: Mutex::new(conn),
            started_at: "test".to_string(),
        }
    }

    #[test]
    fn get_param_helpers_read_values() {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), json!(42));
        params.insert("query".to_string(), json!("Ada"));

        assert_eq!(ContactsService::get_param_u32(&params, "limit", 10), 42);
        assert_eq!(ContactsService::get_param_u32(&params, "missing", 10), 10);
        assert_eq!(ContactsService::get_param_str(&params, "query"), Some("Ada"));
        assert_eq!(ContactsService::get_param_str(&params, "missing"), None);
    }

    #[test]
    fn method_list_includes_defaults_and_required_fields() {
        let methods = test_service().method_list();

        let list_method = methods.iter().find(|m| m.name == "list").expect("list");
        let list_limit = list_method
            .params
            .iter()
            .find(|p| p.name == "limit")
            .expect("limit");
        assert_eq!(list_limit.required, false);
        assert_eq!(list_limit.default, Some(json!(100)));

        let search_method = methods
            .iter()
            .find(|m| m.name == "search")
            .expect("search");
        let search_query = search_method
            .params
            .iter()
            .find(|p| p.name == "query")
            .expect("query");
        assert!(search_query.required);

        let search_limit = search_method
            .params
            .iter()
            .find(|p| p.name == "limit")
            .expect("limit");
        assert_eq!(search_limit.default, Some(json!(20)));

        let recent_method = methods.iter().find(|m| m.name == "recent").expect("recent");
        let days_param = recent_method
            .params
            .iter()
            .find(|p| p.name == "days")
            .expect("days");
        let limit_param = recent_method
            .params
            .iter()
            .find(|p| p.name == "limit")
            .expect("limit");
        assert_eq!(days_param.default, Some(json!(30)));
        assert_eq!(limit_param.default, Some(json!(20)));
    }

    #[test]
    fn dispatch_rejects_unknown_method() {
        let service = test_service();
        let err = service
            .dispatch("contacts.nope", HashMap::new())
            .expect_err("unknown method");
        assert!(err.to_string().contains("Unknown method"));
    }
}
