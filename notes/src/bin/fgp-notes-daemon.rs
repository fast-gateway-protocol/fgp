//! FGP Notes daemon binary.
//!
//! This is the daemonized version that runs in the background.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use clap::Parser;
use fgp_daemon::{daemonize, service_socket_path, FgpServer};
use fgp_notes::daemon::service::NotesService;

/// Notes FGP daemon - runs as a background service.
#[derive(Parser, Debug)]
#[command(name = "fgp-notes-daemon")]
#[command(version, about)]
struct Args {
    /// Run in foreground (don't daemonize)
    #[arg(long)]
    foreground: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let socket_path = service_socket_path("notes");
    let pid_path = fgp_daemon::service_pid_path("notes");

    if !args.foreground {
        daemonize(&pid_path, None)?;
    }

    tracing::info!("Starting Notes FGP daemon at {}", socket_path.display());

    let service = NotesService::new()?;
    let server = FgpServer::new(service, &socket_path)?;
    server.serve()?;

    Ok(())
}
