//! FGP Keychain CLI - Fast access to macOS Keychain.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use clap::{Parser, Subcommand};
use fgp_daemon::FgpService;
use fgp_keychain::KeychainService;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "fgp-keychain")]
#[command(about = "FGP Keychain daemon - fast access to macOS Keychain")]
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
    /// Find a generic password by service and account
    FindGeneric {
        /// Service name (e.g., "myapp")
        #[arg(short, long)]
        service: String,

        /// Account name (e.g., "user@example.com")
        #[arg(short, long)]
        account: String,
    },

    /// Add or update a generic password
    SetGeneric {
        /// Service name
        #[arg(short, long)]
        service: String,

        /// Account name
        #[arg(short, long)]
        account: String,

        /// Password to store
        #[arg(short, long)]
        password: String,
    },

    /// Delete a generic password
    Delete {
        /// Service name
        #[arg(short, long)]
        service: String,

        /// Account name
        #[arg(short, long)]
        account: String,
    },

    /// Check if a password exists
    Exists {
        /// Service name
        #[arg(short, long)]
        service: String,

        /// Account name
        #[arg(short, long)]
        account: String,
    },

    /// Check keychain access status
    Auth,

    /// Run health check
    Health,

    /// List available methods
    Methods,

    /// Start as daemon (use fgp-keychain-daemon instead)
    Start,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fgp_keychain=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let service = KeychainService::new()?;
    let start = Instant::now();

    let result = match cli.command {
        Commands::FindGeneric { service: svc, account } => {
            let mut params = HashMap::new();
            params.insert("service".into(), Value::String(svc));
            params.insert("account".into(), Value::String(account));
            service.dispatch("find_generic", params)
        }
        Commands::SetGeneric {
            service: svc,
            account,
            password,
        } => {
            let mut params = HashMap::new();
            params.insert("service".into(), Value::String(svc));
            params.insert("account".into(), Value::String(account));
            params.insert("password".into(), Value::String(password));
            service.dispatch("set_generic", params)
        }
        Commands::Delete { service: svc, account } => {
            let mut params = HashMap::new();
            params.insert("service".into(), Value::String(svc));
            params.insert("account".into(), Value::String(account));
            service.dispatch("delete", params)
        }
        Commands::Exists { service: svc, account } => {
            let mut params = HashMap::new();
            params.insert("service".into(), Value::String(svc));
            params.insert("account".into(), Value::String(account));
            service.dispatch("exists", params)
        }
        Commands::Auth => service.dispatch("auth", HashMap::new()),
        Commands::Health => {
            let checks = service.health_check();
            Ok(serde_json::to_value(checks)?)
        }
        Commands::Methods => {
            let methods = service.method_list();
            Ok(serde_json::to_value(methods)?)
        }
        Commands::Start => {
            eprintln!("Use 'fgp-keychain-daemon' to start the daemon instead.");
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
