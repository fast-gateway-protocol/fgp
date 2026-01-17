//! Data models for Zapier API responses.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A registered webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Webhook registry (persisted to disk).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhookRegistry {
    #[serde(default)]
    pub webhooks: HashMap<String, Webhook>,
}

/// Webhook trigger response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub status: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
}

/// NLA action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlaAction {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// NLA execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlaResult {
    #[serde(default)]
    pub action_used: Option<String>,
    #[serde(default)]
    pub input_params: serde_json::Value,
    #[serde(default)]
    pub result: serde_json::Value,
    #[serde(default)]
    pub error: Option<String>,
}

/// NLA actions list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlaActionsResponse {
    #[serde(default)]
    pub results: Vec<NlaAction>,
}
