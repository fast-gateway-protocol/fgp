//! FGP daemon for flight and hotel search.
//!
//! # Usage
//!
//! ```bash
//! fgp-travel start           # Start daemon in background
//! fgp-travel start -f        # Start in foreground
//! fgp-travel stop            # Stop daemon
//! fgp-travel status          # Check daemon status
//! ```
//!
//! # Methods
//!
//! - `travel.find_location` - Search airports/cities
//! - `travel.search_flights` - One-way flight search
//! - `travel.search_roundtrip` - Round-trip flight search
//! - `travel.search_hotels` - Hotel search
//! - `travel.hotel_rates` - Get hotel rates
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/14/2026 - Initial implementation (Claude)

#![allow(dead_code)]

mod api;
mod cache;
mod error;
mod locations;
mod models;
mod service;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fgp_daemon::FgpServer;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

use crate::service::TravelService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/travel/daemon.sock";

#[derive(Parser)]
#[command(name = "fgp-travel")]
#[command(about = "FGP daemon for flight and hotel search")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FGP daemon
    Start {
        /// Socket path
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,

        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Stop the running daemon
    Stop {
        /// Socket path
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },

    /// Check daemon status
    Status {
        /// Socket path
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },

    /// Check daemon health
    Health {
        /// Socket path
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { socket, foreground } => cmd_start(socket, foreground),
        Commands::Stop { socket } => cmd_stop(socket),
        Commands::Status { socket } => cmd_status(socket),
        Commands::Health { socket } => cmd_health(socket),
    }
}

fn expand_socket_path(socket: &str) -> String {
    shellexpand::tilde(socket).to_string()
}

fn cmd_start(socket: String, foreground: bool) -> Result<()> {
    let socket_path = expand_socket_path(&socket);

    // Ensure parent directory exists
    if let Some(parent) = Path::new(&socket_path).parent() {
        fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    // Remove stale socket if exists
    if Path::new(&socket_path).exists() {
        fs::remove_file(&socket_path).ok();
    }

    println!("Starting fgp-travel daemon...");
    println!("Socket: {}", socket_path);
    println!();
    println!("Test with:");
    println!(
        "  echo '{{\"id\":\"1\",\"v\":1,\"method\":\"travel.find_location\",\"params\":{{\"term\":\"SFO\"}}}}' | nc -U {}",
        socket_path
    );
    println!();

    if foreground {
        // Run in foreground with logging
        tracing_subscriber::fmt()
            .with_env_filter("fgp_travel=debug,fgp_daemon=info")
            .init();

        let service = TravelService::new().context("Failed to create TravelService")?;
        let server =
            FgpServer::new(service, &socket_path).context("Failed to create FGP server")?;
        server.serve().context("Server error")?;
    } else {
        // Daemonize
        let pid_file = format!("{}.pid", socket_path);

        let daemonize = daemonize::Daemonize::new()
            .pid_file(&pid_file)
            .working_directory("/tmp");

        match daemonize.start() {
            Ok(_) => {
                // We're now in the daemon process
                tracing_subscriber::fmt()
                    .with_env_filter("fgp_travel=info,fgp_daemon=info")
                    .with_ansi(false)
                    .init();

                let service = TravelService::new().context("Failed to create TravelService")?;
                let server =
                    FgpServer::new(service, &socket_path).context("Failed to create FGP server")?;
                server.serve().context("Server error")?;
            }
            Err(e) => {
                eprintln!("Failed to daemonize: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn cmd_stop(socket: String) -> Result<()> {
    let socket_path = expand_socket_path(&socket);

    if !Path::new(&socket_path).exists() {
        println!("Daemon not running (socket not found)");
        return Ok(());
    }

    // Send stop command
    let request = serde_json::json!({
        "id": "stop",
        "v": 1,
        "method": "stop",
        "params": {}
    });

    match send_request(&socket_path, &request) {
        Ok(response) => {
            if response["ok"].as_bool() == Some(true) {
                println!("Daemon stopped successfully");
            } else {
                println!("Stop request sent: {:?}", response);
            }
        }
        Err(e) => {
            // Connection refused usually means daemon is already stopped
            println!("Daemon appears to be stopped: {}", e);
        }
    }

    // Clean up socket file
    fs::remove_file(&socket_path).ok();

    // Clean up PID file
    let pid_file = format!("{}.pid", socket_path);
    fs::remove_file(&pid_file).ok();

    Ok(())
}

fn cmd_status(socket: String) -> Result<()> {
    let socket_path = expand_socket_path(&socket);

    if !Path::new(&socket_path).exists() {
        println!("Status: NOT RUNNING (socket not found)");
        return Ok(());
    }

    let request = serde_json::json!({
        "id": "health",
        "v": 1,
        "method": "health",
        "params": {}
    });

    match send_request(&socket_path, &request) {
        Ok(response) => {
            if response["ok"].as_bool() == Some(true) {
                println!("Status: RUNNING");
                if let Some(result) = response.get("result") {
                    println!(
                        "Service: {}",
                        result["service"].as_str().unwrap_or("travel")
                    );
                    println!(
                        "Version: {}",
                        result["version"].as_str().unwrap_or("unknown")
                    );
                    println!(
                        "Uptime: {:.1}s",
                        result["uptime_secs"].as_f64().unwrap_or(0.0)
                    );
                }
            } else {
                println!("Status: ERROR");
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
        }
        Err(e) => {
            println!("Status: NOT RUNNING (connection failed: {})", e);
        }
    }

    Ok(())
}

fn cmd_health(socket: String) -> Result<()> {
    let socket_path = expand_socket_path(&socket);

    if !Path::new(&socket_path).exists() {
        println!("Daemon not running (socket not found)");
        return Ok(());
    }

    let request = serde_json::json!({
        "id": "health",
        "v": 1,
        "method": "health",
        "params": {}
    });

    match send_request(&socket_path, &request) {
        Ok(response) => {
            if response["ok"].as_bool() == Some(true) {
                if let Some(checks) = response["result"]["checks"].as_object() {
                    println!("Health checks:");
                    for (name, status) in checks {
                        let ok = status["ok"].as_bool().unwrap_or(false);
                        let symbol = if ok { "✓" } else { "✗" };
                        let latency = status["latency_ms"]
                            .as_f64()
                            .map(|l| format!(" ({:.1}ms)", l))
                            .unwrap_or_default();
                        let message = status["message"]
                            .as_str()
                            .map(|m| format!(" - {}", m))
                            .unwrap_or_default();
                        println!("  {} {}{}{}", symbol, name, latency, message);
                    }
                }
            } else {
                println!("Health check failed:");
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }

    Ok(())
}

fn send_request(socket_path: &str, request: &serde_json::Value) -> Result<serde_json::Value> {
    let mut stream = UnixStream::connect(socket_path).context("Failed to connect to daemon")?;

    let mut request_str = serde_json::to_string(request)?;
    request_str.push('\n');

    stream
        .write_all(request_str.as_bytes())
        .context("Failed to send request")?;

    let mut reader = BufReader::new(stream);
    let mut response_str = String::new();
    reader
        .read_line(&mut response_str)
        .context("Failed to read response")?;

    let response: serde_json::Value =
        serde_json::from_str(&response_str).context("Failed to parse response")?;

    Ok(response)
}
