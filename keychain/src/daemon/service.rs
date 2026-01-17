//! Keychain daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;

use crate::keychain::KeychainStore;

/// Keychain service.
pub struct KeychainService;

impl KeychainService {
    /// Create a new keychain service.
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Helper to get a required string parameter.
    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Result<&'a str> {
        params
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: {}", key))
    }

    /// Helper to get an optional string parameter.
    fn get_param_str_opt<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Find a generic password.
    fn find_generic(&self, params: HashMap<String, Value>) -> Result<Value> {
        let service = Self::get_param_str(&params, "service")?;
        let account = Self::get_param_str(&params, "account")?;

        let store = KeychainStore::new()?;
        let password = store.find_generic(service, account)?;

        Ok(serde_json::json!({
            "found": true,
            "service": service,
            "account": account,
            "password": password,
        }))
    }

    /// Add or update a generic password.
    fn set_generic(&self, params: HashMap<String, Value>) -> Result<Value> {
        let service = Self::get_param_str(&params, "service")?;
        let account = Self::get_param_str(&params, "account")?;
        let password = Self::get_param_str(&params, "password")?;

        let store = KeychainStore::new()?;
        store.set_generic(service, account, password)?;

        Ok(serde_json::json!({
            "success": true,
            "service": service,
            "account": account,
            "message": "Password stored successfully",
        }))
    }

    /// Delete a generic password.
    fn delete_generic(&self, params: HashMap<String, Value>) -> Result<Value> {
        let service = Self::get_param_str(&params, "service")?;
        let account = Self::get_param_str(&params, "account")?;

        let store = KeychainStore::new()?;
        store.delete_generic(service, account)?;

        Ok(serde_json::json!({
            "success": true,
            "service": service,
            "account": account,
            "message": "Password deleted successfully",
        }))
    }

    /// Check if a password exists.
    fn exists(&self, params: HashMap<String, Value>) -> Result<Value> {
        let service = Self::get_param_str(&params, "service")?;
        let account = Self::get_param_str(&params, "account")?;

        let store = KeychainStore::new()?;
        let exists = store.exists_generic(service, account);

        Ok(serde_json::json!({
            "exists": exists,
            "service": service,
            "account": account,
        }))
    }

    /// Get authorization/access status.
    fn auth_status(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let accessible = KeychainStore::check_access();

        Ok(serde_json::json!({
            "accessible": accessible,
            "message": if accessible {
                "Keychain access available. Note: First access to specific passwords may require user approval."
            } else {
                "Keychain access not available. Ensure the binary is code-signed."
            },
            "note": "Code signing required: codesign -s - ./fgp-keychain-daemon",
        }))
    }
}

impl Default for KeychainService {
    fn default() -> Self {
        Self::new().expect("Failed to create KeychainService")
    }
}

impl FgpService for KeychainService {
    fn name(&self) -> &str {
        "keychain"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "keychain.find_generic" | "find_generic" => self.find_generic(params),
            "keychain.set_generic" | "set_generic" => self.set_generic(params),
            "keychain.delete_generic" | "delete_generic" | "delete" => self.delete_generic(params),
            "keychain.exists" | "exists" => self.exists(params),
            "keychain.auth" | "auth" => self.auth_status(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("find_generic", "Find a generic password by service and account")
                .param(ParamInfo {
                    name: "service".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "account".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }),
            MethodInfo::new("set_generic", "Add or update a generic password")
                .param(ParamInfo {
                    name: "service".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "account".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "password".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }),
            MethodInfo::new("delete", "Delete a generic password")
                .param(ParamInfo {
                    name: "service".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "account".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }),
            MethodInfo::new("exists", "Check if a password exists")
                .param(ParamInfo {
                    name: "service".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                })
                .param(ParamInfo {
                    name: "account".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }),
            MethodInfo::new("auth", "Check keychain access status"),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let accessible = KeychainStore::check_access();

        checks.insert(
            "access".into(),
            HealthStatus {
                ok: accessible,
                latency_ms: None,
                message: Some(if accessible {
                    "Keychain accessible".to_string()
                } else {
                    "Keychain not accessible - check code signing".to_string()
                }),
            },
        );

        checks
    }

    fn on_start(&self) -> Result<()> {
        let accessible = KeychainStore::check_access();
        if accessible {
            tracing::info!("Keychain access available");
        } else {
            tracing::warn!(
                "Keychain access may not be available. Ensure binary is code-signed: codesign -s - ./fgp-keychain-daemon"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::KeychainService;
    use fgp_daemon::FgpService;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn get_param_str_reads_required_values() {
        let mut params = HashMap::new();
        params.insert("service".to_string(), json!("login"));

        let value = KeychainService::get_param_str(&params, "service").expect("service");
        assert_eq!(value, "login");
    }

    #[test]
    fn get_param_str_errors_on_missing_or_invalid() {
        let params = HashMap::new();
        let err = KeychainService::get_param_str(&params, "service").expect_err("missing");
        assert!(err.to_string().contains("service"));

        let mut params = HashMap::new();
        params.insert("service".to_string(), json!(42));
        let err = KeychainService::get_param_str(&params, "service").expect_err("invalid");
        assert!(err.to_string().contains("service"));
    }

    #[test]
    fn get_param_str_opt_handles_optional_values() {
        let mut params = HashMap::new();
        params.insert("account".to_string(), json!("me"));

        assert_eq!(KeychainService::get_param_str_opt(&params, "account"), Some("me"));
        assert_eq!(KeychainService::get_param_str_opt(&params, "missing"), None);
    }

    #[test]
    fn method_list_includes_required_fields() {
        let service = KeychainService::new().expect("service");
        let methods = service.method_list();

        let set_method = methods
            .iter()
            .find(|m| m.name == "set_generic")
            .expect("set_generic");
        let service_param = set_method
            .params
            .iter()
            .find(|p| p.name == "service")
            .expect("service");
        let account_param = set_method
            .params
            .iter()
            .find(|p| p.name == "account")
            .expect("account");
        let password_param = set_method
            .params
            .iter()
            .find(|p| p.name == "password")
            .expect("password");
        assert!(service_param.required);
        assert!(account_param.required);
        assert!(password_param.required);
    }

    #[test]
    fn dispatch_rejects_unknown_method() {
        let service = KeychainService::new().expect("service");
        let err = service
            .dispatch("keychain.nope", HashMap::new())
            .expect_err("unknown method");
        assert!(err.to_string().contains("Unknown method"));
    }
}
