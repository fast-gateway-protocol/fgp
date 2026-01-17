//! Spotlight FGP daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use fgp_daemon::service::{FgpService, HealthStatus, MethodInfo, ParamInfo};
use serde_json::Value;
use std::collections::HashMap;

use crate::spotlight::queries;

/// Spotlight daemon service.
pub struct SpotlightService;

impl SpotlightService {
    /// Create new Spotlight service.
    pub fn new() -> Result<Self> {
        Ok(Self)
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

    fn get_param_u64(params: &HashMap<String, Value>, key: &str) -> Option<u64> {
        params.get(key).and_then(|v| v.as_u64())
    }

    // ========================================================================
    // Handlers
    // ========================================================================

    /// Raw Spotlight query.
    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_param_str(&params, "query")
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;
        let scope = Self::get_param_str(&params, "scope");
        let limit = Self::get_param_u32(&params, "limit", 50);

        let results = queries::search_raw(query, scope, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
            "query": query,
        }))
    }

    /// Search by name.
    fn by_name(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_param_str(&params, "name")
            .ok_or_else(|| anyhow!("Missing required parameter: name"))?;
        let scope = Self::get_param_str(&params, "scope");
        let limit = Self::get_param_u32(&params, "limit", 50);

        let results = queries::search_by_name(name, scope, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
            "name": name,
        }))
    }

    /// Search by extension.
    fn by_extension(&self, params: HashMap<String, Value>) -> Result<Value> {
        let ext = Self::get_param_str(&params, "extension")
            .ok_or_else(|| anyhow!("Missing required parameter: extension"))?;
        let scope = Self::get_param_str(&params, "scope");
        let limit = Self::get_param_u32(&params, "limit", 50);

        let results = queries::search_by_extension(ext, scope, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
            "extension": ext,
        }))
    }

    /// Search by kind.
    fn by_kind(&self, params: HashMap<String, Value>) -> Result<Value> {
        let kind = Self::get_param_str(&params, "kind")
            .ok_or_else(|| anyhow!("Missing required parameter: kind"))?;
        let name = Self::get_param_str(&params, "name");
        let scope = Self::get_param_str(&params, "scope");
        let limit = Self::get_param_u32(&params, "limit", 50);

        let results = queries::search_by_kind(kind, name, scope, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
            "kind": kind,
        }))
    }

    /// Search recent files.
    fn recent(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 7);
        let scope = Self::get_param_str(&params, "scope");
        let limit = Self::get_param_u32(&params, "limit", 50);

        let results = queries::search_recent(days, scope, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
            "days": days,
        }))
    }

    /// Search applications.
    fn apps(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_param_str(&params, "name");
        let limit = Self::get_param_u32(&params, "limit", 50);

        let results = queries::search_apps(name, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
        }))
    }

    /// Search directories.
    fn directories(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_param_str(&params, "name");
        let scope = Self::get_param_str(&params, "scope");
        let limit = Self::get_param_u32(&params, "limit", 50);

        let results = queries::search_directories(name, scope, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
        }))
    }

    /// Search by size.
    fn by_size(&self, params: HashMap<String, Value>) -> Result<Value> {
        let min_bytes = Self::get_param_u64(&params, "min_bytes");
        let max_bytes = Self::get_param_u64(&params, "max_bytes");
        let scope = Self::get_param_str(&params, "scope");
        let limit = Self::get_param_u32(&params, "limit", 50);

        if min_bytes.is_none() && max_bytes.is_none() {
            return Err(anyhow!(
                "At least one of min_bytes or max_bytes is required"
            ));
        }

        let results = queries::search_by_size(min_bytes, max_bytes, scope, limit)?;

        Ok(serde_json::json!({
            "results": results,
            "count": results.len(),
        }))
    }
}

