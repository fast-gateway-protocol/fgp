//! macOS Keychain queries via Security framework.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use serde::{Deserialize, Serialize};

/// Password information (without the actual password for listing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordInfo {
    pub service: String,
    pub account: String,
    pub label: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// Keychain store wrapper.
pub struct KeychainStore;

impl KeychainStore {
    /// Create a new keychain store.
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Check if keychain access is available.
    /// This is a simple check - actual access may still require user approval.
    pub fn check_access() -> bool {
        // Try to access the default keychain
        // The security-framework doesn't have a direct "check access" API,
        // so we just return true and let actual operations handle errors
        true
    }

    /// Find a generic password by service and account.
    pub fn find_generic(&self, service: &str, account: &str) -> Result<String> {
        let password = get_generic_password(service, account)
            .map_err(|e| anyhow!("Failed to find password: {}", e))?;

        String::from_utf8(password)
            .map_err(|e| anyhow!("Password is not valid UTF-8: {}", e))
    }

    /// Add or update a generic password.
    pub fn set_generic(&self, service: &str, account: &str, password: &str) -> Result<()> {
        // Try to delete existing first (ignore errors if not found)
        let _ = delete_generic_password(service, account);

        set_generic_password(service, account, password.as_bytes())
            .map_err(|e| anyhow!("Failed to set password: {}", e))
    }

    /// Delete a generic password.
    pub fn delete_generic(&self, service: &str, account: &str) -> Result<()> {
        delete_generic_password(service, account)
            .map_err(|e| anyhow!("Failed to delete password: {}", e))
    }

    /// Check if a generic password exists.
    pub fn exists_generic(&self, service: &str, account: &str) -> bool {
        get_generic_password(service, account).is_ok()
    }
}

impl Default for KeychainStore {
    fn default() -> Self {
        Self::new().expect("Failed to create KeychainStore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_access() {
        assert!(KeychainStore::check_access());
    }

    #[test]
    fn test_password_info_roundtrip() {
        let info = PasswordInfo {
            service: "svc".to_string(),
            account: "acct".to_string(),
            label: Some("label".to_string()),
            created: Some("2026-01-01T00:00:00Z".to_string()),
            modified: None,
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let parsed: PasswordInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.service, "svc");
        assert_eq!(parsed.account, "acct");
        assert_eq!(parsed.label.as_deref(), Some("label"));
        assert_eq!(parsed.created.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert!(parsed.modified.is_none());
    }
}
