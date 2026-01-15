//! FGP Apple Reminders Daemon binary.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use fgp_apple_reminders::RemindersService;
use fgp_daemon::FgpServer;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fgp_daemon=info".parse().unwrap())
                .add_directive("fgp_apple_reminders=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    // Determine socket path
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let socket_dir = PathBuf::from(home).join(".fgp/services/apple-reminders");

    std::fs::create_dir_all(&socket_dir)?;
    let socket_path = socket_dir.join("daemon.sock");

    // Create service
    let service = RemindersService::new()?;

    // Create and run server
    let server = FgpServer::new(service, &socket_path)?;

    tracing::info!(
        "Apple Reminders daemon listening on {}",
        socket_path.display()
    );

    server.serve()?;

    Ok(())
}
