//! FGP daemon for Zapier webhook automation.
//!
//! Provides fast webhook triggering and NLA (Natural Language Actions) for Zapier.
//! ~40-100x faster than MCP stdio servers.
//!
//! # Usage
//! ```bash
//! fgp-zapier start           # Start daemon in background
//! fgp-zapier start -f        # Start in foreground
//! fgp-zapier stop            # Stop daemon
//! fgp-zapier status          # Check daemon status
//! ```
//!
//! # Authentication
//! Environment variables (optional):
//! - ZAPIER_NLA_API_KEY: For Natural Language Actions API
//!
//! # Methods
//! - `zapier.trigger` - Trigger a Zap via webhook URL
//! - `zapier.trigger_named` - Trigger registered Zap by name
//! - `zapier.register_webhook` - Register webhook URL with alias
//! - `zapier.list_webhooks` - List registered webhooks
//! - `zapier.remove_webhook` - Remove registered webhook
//! - `zapier.nla_actions` - List NLA actions (if API key set)
//! - `zapier.nla_execute` - Execute NLA action
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

use crate::service::ZapierService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/zapier/daemon.sock";

#[derive(Parser)]
#[command(name = "fgp-zapier")]
#[command(about = "FGP daemon for Zapier webhook automation")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FGP daemon
    Start {
        /// Socket path (default: ~/.fgp/services/zapier/daemon.sock)
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

    // Create parent directory
    if let Some(parent) = Path::new(&socket_path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    let pid_file = format!("{}.pid", socket_path);

    println!("Starting fgp-zapier daemon...");
    println!("Socket: {}", socket_path);
    println!();
    println!("Available methods:");
    println!("  zapier.trigger          - Trigger Zap via webhook URL");
    println!("  zapier.trigger_named    - Trigger registered Zap by name");
    println!("  zapier.register_webhook - Register webhook with alias");
    println!("  zapier.list_webhooks    - List registered webhooks");
    println!("  zapier.remove_webhook   - Remove registered webhook");
    println!("  zapier.nla_actions      - List NLA actions");
    println!("  zapier.nla_execute      - Execute NLA action");
    println!();
    println!("Test with:");
    println!("  fgp call zapier.list_webhooks");
    println!();

    if foreground {
        tracing_subscriber::fmt()
            .with_env_filter("fgp_zapier=debug,fgp_daemon=debug")
            .init();

        let service = ZapierService::new().context("Failed to create ZapierService")?;
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
                    .with_env_filter("fgp_zapier=debug,fgp_daemon=debug")
                    .init();

                let service = ZapierService::new().context("Failed to create ZapierService")?;
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

    if !pid_matches_process(pid, "fgp-zapier") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-zapier daemon (PID: {})...", pid);

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
