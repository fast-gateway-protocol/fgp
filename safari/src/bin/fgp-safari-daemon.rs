//! FGP Safari daemon binary.
//!
//! This is the daemonized version that runs in the background.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use clap::Parser;
use fgp_daemon::{daemonize, service_socket_path, write_pid_file, FgpServer};
use fgp_safari::daemon::service::SafariService;

/// Safari FGP daemon - runs as a background service.
#[derive(Parser, Debug)]
#[command(name = "fgp-safari-daemon")]
#[command(version, about)]
struct Args {
    /// Run in foreground (don't daemonize)
    #[arg(long)]
    foreground: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let socket_path = service_socket_path("safari");
    let pid_path = fgp_daemon::service_pid_path("safari");

    if !args.foreground {
        daemonize(&pid_path, None)?;
    }

    tracing::info!("Starting Safari FGP daemon at {}", socket_path.display());

    let service = SafariService::new()?;
    let server = FgpServer::new(service, &socket_path)?;
    server.serve()?;

    Ok(())
}
