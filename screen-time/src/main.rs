//! FGP Screen Time CLI - Fast access to macOS app usage data.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use clap::{Parser, Subcommand};
use fgp_daemon::FgpService;
use fgp_screen_time::ScreenTimeService;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "fgp-screen-time")]
#[command(about = "FGP Screen Time daemon - fast access to macOS app usage data")]
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
    /// Get daily total screen time with per-app breakdown
    DailyTotal {
        /// Date in YYYY-MM-DD format (default: today)
        #[arg(short, long)]
        date: Option<String>,
    },

    /// Get usage for a specific app
    AppUsage {
        /// Bundle ID (e.g., com.apple.Safari)
        #[arg(short, long)]
        bundle_id: String,

        /// Number of days to look back
        #[arg(short, long, default_value = "7")]
        days: i64,
    },

    /// Get 7-day weekly summary
    WeeklySummary,

    /// Get most used apps
    MostUsed {
        /// Number of apps to show
        #[arg(short, long, default_value = "10")]
        limit: i64,

        /// Number of days to look back
        #[arg(short, long, default_value = "7")]
        days: i64,
    },

    /// Get hourly usage timeline for a day
    Timeline {
        /// Date in YYYY-MM-DD format (default: today)
        #[arg(short, long)]
        date: Option<String>,
    },

    /// Check Screen Time data access status
    Auth,

    /// Run health check
    Health,

    /// List available methods
    Methods,

    /// Start as daemon (use fgp-screen-time-daemon instead)
    Start,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fgp_screen_time=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let service = ScreenTimeService::new()?;
    let start = Instant::now();

    let result = match cli.command {
        Commands::DailyTotal { date } => {
            let mut params = HashMap::new();
            if let Some(d) = date {
                params.insert("date".into(), Value::String(d));
            }
            service.dispatch("daily_total", params)
        }
        Commands::AppUsage { bundle_id, days } => {
            let mut params = HashMap::new();
            params.insert("bundle_id".into(), Value::String(bundle_id));
            params.insert("days".into(), Value::Number(days.into()));
            service.dispatch("app_usage", params)
        }
        Commands::WeeklySummary => service.dispatch("weekly_summary", HashMap::new()),
        Commands::MostUsed { limit, days } => {
            let mut params = HashMap::new();
            params.insert("limit".into(), Value::Number(limit.into()));
            params.insert("days".into(), Value::Number(days.into()));
            service.dispatch("most_used", params)
        }
        Commands::Timeline { date } => {
            let mut params = HashMap::new();
            if let Some(d) = date {
                params.insert("date".into(), Value::String(d));
            }
            service.dispatch("usage_timeline", params)
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
            eprintln!("Use 'fgp-screen-time-daemon' to start the daemon instead.");
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
