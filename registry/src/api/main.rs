//! FGP Skill Registry API Server
//!
//! A REST API for the FGP Skill Registry, allowing distributed access
//! to skill search and installation without direct database credentials.
//!
//! ## Usage
//!
//! ```bash
//! # Set database URL
//! export DATABASE_URL="postgres://..."
//!
//! # Run the server
//! fgp-registry-api
//!
//! # Or with custom host/port
//! fgp-registry-api --host 0.0.0.0 --port 8080
//! ```
//!
//! ## Endpoints
//!
//! - `GET /health` - Health check
//! - `GET /api/v1/skills?q=<query>&tier=<tier>&category=<cat>` - Search skills
//! - `GET /api/v1/skills/:slug` - Get skill details
//! - `GET /api/v1/skills/:slug/install` - Get SKILL.md content
//! - `GET /api/v1/categories` - List categories
//! - `GET /api/v1/stats` - Registry statistics

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use fgp_registry::api::{create_router, AppState};
use fgp_registry::db::Database;

/// FGP Skill Registry API Server
#[derive(Parser, Debug)]
#[command(name = "fgp-registry-api")]
#[command(about = "HTTP API server for the FGP Skill Registry")]
#[command(version)]
struct Args {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Database URL (or use DATABASE_URL env var)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fgp_registry_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse arguments
    let args = Args::parse();

    // Get database URL
    let database_url = args.database_url.or_else(|| {
        dotenvy::dotenv().ok();
        std::env::var("DATABASE_URL").ok()
    }).ok_or_else(|| anyhow::anyhow!("DATABASE_URL not set"))?;

    // Connect to database
    tracing::info!("Connecting to database...");
    let db = Database::connect(&database_url).await?;
    tracing::info!("Database connected");

    // Get optional admin API key for protected endpoints
    let admin_api_key = std::env::var("FGP_REGISTRY_API_KEY").ok();
    if admin_api_key.is_some() {
        tracing::info!("Admin API key configured for publish operations");
    }

    // Create app state
    let state = Arc::new(AppState { db, admin_api_key });

    // Create router
    let app = create_router(state);

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("🚀 FGP Registry API listening on http://{}", addr);
    tracing::info!("   Health: http://{}/health", addr);
    tracing::info!("   Search: http://{}/api/v1/skills?q=<query>", addr);
    tracing::info!("   Stats:  http://{}/api/v1/stats", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
