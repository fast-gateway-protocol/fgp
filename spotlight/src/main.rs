//! fgp-spotlight - Fast Spotlight gateway for macOS
//!
//! Direct Spotlight metadata queries using mdquery-rs.
//! No special permissions required.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::process::ExitCode;

mod daemon;
mod spotlight;

use daemon::service::SpotlightService;
use fgp_daemon::service::FgpService;

/// Fast Spotlight gateway for macOS - direct metadata queries.
#[derive(Parser, Debug)]
#[command(name = "fgp-spotlight")]
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
    /// Raw Spotlight query (mdfind syntax)
    Search {
        /// Query string (mdfind syntax)
        query: String,

        /// Search scope: home, computer, network, or a path
        #[arg(short, long, default_value = "home")]
        scope: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Search files by name (substring match)
    ByName {
        /// Name pattern to search
        name: String,

        /// Search scope
        #[arg(short, long, default_value = "home")]
        scope: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Search files by extension
    ByExtension {
        /// File extension (without dot)
        extension: String,

        /// Search scope
        #[arg(short, long, default_value = "home")]
        scope: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Search by kind (pdf, image, video, audio, document, etc)
    ByKind {
        /// Kind: pdf, image, video, audio, document, text, source, folder, app, archive
        kind: String,

        /// Optional name filter
        #[arg(short, long)]
        name: Option<String>,

        /// Search scope
        #[arg(short, long, default_value = "home")]
        scope: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Find recently modified files
    Recent {
        /// Days to look back
        #[arg(short, long, default_value_t = 7)]
        days: u32,

        /// Search scope
        #[arg(short, long, default_value = "home")]
        scope: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Find applications
    Apps {
        /// Optional name filter
        #[arg(short, long)]
        name: Option<String>,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Find directories
    Directories {
        /// Optional name filter
        #[arg(short, long)]
        name: Option<String>,

        /// Search scope
        #[arg(short, long, default_value = "home")]
        scope: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Find files by size range
    BySize {
        /// Minimum size in bytes
        #[arg(long)]
        min: Option<u64>,

        /// Maximum size in bytes
        #[arg(long)]
        max: Option<u64>,

        /// Search scope
        #[arg(short, long, default_value = "home")]
        scope: String,

        /// Maximum results
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

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
            let service = SpotlightService::new()?;
            let socket_path = fgp_daemon::service_socket_path("spotlight");

            println!("Starting Spotlight daemon at {}", socket_path.display());

            let server = fgp_daemon::FgpServer::new(service, &socket_path)?;
            server.serve()?;
            Ok(())
        }

        Command::Stop => {
            fgp_daemon::stop_service("spotlight")?;
            println!("Spotlight daemon stopped");
            Ok(())
        }

        Command::Health => {
            let service = SpotlightService::new()?;
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
            let service = SpotlightService::new()?;
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
                            println!(
                                "    {}{}: {}{}",
                                param.name, req, param.param_type, default
                            );
                        }
                    }
                    println!();
                }
            }
            Ok(())
        }

        Command::Search { query, scope, limit } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            params.insert("query".to_string(), serde_json::json!(query));
            params.insert("scope".to_string(), serde_json::json!(scope));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("search", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::ByName { name, scope, limit } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            params.insert("name".to_string(), serde_json::json!(name));
            params.insert("scope".to_string(), serde_json::json!(scope));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("by_name", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::ByExtension {
            extension,
            scope,
            limit,
        } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            params.insert("extension".to_string(), serde_json::json!(extension));
            params.insert("scope".to_string(), serde_json::json!(scope));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("by_extension", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::ByKind {
            kind,
            name,
            scope,
            limit,
        } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            params.insert("kind".to_string(), serde_json::json!(kind));
            if let Some(n) = name {
                params.insert("name".to_string(), serde_json::json!(n));
            }
            params.insert("scope".to_string(), serde_json::json!(scope));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("by_kind", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Recent { days, scope, limit } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            params.insert("days".to_string(), serde_json::json!(days));
            params.insert("scope".to_string(), serde_json::json!(scope));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("recent", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Apps { name, limit } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            if let Some(n) = name {
                params.insert("name".to_string(), serde_json::json!(n));
            }
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("apps", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Directories { name, scope, limit } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            if let Some(n) = name {
                params.insert("name".to_string(), serde_json::json!(n));
            }
            params.insert("scope".to_string(), serde_json::json!(scope));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("directories", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::BySize {
            min,
            max,
            scope,
            limit,
        } => {
            let service = SpotlightService::new()?;
            let mut params = HashMap::new();
            if let Some(m) = min {
                params.insert("min_bytes".to_string(), serde_json::json!(m));
            }
            if let Some(m) = max {
                params.insert("max_bytes".to_string(), serde_json::json!(m));
            }
            params.insert("scope".to_string(), serde_json::json!(scope));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("by_size", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }
    }
}

fn print_result(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else if let Some(results) = result.get("results").and_then(|v| v.as_array()) {
        for (i, item) in results.iter().enumerate() {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("(unknown)");
            let path = item.get("path").and_then(|v| v.as_str()).unwrap_or("");

            println!("{}. {}", i + 1, name);
            println!("   {}", path);
        }
        if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
            println!("\nTotal: {} results", count);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(result)?);
    }
    Ok(())
}
