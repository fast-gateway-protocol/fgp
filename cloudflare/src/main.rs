//! FGP daemon for Cloudflare operations.
//!
//! Provides fast DNS, Workers, and KV operations for Cloudflare.
//! ~40-80x faster than MCP stdio servers.
//!
//! # Usage
//! ```bash
//! fgp-cloudflare start           # Start daemon in background
//! fgp-cloudflare start -f        # Start in foreground
//! fgp-cloudflare stop            # Stop daemon
//! fgp-cloudflare status          # Check daemon status
//! ```
//!
//! # Authentication
//! Environment variables:
//! - CLOUDFLARE_API_TOKEN: API token with appropriate permissions
//! - CLOUDFLARE_ACCOUNT_ID: Account ID (for KV and Workers)
//!
//! # Methods
//! ## Zones
//! - `cloudflare.zones` - List all zones
//! - `cloudflare.zone` - Get zone by ID
//!
//! ## DNS
//! - `cloudflare.dns.list` - List DNS records
//! - `cloudflare.dns.create` - Create DNS record
//! - `cloudflare.dns.update` - Update DNS record
//! - `cloudflare.dns.delete` - Delete DNS record
//!
//! ## KV
//! - `cloudflare.kv.namespaces` - List KV namespaces
//! - `cloudflare.kv.keys` - List keys in namespace
//! - `cloudflare.kv.read` - Read a value
//! - `cloudflare.kv.write` - Write a value
//! - `cloudflare.kv.delete` - Delete a key
//!
//! ## Workers
//! - `cloudflare.workers` - List Workers
//! - `cloudflare.workers.routes` - List Worker routes
//!
//! ## Cache
//! - `cloudflare.purge_cache` - Purge cache for zone
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

mod api;
mod models;
mod service;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fgp_daemon::{cleanup_socket, FgpServer};
use std::path::Path;
use std::process::Command;

use crate::service::CloudflareService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/cloudflare/daemon.sock";

#[derive(Parser)]
#[command(name = "fgp-cloudflare")]
#[command(about = "FGP daemon for Cloudflare DNS, Workers, and KV operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FGP daemon
    Start {
        /// Socket path (default: ~/.fgp/services/cloudflare/daemon.sock)
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { socket, foreground } => cmd_start(socket, foreground),
        Commands::Stop { socket } => cmd_stop(socket),
        Commands::Status { socket } => cmd_status(socket),
    }
}

fn cmd_start(socket: String, foreground: bool) -> Result<()> {
    let socket_path = shellexpand::tilde(&socket).to_string();

    if let Some(parent) = Path::new(&socket_path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    let pid_file = format!("{}.pid", socket_path);

    println!("Starting fgp-cloudflare daemon...");
    println!("Socket: {}", socket_path);
    println!();
    println!("Available methods:");
    println!("  cloudflare.zones        - List all zones");
    println!("  cloudflare.zone         - Get zone by ID");
    println!("  cloudflare.dns.list     - List DNS records");
    println!("  cloudflare.dns.create   - Create DNS record");
    println!("  cloudflare.dns.update   - Update DNS record");
    println!("  cloudflare.dns.delete   - Delete DNS record");
    println!("  cloudflare.kv.namespaces - List KV namespaces");
    println!("  cloudflare.kv.keys      - List keys in namespace");
    println!("  cloudflare.kv.read      - Read a value");
    println!("  cloudflare.kv.write     - Write a value");
    println!("  cloudflare.kv.delete    - Delete a key");
    println!("  cloudflare.workers      - List Workers");
    println!("  cloudflare.purge_cache  - Purge cache for zone");
    println!();
    println!("Test with:");
    println!("  fgp call cloudflare.zones");
    println!();

    if foreground {
        tracing_subscriber::fmt()
            .with_env_filter("fgp_cloudflare=debug,fgp_daemon=debug")
            .init();

        let service = CloudflareService::new().context("Failed to create CloudflareService")?;
        let server =
            FgpServer::new(service, &socket_path).context("Failed to create FGP server")?;
        server.serve().context("Server error")?;
    } else {
        use daemonize::Daemonize;

        let daemonize = Daemonize::new()
            .pid_file(&pid_file)
            .working_directory("/tmp");

        match daemonize.start() {
            Ok(_) => {
                tracing_subscriber::fmt()
                    .with_env_filter("fgp_cloudflare=debug,fgp_daemon=debug")
                    .init();

                let service = CloudflareService::new().context("Failed to create CloudflareService")?;
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
    let socket_path = shellexpand::tilde(&socket).to_string();
    let pid_file = format!("{}.pid", socket_path);

    if Path::new(&socket_path).exists() {
        if let Ok(client) = fgp_daemon::FgpClient::new(&socket_path) {
            if let Ok(response) = client.stop() {
                if response.ok {
                    println!("Daemon stopped.");
                    return Ok(());
                }
            }
        }
    }

    let pid_str = std::fs::read_to_string(&pid_file)
        .context("Failed to read PID file - daemon may not be running")?;
    let pid: i32 = pid_str.trim().parse().context("Invalid PID in file")?;

    if !pid_matches_process(pid, "fgp-cloudflare") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-cloudflare daemon (PID: {})...", pid);

    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    let _ = cleanup_socket(&socket_path, Some(Path::new(&pid_file)));
    let _ = std::fs::remove_file(&pid_file);

    println!("Daemon stopped.");

    Ok(())
}

fn pid_matches_process(pid: i32, expected_name: &str) -> bool {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let command = String::from_utf8_lossy(&output.stdout);
            command.trim().contains(expected_name)
        }
        _ => false,
    }
}

fn cmd_status(socket: String) -> Result<()> {
    let socket_path = shellexpand::tilde(&socket).to_string();

    if !Path::new(&socket_path).exists() {
        println!("Status: NOT RUNNING");
        println!("Socket {} does not exist", socket_path);
        return Ok(());
    }

    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            let request = r#"{"id":"status","v":1,"method":"health","params":{}}"#;
            writeln!(stream, "{}", request)?;
            stream.flush()?;

            let mut reader = BufReader::new(stream);
            let mut response = String::new();
            reader.read_line(&mut response)?;

            println!("Status: RUNNING");
            println!("Socket: {}", socket_path);
            println!("Health: {}", response.trim());
        }
        Err(e) => {
            println!("Status: NOT RESPONDING");
            println!("Socket exists but connection failed: {}", e);
        }
    }

    Ok(())
}
