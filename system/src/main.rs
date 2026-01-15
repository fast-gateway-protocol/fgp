//! FGP System CLI - Cached macOS system information.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use clap::{Parser, Subcommand};
use fgp_daemon::FgpService;
use fgp_system::SystemService;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "fgp-system")]
#[command(about = "FGP System daemon - cached macOS system information")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: json or pretty
    #[arg(short, long, default_value = "pretty")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Get hardware information (CPU, RAM, model)
    Hardware,

    /// Get disk usage information
    Disks,

    /// Get network interface information
    Network,

    /// Get running processes sorted by CPU usage
    Processes {
        /// Maximum number of processes to return
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// List installed applications
    Apps {
        /// Search query to filter apps
        #[arg(short, long)]
        query: Option<String>,

        /// Maximum number of apps to return
        #[arg(short, long, default_value = "100")]
        limit: u32,
    },

    /// Get battery status and health
    Battery,

    /// Get system statistics (uptime, load, memory)
    Stats,

    /// Invalidate all caches
    Invalidate,

    /// Get cache statistics
    Cache,

    /// Get bundled system info for dashboards
    Bundle {
        /// Comma-separated list of info to include
        #[arg(short, long, default_value = "hardware,stats,battery")]
        include: String,
    },

    /// Run health check
    Health,

    /// List available methods
    Methods,

    /// Start as daemon (use fgp-system-daemon instead)
    Start,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing for CLI output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fgp_system=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let service = SystemService::new()?;
    let start = Instant::now();

    let result = match cli.command {
        Commands::Hardware => service.dispatch("hardware", HashMap::new()),
        Commands::Disks => service.dispatch("disks", HashMap::new()),
        Commands::Network => service.dispatch("network", HashMap::new()),
        Commands::Processes { limit } => {
            let mut params = HashMap::new();
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("processes", params)
        }
        Commands::Apps { query, limit } => {
            let mut params = HashMap::new();
            if let Some(q) = query {
                params.insert("query".into(), Value::String(q));
            }
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("apps", params)
        }
        Commands::Battery => service.dispatch("battery", HashMap::new()),
        Commands::Stats => service.dispatch("stats", HashMap::new()),
        Commands::Invalidate => service.dispatch("invalidate", HashMap::new()),
        Commands::Cache => service.dispatch("cache", HashMap::new()),
        Commands::Bundle { include } => {
            let mut params = HashMap::new();
            params.insert("include".into(), Value::String(include));
            service.dispatch("bundle", params)
        }
        Commands::Health => {
            let checks = service.health_check();
            Ok(serde_json::to_value(checks)?)
        }
        Commands::Methods => {
            let methods = service.method_list();
            Ok(serde_json::to_value(methods)?)
        }
        Commands::Start => {
            eprintln!("Use 'fgp-system-daemon' to start the daemon instead.");
            std::process::exit(1);
        }
    };

    let elapsed = start.elapsed();

    match result {
        Ok(value) => {
            if cli.format == "json" {
                println!("{}", serde_json::to_string(&value)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&value)?);
                eprintln!("\n⏱️  {}ms", elapsed.as_millis());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
