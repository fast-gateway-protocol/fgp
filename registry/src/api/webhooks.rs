//! GitHub webhook handlers for auto-sync.
//!
//! Supports:
//! - `push` events - Trigger sync when code is pushed
//! - `release` events - Trigger sync on new releases

use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use std::sync::Arc;

use super::handlers::AppState;
use super::responses::ApiResponse;
use crate::sync::SyncEngine;

type HmacSha256 = Hmac<Sha256>;

/// GitHub webhook payload (common fields across event types)
#[derive(Debug, Deserialize)]
pub struct GitHubWebhookPayload {
    /// The action that triggered the webhook (for release events)
    pub action: Option<String>,

    /// Repository info
    pub repository: GitHubRepository,

    /// Sender info
    pub sender: GitHubSender,

    /// Ref being pushed (for push events)
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,

    /// Commits pushed (for push events)
    pub commits: Option<Vec<GitHubCommit>>,

    /// Release info (for release events)
    pub release: Option<GitHubRelease>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubOwner,
    pub default_branch: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubSender {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubCommit {
    pub id: String,
    pub message: String,
    pub modified: Option<Vec<String>>,
    pub added: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
}

/// Response from webhook processing
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub processed: bool,
    pub message: String,
    pub skills_synced: Option<usize>,
}

/// POST /api/v1/webhooks/github - Handle GitHub webhook events
pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<ApiResponse<WebhookResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Get webhook secret from environment
    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok();

    // Verify signature if secret is configured
    if let Some(ref secret) = webhook_secret {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::error("MISSING_SIGNATURE", "X-Hub-Signature-256 header required")),
                )
            })?;

        if !verify_signature(secret, &body, signature) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("INVALID_SIGNATURE", "Webhook signature verification failed")),
            ));
        }
    }

    // Get event type
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::info!("Received GitHub webhook: {}", event_type);

    // Parse payload
    let payload: GitHubWebhookPayload = serde_json::from_str(&body)
        .map_err(|e| {
            tracing::error!("Failed to parse webhook payload: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("INVALID_PAYLOAD", &format!("Failed to parse payload: {}", e))),
            )
        })?;

    // Process based on event type
    let response = match event_type {
        "push" => process_push_event(&state, &payload).await,
        "release" => process_release_event(&state, &payload).await,
        "ping" => {
            // GitHub sends a ping event when webhook is first configured
            Ok(WebhookResponse {
                processed: true,
                message: "Pong! Webhook configured successfully".to_string(),
                skills_synced: None,
            })
        }
        _ => {
            Ok(WebhookResponse {
                processed: false,
                message: format!("Event type '{}' not processed", event_type),
                skills_synced: None,
            })
        }
    }
    .map_err(|e| {
        tracing::error!("Webhook processing error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("PROCESSING_ERROR", &e.to_string())),
        )
    })?;

    Ok(Json(ApiResponse::ok(response)))
}

/// Process a push event - sync if SKILL.md was modified
async fn process_push_event(
    state: &Arc<AppState>,
    payload: &GitHubWebhookPayload,
) -> anyhow::Result<WebhookResponse> {
    let repo = &payload.repository;
    let owner = &repo.owner.login;
    let repo_name = &repo.name;

    // Check if this is a push to the default branch
    let default_ref = format!("refs/heads/{}", repo.default_branch);
    if payload.git_ref.as_deref() != Some(&default_ref) {
        return Ok(WebhookResponse {
            processed: false,
            message: format!("Push to non-default branch ignored"),
            skills_synced: None,
        });
    }

    // Check if any SKILL.md files were modified
    let skill_modified = payload.commits.as_ref().map_or(false, |commits| {
        commits.iter().any(|c| {
            let files = c.modified.iter().flatten()
                .chain(c.added.iter().flatten());
            files.clone().any(|f| f.ends_with("SKILL.md") || f.ends_with("skill.md"))
        })
    });

    if !skill_modified {
        return Ok(WebhookResponse {
            processed: false,
            message: "No SKILL.md changes detected".to_string(),
            skills_synced: None,
        });
    }

    tracing::info!("SKILL.md modified in {}/{}, triggering sync", owner, repo_name);

    // Trigger sync for this repo
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let engine = SyncEngine::new(state.db.clone(), None, github_token);

    let result = engine.sync_github_repo(owner, repo_name).await?;

    Ok(WebhookResponse {
        processed: true,
        message: format!(
            "Synced {}/{}: {} imported, {} skipped, {} failed",
            owner, repo_name,
            result.imported, result.skipped, result.failed
        ),
        skills_synced: Some(result.imported),
    })
}

/// Process a release event - sync on new releases
async fn process_release_event(
    state: &Arc<AppState>,
    payload: &GitHubWebhookPayload,
) -> anyhow::Result<WebhookResponse> {
    // Only process published releases
    if payload.action.as_deref() != Some("published") {
        return Ok(WebhookResponse {
            processed: false,
            message: "Only 'published' release events are processed".to_string(),
            skills_synced: None,
        });
    }

    let release = payload.release.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing release data"))?;

    // Skip drafts and prereleases
    if release.draft || release.prerelease {
        return Ok(WebhookResponse {
            processed: false,
            message: "Draft and prerelease events ignored".to_string(),
            skills_synced: None,
        });
    }

    let repo = &payload.repository;
    let owner = &repo.owner.login;
    let repo_name = &repo.name;

    tracing::info!(
        "Release {} published in {}/{}, triggering sync",
        release.tag_name, owner, repo_name
    );

    // Trigger sync
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let engine = SyncEngine::new(state.db.clone(), None, github_token);

    let result = engine.sync_github_repo(owner, repo_name).await?;

    Ok(WebhookResponse {
        processed: true,
        message: format!(
            "Release {} synced for {}/{}: {} imported, {} skipped",
            release.tag_name, owner, repo_name,
            result.imported, result.skipped
        ),
        skills_synced: Some(result.imported),
    })
}

/// Verify GitHub webhook signature
fn verify_signature(secret: &str, payload: &str, signature: &str) -> bool {
    // Signature format: "sha256=<hex>"
    let expected_prefix = "sha256=";
    if !signature.starts_with(expected_prefix) {
        return false;
    }

    let provided_sig = &signature[expected_prefix.len()..];

    // Compute HMAC
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload.as_bytes());

    // Compare signatures (constant-time)
    let computed_sig = hex::encode(mac.finalize().into_bytes());

    // Use constant-time comparison to prevent timing attacks
    constant_time_compare(&computed_sig, provided_sig)
}

/// Constant-time string comparison
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature() {
        let secret = "test_secret";
        let payload = r#"{"test": "data"}"#;

        // Compute expected signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        assert!(verify_signature(secret, payload, &expected));
        assert!(!verify_signature(secret, payload, "sha256=invalid"));
        assert!(!verify_signature(secret, payload, "invalid"));
    }
}
