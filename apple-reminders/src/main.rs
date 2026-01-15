//! FGP Apple Reminders CLI - Fast access to Apple Reminders.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use clap::{Parser, Subcommand};
use fgp_apple_reminders::RemindersService;
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "fgp-apple-reminders")]
#[command(about = "FGP Apple Reminders daemon - fast access via EventKit")]
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
    /// List all reminder lists
    Lists,

    /// Get all reminders
    All {
        /// Maximum number of reminders
        #[arg(short, long, default_value = "100")]
        limit: u32,
    },

    /// Get incomplete reminders
    Incomplete {
        /// Maximum number of reminders
        #[arg(short, long, default_value = "100")]
        limit: u32,
    },

    /// Get completed reminders
    Completed {
        /// Days to look back
        #[arg(short, long, default_value = "30")]
        days: u32,

        /// Maximum number of reminders
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Get reminders due today
    DueToday,

    /// Get overdue reminders
    Overdue,

    /// Get upcoming reminders
    Upcoming {
        /// Number of days to look ahead
        #[arg(short, long, default_value = "7")]
        days: u32,

        /// Maximum number of reminders
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Search reminders by title/notes
    Search {
        /// Search query
        query: String,

        /// Include completed reminders
        #[arg(short, long)]
        include_completed: bool,

        /// Maximum results
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Get reminders for a specific date
    OnDate {
        /// Date (YYYY-MM-DD)
        date: String,
    },

    /// Check authorization status
    Auth,

    /// Run health check
    Health,

    /// List available methods
    Methods,

    /// Start as daemon (use fgp-apple-reminders-daemon instead)
    Start,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fgp_apple_reminders=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let service = RemindersService::new()?;
    let start = Instant::now();

    let result = match cli.command {
        Commands::Lists => service.dispatch("lists", HashMap::new()),
        Commands::All { limit } => {
            let mut params = HashMap::new();
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("all", params)
        }
        Commands::Incomplete { limit } => {
            let mut params = HashMap::new();
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("incomplete", params)
        }
        Commands::Completed { days, limit } => {
            let mut params = HashMap::new();
            params.insert("days".into(), Value::Number(days.into()));
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("completed", params)
        }
        Commands::DueToday => service.dispatch("due_today", HashMap::new()),
        Commands::Overdue => service.dispatch("overdue", HashMap::new()),
        Commands::Upcoming { days, limit } => {
            let mut params = HashMap::new();
            params.insert("days".into(), Value::Number(days.into()));
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("upcoming", params)
        }
        Commands::Search {
            query,
            include_completed,
            limit,
        } => {
            let mut params = HashMap::new();
            params.insert("query".into(), Value::String(query));
            params.insert("include_completed".into(), Value::Bool(include_completed));
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("search", params)
        }
        Commands::OnDate { date } => {
            let mut params = HashMap::new();
            params.insert("date".into(), Value::String(date));
            service.dispatch("on_date", params)
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
            eprintln!("Use 'fgp-apple-reminders-daemon' to start the daemon instead.");
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
