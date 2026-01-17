//! HTTP API for the FGP Skill Registry.
//!
//! This module provides a REST API for searching and installing skills
//! without requiring direct database access.
//!
//! ## Endpoints
//!
//! - `GET /health` - Health check
//! - `GET /api/v1/skills` - Search skills (query params: q, tier, category, fgp_only, sort, page, limit)
//! - `POST /api/v1/skills` - Publish a new skill (requires X-API-Key header)
//! - `GET /api/v1/skills/:slug` - Get skill details
//! - `GET /api/v1/skills/:slug/install` - Get skill content for installation
//! - `GET /api/v1/categories` - List all categories
//! - `GET /api/v1/stats` - Get registry statistics
//! - `POST /api/v1/webhooks/github` - GitHub webhook for auto-sync

pub mod auth;
pub mod handlers;
pub mod responses;
pub mod routes;
pub mod webhooks;

pub use handlers::AppState;
pub use routes::create_router;
