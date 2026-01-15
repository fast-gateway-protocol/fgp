//! fgp-safari - Fast Safari gateway for macOS
//!
//! Direct SQLite access to Safari History, CloudTabs, and Bookmarks.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::process::ExitCode;

mod daemon;
mod db;

use daemon::service::SafariService;
use fgp_daemon::service::FgpService;

/// Fast Safari gateway for macOS - direct SQLite queries.
#[derive(Parser, Debug)]
#[command(name = "fgp-safari")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Get recent browser history
    History {
        /// Number of days to look back
        #[arg(short, long, default_value_t = 7)]
        days: u32,

        /// Maximum items to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Search history by URL or title
    Search {
        /// Search query
        query: String,

        /// Number of days to look back
        #[arg(short, long, default_value_t = 30)]
        days: u32,

        /// Maximum items to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Get most visited sites
    TopSites {
        /// Number of days to look back
        #[arg(short, long, default_value_t = 30)]
        days: u32,

        /// Maximum sites to return
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// Get browsing statistics
    Stats {
        /// Number of days to analyze
        #[arg(short, long, default_value_t = 30)]
        days: u32,
    },

    /// Get tabs from other devices via iCloud
    CloudTabs,

    /// Check daemon health
    Health,

    /// List available methods
    Methods,

    /// Start the daemon (foreground)
    Start,

    /// Stop a running daemon
    Stop,
}

fn main() -> ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match run(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Start => {
            // Start daemon in foreground
            let service = SafariService::new()?;
            let socket_path = fgp_daemon::service_socket_path("safari");

            println!("Starting Safari daemon at {}", socket_path.display());

            let server = fgp_daemon::FgpServer::new(service, &socket_path)?;
            server.serve()?;
            Ok(())
        }

        Command::Stop => {
            fgp_daemon::stop_service("safari")?;
            println!("Safari daemon stopped");
            Ok(())
        }

        Command::Health => {
            let service = SafariService::new()?;
            let checks = service.health_check();
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&checks)?);
            } else {
                for (name, status) in &checks {
                    let icon = if status.ok { "✓" } else { "✗" };
                    let msg = status.message.as_deref().unwrap_or("");
                    println!("{} {} - {}", icon, name, msg);
                }
            }
            Ok(())
        }

        Command::Methods => {
            let service = SafariService::new()?;
            let methods = service.method_list();
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&methods)?);
            } else {
                for method in &methods {
                    println!("{}:", method.name);
                    println!("  {}", method.description);
                    if !method.params.is_empty() {
                        println!("  Parameters:");
                        for param in &method.params {
                            let req = if param.required { "*" } else { "" };
                            let default = param
                                .default
                                .as_ref()
                                .map(|v| format!(" (default: {})", v))
                                .unwrap_or_default();
                            println!("    {}{}: {}{}", param.name, req, param.param_type, default);
                        }
                    }
                    println!();
                }
            }
            Ok(())
        }

        Command::History { days, limit } => {
            let service = SafariService::new()?;
            let mut params = HashMap::new();
            params.insert("days".to_string(), serde_json::json!(days));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("history", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Search { query, days, limit } => {
            let service = SafariService::new()?;
            let mut params = HashMap::new();
            params.insert("query".to_string(), serde_json::json!(query));
            params.insert("days".to_string(), serde_json::json!(days));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("search", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::TopSites { days, limit } => {
            let service = SafariService::new()?;
            let mut params = HashMap::new();
            params.insert("days".to_string(), serde_json::json!(days));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("top_sites", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Stats { days } => {
            let service = SafariService::new()?;
            let mut params = HashMap::new();
            params.insert("days".to_string(), serde_json::json!(days));

            let result = service.dispatch("stats", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::CloudTabs => {
            let service = SafariService::new()?;
            let result = service.dispatch("cloud_tabs", HashMap::new())?;
            print_result(&result, cli.json)?;
            Ok(())
        }
    }
}

fn print_result(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else {
        // Pretty print based on result type
        if let Some(items) = result.get("items").and_then(|v| v.as_array()) {
            for item in items {
                if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                    let title = item
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("(no title)");
                    let time = item.get("visit_time").and_then(|v| v.as_str()).unwrap_or("");
                    println!("{}", title);
                    println!("  {}", url);
                    if !time.is_empty() {
                        println!("  {}", time);
                    }
                    println!();
                }
            }
            if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
                println!("Total: {} items", count);
            }
        } else if let Some(sites) = result.get("sites").and_then(|v| v.as_array()) {
            for (i, site) in sites.iter().enumerate() {
                let url = site.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let visits = site.get("visit_count").and_then(|v| v.as_i64()).unwrap_or(0);
                let domain = site.get("domain").and_then(|v| v.as_str()).unwrap_or("");
                println!("{}. {} ({} visits)", i + 1, domain, visits);
                println!("   {}", url);
            }
        } else if let Some(tabs) = result.get("tabs").and_then(|v| v.as_array()) {
            let mut current_device = String::new();
            for tab in tabs {
                let device = tab
                    .get("device_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown Device");
                if device != current_device {
                    println!("\n[{}]", device);
                    current_device = device.to_string();
                }
                let title = tab
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no title)");
                let url = tab.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let pinned = tab
                    .get("is_pinned")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let pin_icon = if pinned { "📌 " } else { "  " };
                println!("{}{}", pin_icon, title);
                println!("    {}", url);
            }
            if let Some(total) = result.get("total_tabs").and_then(|v| v.as_i64()) {
                println!("\nTotal: {} tabs across {} devices", total,
                    result.get("total_devices").and_then(|v| v.as_i64()).unwrap_or(0));
            }
        } else if result.get("total_visits").is_some() {
            // Stats output
            let total = result.get("total_visits").and_then(|v| v.as_i64()).unwrap_or(0);
            let unique = result.get("unique_pages").and_then(|v| v.as_i64()).unwrap_or(0);
            let days = result.get("active_days").and_then(|v| v.as_i64()).unwrap_or(0);
            let period = result.get("period_days").and_then(|v| v.as_i64()).unwrap_or(0);
            let avg = result.get("avg_visits_per_day").and_then(|v| v.as_i64()).unwrap_or(0);

            println!("Safari Statistics ({} days)", period);
            println!("  Total visits:      {}", total);
            println!("  Unique pages:      {}", unique);
            println!("  Active days:       {}", days);
            println!("  Avg visits/day:    {}", avg);
        } else {
            // Fallback to JSON
            println!("{}", serde_json::to_string_pretty(result)?);
        }
    }
    Ok(())
}
