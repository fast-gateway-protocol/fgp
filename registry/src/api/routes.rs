//! Route definitions for the FGP Skill Registry API.

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    compression::CompressionLayer,
    trace::TraceLayer,
};

use super::handlers::{self, AppState};
use super::webhooks;

/// Create the main API router
pub fn create_router(state: Arc<AppState>) -> Router {
    // API v1 routes
    let api_v1 = Router::new()
        // Skills
        .route("/skills", get(handlers::search_skills).post(handlers::publish_skill))
        .route("/skills/{slug}", get(handlers::get_skill))
        .route("/skills/{slug}/install", get(handlers::install_skill))
        // Categories
        .route("/categories", get(handlers::list_categories))
        // Stats
        .route("/stats", get(handlers::get_stats))
        // Webhooks
        .route("/webhooks/github", post(webhooks::github_webhook));

    // Main router with middleware
    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/api/v1", api_v1)
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
