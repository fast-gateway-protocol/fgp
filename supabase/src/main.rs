//! FGP daemon for Supabase operations.
//!
//! Provides fast access to Supabase services: Database, Auth, Storage, Functions, and Vectors.
//! ~40-120x faster than MCP stdio servers.
//!
//! # Usage
//! ```bash
//! fgp-supabase start           # Start daemon in background
//! fgp-supabase start -f        # Start in foreground
//! fgp-supabase stop            # Stop daemon
//! fgp-supabase status          # Check daemon status
//! ```
//!
//! # Authentication
//! Environment variables:
//! - SUPABASE_URL: Project URL (e.g., https://xxx.supabase.co)
//! - SUPABASE_KEY: Anon/public key for client operations
//! - SUPABASE_SERVICE_KEY: Service role key for admin operations (optional)
//!
//! # Methods
//! ## Database
//! - `supabase.sql` - Execute raw SQL query
//! - `supabase.select` - Select from table
//! - `supabase.insert` - Insert rows
//! - `supabase.update` - Update rows
//! - `supabase.delete` - Delete rows
//! - `supabase.rpc` - Call database function
//!
//! ## Auth
//! - `supabase.auth.signup` - Create user
//! - `supabase.auth.signin` - Sign in user
//! - `supabase.auth.signout` - Sign out user
//! - `supabase.auth.user` - Get user by token
//!
//! ## Storage
//! - `supabase.storage.buckets` - List buckets
//! - `supabase.storage.list` - List files
//! - `supabase.storage.upload` - Upload file
//! - `supabase.storage.download` - Download file
//! - `supabase.storage.delete` - Delete file
//! - `supabase.storage.signed_url` - Get signed URL
//!
//! ## Functions
//! - `supabase.functions.invoke` - Invoke edge function
//!
//! ## Vectors
//! - `supabase.vectors.search` - Similarity search
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

use crate::service::SupabaseService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/supabase/daemon.sock";

#[derive(Parser)]
#[command(name = "fgp-supabase")]
#[command(about = "FGP daemon for Supabase (Auth, Storage, Functions, Vectors, SQL)")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FGP daemon
    Start {
        /// Socket path (default: ~/.fgp/services/supabase/daemon.sock)
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

    println!("Starting fgp-supabase daemon...");
    println!("Socket: {}", socket_path);
    println!();
    println!("Available methods:");
    println!("  Database:");
    println!("    supabase.sql           - Execute raw SQL query");
    println!("    supabase.select        - Select from table");
    println!("    supabase.insert        - Insert rows");
    println!("    supabase.update        - Update rows");
    println!("    supabase.delete        - Delete rows");
    println!("    supabase.rpc           - Call database function");
    println!();
    println!("  Auth:");
    println!("    supabase.auth.signup   - Create user");
    println!("    supabase.auth.signin   - Sign in user");
    println!("    supabase.auth.signout  - Sign out user");
    println!("    supabase.auth.user     - Get user by token");
    println!();
    println!("  Storage:");
    println!("    supabase.storage.buckets    - List buckets");
    println!("    supabase.storage.list       - List files in bucket");
    println!("    supabase.storage.upload     - Upload file");
    println!("    supabase.storage.download   - Download file");
    println!("    supabase.storage.delete     - Delete file");
    println!("    supabase.storage.signed_url - Get signed URL");
    println!();
    println!("  Functions:");
    println!("    supabase.functions.invoke   - Invoke edge function");
    println!();
    println!("  Vectors:");
    println!("    supabase.vectors.search     - Similarity search");
    println!();
    println!("Test with:");
    println!("  fgp call supabase.select -p '{{\"table\": \"users\", \"limit\": 5}}'");
    println!();

    if foreground {
        // Foreground mode - initialize logging and run directly
        tracing_subscriber::fmt()
            .with_env_filter("fgp_supabase=debug,fgp_daemon=debug")
            .init();

        let service = SupabaseService::new().context("Failed to create SupabaseService")?;
        let server =
            FgpServer::new(service, &socket_path).context("Failed to create FGP server")?;
        server.serve().context("Server error")?;
    } else {
        // Background mode - daemonize first, THEN create service
        use daemonize::Daemonize;

        let daemonize = Daemonize::new()
            .pid_file(&pid_file)
            .working_directory("/tmp");

        match daemonize.start() {
            Ok(_) => {
                // Child process: initialize logging and run server
                tracing_subscriber::fmt()
                    .with_env_filter("fgp_supabase=debug,fgp_daemon=debug")
                    .init();

                let service = SupabaseService::new().context("Failed to create SupabaseService")?;
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

    // Read PID
    let pid_str = std::fs::read_to_string(&pid_file)
        .context("Failed to read PID file - daemon may not be running")?;
    let pid: i32 = pid_str.trim().parse().context("Invalid PID in file")?;

    if !pid_matches_process(pid, "fgp-supabase") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-supabase daemon (PID: {})...", pid);

    // Send SIGTERM
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    // Wait a moment for cleanup
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Cleanup files
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

    // Check if socket exists
    if !Path::new(&socket_path).exists() {
        println!("Status: NOT RUNNING");
        println!("Socket {} does not exist", socket_path);
        return Ok(());
    }

    // Try to connect and send health check
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            // Send health request
            let request = r#"{"id":"status","v":1,"method":"health","params":{}}"#;
            writeln!(stream, "{}", request)?;
            stream.flush()?;

            // Read response
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
