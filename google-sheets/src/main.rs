//! FGP daemon for Google Sheets spreadsheet operations.
//!
//! Provides fast spreadsheet operations for Google Sheets.
//! ~40-80x faster than MCP stdio servers.
//!
//! # Usage
//! ```bash
//! fgp-google-sheets start           # Start daemon in background
//! fgp-google-sheets start -f        # Start in foreground
//! fgp-google-sheets stop            # Stop daemon
//! fgp-google-sheets status          # Check daemon status
//! ```
//!
//! # Authentication
//! Requires OAuth2 token at ~/.fgp/auth/google/token.json
//!
//! # Methods
//! - `sheets.get` - Get spreadsheet metadata
//! - `sheets.values` - Get values from a range
//! - `sheets.batch_get` - Get values from multiple ranges
//! - `sheets.update` - Update values in a range
//! - `sheets.append` - Append values to a range
//! - `sheets.clear` - Clear values in a range
//! - `sheets.create` - Create a new spreadsheet
//! - `sheets.add_sheet` - Add a new sheet
//! - `sheets.delete_sheet` - Delete a sheet
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

use crate::service::SheetsService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/sheets/daemon.sock";

#[derive(Parser)]
#[command(name = "fgp-google-sheets")]
#[command(about = "FGP daemon for Google Sheets spreadsheet operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FGP daemon
    Start {
        /// Socket path (default: ~/.fgp/services/sheets/daemon.sock)
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

    println!("Starting fgp-google-sheets daemon...");
    println!("Socket: {}", socket_path);
    println!();
    println!("Available methods:");
    println!("  sheets.get         - Get spreadsheet metadata");
    println!("  sheets.values      - Get values from a range");
    println!("  sheets.batch_get   - Get values from multiple ranges");
    println!("  sheets.update      - Update values in a range");
    println!("  sheets.append      - Append values to a range");
    println!("  sheets.clear       - Clear values in a range");
    println!("  sheets.create      - Create a new spreadsheet");
    println!("  sheets.add_sheet   - Add a new sheet");
    println!("  sheets.delete_sheet - Delete a sheet");
    println!();
    println!("Test with:");
    println!("  fgp call sheets.values -p '{{\"spreadsheet_id\": \"abc123\", \"range\": \"Sheet1!A1:C10\"}}'");
    println!();

    if foreground {
        tracing_subscriber::fmt()
            .with_env_filter("fgp_google_sheets=debug,fgp_daemon=debug")
            .init();

        let service = SheetsService::new().context("Failed to create SheetsService")?;
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
                    .with_env_filter("fgp_google_sheets=debug,fgp_daemon=debug")
                    .init();

                let service = SheetsService::new().context("Failed to create SheetsService")?;
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

    if !pid_matches_process(pid, "fgp-google-sheets") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-google-sheets daemon (PID: {})...", pid);

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
