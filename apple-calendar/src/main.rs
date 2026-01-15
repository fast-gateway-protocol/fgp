//! FGP Apple Calendar CLI - Fast access to Apple Calendar.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use clap::{Parser, Subcommand};
use fgp_apple_calendar::CalendarService;
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "fgp-apple-calendar")]
#[command(about = "FGP Apple Calendar daemon - fast access via EventKit")]
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
    /// List all calendars
    Calendars,

    /// Get today's events
    Today,

    /// Get upcoming events
    Upcoming {
        /// Number of days to look ahead
        #[arg(short, long, default_value = "7")]
        days: u32,

        /// Maximum number of events
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },

    /// Get events in a date range
    Events {
        /// Start date (YYYY-MM-DD)
        #[arg(short, long)]
        start: String,

        /// End date (YYYY-MM-DD)
        #[arg(short, long)]
        end: String,

        /// Calendar IDs (comma-separated)
        #[arg(short, long)]
        calendars: Option<String>,
    },

    /// Search events by title/location
    Search {
        /// Search query
        query: String,

        /// Days to search
        #[arg(short, long, default_value = "30")]
        days: u32,

        /// Maximum results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Get events for a specific date
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

    /// Start as daemon (use fgp-apple-calendar-daemon instead)
    Start,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fgp_apple_calendar=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let service = CalendarService::new()?;
    let start = Instant::now();

    let result = match cli.command {
        Commands::Calendars => service.dispatch("calendars", HashMap::new()),
        Commands::Today => service.dispatch("today", HashMap::new()),
        Commands::Upcoming { days, limit } => {
            let mut params = HashMap::new();
            params.insert("days".into(), Value::Number(days.into()));
            params.insert("limit".into(), Value::Number(limit.into()));
            service.dispatch("upcoming", params)
        }
        Commands::Events {
            start,
            end,
            calendars,
        } => {
            let mut params = HashMap::new();
            params.insert("start".into(), Value::String(start));
            params.insert("end".into(), Value::String(end));
            if let Some(cals) = calendars {
                params.insert("calendars".into(), Value::String(cals));
            }
            service.dispatch("events", params)
        }
        Commands::Search { query, days, limit } => {
            let mut params = HashMap::new();
            params.insert("query".into(), Value::String(query));
            params.insert("days".into(), Value::Number(days.into()));
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
            eprintln!("Use 'fgp-apple-calendar-daemon' to start the daemon instead.");
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