impl FgpService for SpotlightService {
    fn name(&self) -> &str {
        "spotlight"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "spotlight.search" | "search" => self.search(params),
            "spotlight.by_name" | "by_name" => self.by_name(params),
            "spotlight.by_extension" | "by_extension" => self.by_extension(params),
            "spotlight.by_kind" | "by_kind" => self.by_kind(params),
            "spotlight.recent" | "recent" => self.recent(params),
            "spotlight.apps" | "apps" => self.apps(params),
            "spotlight.directories" | "directories" => self.directories(params),
            "spotlight.by_size" | "by_size" => self.by_size(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("search", "Raw Spotlight query (mdfind syntax)")
                .param(ParamInfo { name: "query".into(), param_type: "string".into(), required: true, default: None })
                .param(ParamInfo { name: "scope".into(), param_type: "string".into(), required: false, default: Some(Value::String("home".into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("by_name", "Search files by name (substring match)")
                .param(ParamInfo { name: "name".into(), param_type: "string".into(), required: true, default: None })
                .param(ParamInfo { name: "scope".into(), param_type: "string".into(), required: false, default: Some(Value::String("home".into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("by_extension", "Search files by extension")
                .param(ParamInfo { name: "extension".into(), param_type: "string".into(), required: true, default: None })
                .param(ParamInfo { name: "scope".into(), param_type: "string".into(), required: false, default: Some(Value::String("home".into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("by_kind", "Search by kind: pdf, image, video, audio, document, text, source, folder, app, archive")
                .param(ParamInfo { name: "kind".into(), param_type: "string".into(), required: true, default: None })
                .param(ParamInfo { name: "name".into(), param_type: "string".into(), required: false, default: None })
                .param(ParamInfo { name: "scope".into(), param_type: "string".into(), required: false, default: Some(Value::String("home".into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("recent", "Find recently modified files")
                .param(ParamInfo { name: "days".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(7.into())) })
                .param(ParamInfo { name: "scope".into(), param_type: "string".into(), required: false, default: Some(Value::String("home".into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("apps", "Find applications")
                .param(ParamInfo { name: "name".into(), param_type: "string".into(), required: false, default: None })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("directories", "Find directories")
                .param(ParamInfo { name: "name".into(), param_type: "string".into(), required: false, default: None })
                .param(ParamInfo { name: "scope".into(), param_type: "string".into(), required: false, default: Some(Value::String("home".into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("by_size", "Find files by size range (requires min_bytes and/or max_bytes)")
                .param(ParamInfo { name: "min_bytes".into(), param_type: "integer".into(), required: false, default: None })
                .param(ParamInfo { name: "max_bytes".into(), param_type: "integer".into(), required: false, default: None })
                .param(ParamInfo { name: "scope".into(), param_type: "string".into(), required: false, default: Some(Value::String("home".into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        // Test a simple query
        let result = queries::search_apps(None, 1);
        let (ok, msg) = match result {
            Ok(_) => (true, "Spotlight index accessible".to_string()),
            Err(e) => (false, format!("Spotlight error: {}", e)),
        };

        checks.insert(
            "spotlight".into(),
            HealthStatus {
                ok,
                latency_ms: None,
                message: Some(msg),
            },
        );

        checks
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("Spotlight daemon starting");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), Value::from(25));
        params.insert("query".to_string(), Value::String("name:report".to_string()));
        params.insert("min_bytes".to_string(), Value::from(1024));

        assert_eq!(SpotlightService::get_param_u32(&params, "limit", 50), 25);
        assert_eq!(SpotlightService::get_param_u32(&params, "missing", 50), 50);
        assert_eq!(
            SpotlightService::get_param_str(&params, "query"),
            Some("name:report")
        );
        assert_eq!(SpotlightService::get_param_str(&params, "missing"), None);
        assert_eq!(SpotlightService::get_param_u64(&params, "min_bytes"), Some(1024));
        assert_eq!(SpotlightService::get_param_u64(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_defaults() {
        let service = SpotlightService::new().expect("service");
        let methods = service.method_list();

        let search = methods.iter().find(|m| m.name == "search").expect("search");
        let search_scope = search
            .params
            .iter()
            .find(|p| p.name == "scope")
            .expect("scope param");
        assert_eq!(
            search_scope.default.as_ref().and_then(Value::as_str),
            Some("home")
        );

        let recent = methods.iter().find(|m| m.name == "recent").expect("recent");
        let recent_days = recent
            .params
            .iter()
            .find(|p| p.name == "days")
            .expect("days param");
        assert_eq!(
            recent_days.default.as_ref().and_then(Value::as_i64),
            Some(7)
        );
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = SpotlightService::new().expect("service");
        let result = service.dispatch("spotlight.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
