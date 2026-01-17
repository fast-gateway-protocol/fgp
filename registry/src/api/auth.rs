//! Authentication middleware for admin operations.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use super::handlers::AppState;

/// API key header name
pub const API_KEY_HEADER: &str = "X-API-Key";

/// Middleware to verify admin API key
pub async fn require_api_key(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Get the expected API key from state
    let expected_key = match &state.admin_api_key {
        Some(key) => key,
        None => {
            // No API key configured, allow all requests (dev mode)
            return next.run(request).await;
        }
    };

    // Check the API key header
    let provided_key = request
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|v| v.to_str().ok());

    match provided_key {
        Some(key) if key == expected_key => {
            // Valid API key, proceed
            next.run(request).await
        }
        Some(_) => {
            // Invalid API key
            (StatusCode::UNAUTHORIZED, "Invalid API key").into_response()
        }
        None => {
            // Missing API key
            (StatusCode::UNAUTHORIZED, "API key required").into_response()
        }
    }
}
