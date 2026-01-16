//! FGP daemon for Linear issue tracking operations.
//!
//! # Usage
//! ```bash
//! fgp-linear start           # Start daemon in background
//! fgp-linear start -f        # Start in foreground
//! fgp-linear stop            # Stop daemon
//! fgp-linear status          # Check daemon status
//! fgp-linear me              # Quick: get current user (no daemon)
//! fgp-linear issues          # Quick: list issues (no daemon)
//! ```

mod client;
mod service;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fgp_daemon::{cleanup_socket, FgpServer};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::client::LinearClient;
use crate::service::LinearService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/linear/daemon.sock";

/// Linear credentials config file structure.
#[derive(Debug, Deserialize, Serialize)]
struct LinearCredentials {
    api_key: String,
}

/// Resolve Linear API key from environment or config.
fn get_api_key() -> Result<String> {
    // Try LINEAR_API_KEY env var first
    if let Ok(key) = std::env::var("LINEAR_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // Fall back to config file
    let config_path = shellexpand::tilde("~/.fgp/auth/linear/credentials.json").to_string();
    if let Ok(config_str) = std::fs::read_to_string(&config_path) {
        let creds: LinearCredentials = serde_json::from_str(&config_str)
            .context("Failed to parse Linear credentials")?;
        return Ok(creds.api_key);
    }

    anyhow::bail!(
        "No Linear API key found.\n\
         Set LINEAR_API_KEY env var or create ~/.fgp/auth/linear/credentials.json\n\
         Get your API key from: https://linear.app/settings/api"
    )
}

#[derive(Parser)]
#[command(name = "fgp-linear")]
#[command(about = "FGP daemon for Linear issue tracking operations")]
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

        /// Run in foreground
        #[arg(short, long)]
        foreground: bool,
    },

    /// Stop the running daemon
    Stop {
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },

    /// Check daemon status
    Status {
        #[arg(short, long, default_value = DEFAULT_SOCKET)]
        socket: String,
    },

    /// Get current user info (quick, no daemon)
    Me,

    /// List teams (quick, no daemon)
    Teams,

    /// List issues (quick, no daemon)
    Issues {
        /// Team key filter
        #[arg(short, long)]
        team: Option<String>,

        /// State filter
        #[arg(short, long)]
        state: Option<String>,

        /// Limit results
        #[arg(short, long, default_value = "10")]
        limit: i32,
    },

    /// Search issues (quick, no daemon)
    Search {
        /// Search query
        query: String,

        /// Limit results
        #[arg(short, long, default_value = "10")]
        limit: i32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { socket, foreground } => cmd_start(socket, foreground),
        Commands::Stop { socket } => cmd_stop(socket),
        Commands::Status { socket } => cmd_status(socket),
        Commands::Me => cmd_me(),
        Commands::Teams => cmd_teams(),
        Commands::Issues { team, state, limit } => cmd_issues(team, state, limit),
        Commands::Search { query, limit } => cmd_search(query, limit),
    }
}

fn cmd_start(socket: String, foreground: bool) -> Result<()> {
    let socket_path = shellexpand::tilde(&socket).to_string();

    if let Some(parent) = Path::new(&socket_path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    let api_key = get_api_key()?;
    let pid_file = format!("{}.pid", socket_path);

    println!("Starting fgp-linear daemon...");
    println!("Socket: {}", socket_path);

    if foreground {
        tracing_subscriber::fmt()
            .with_env_filter("fgp_linear=debug,fgp_daemon=debug")
            .init();

        let service = LinearService::new(api_key).context("Failed to create LinearService")?;
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
                    .with_env_filter("fgp_linear=debug,fgp_daemon=debug")
                    .init();

                let service =
                    LinearService::new(api_key).context("Failed to create LinearService")?;
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

    if !pid_matches_process(pid, "fgp-linear") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-linear daemon (PID: {})...", pid);

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

fn cmd_me() -> Result<()> {
    let api_key = get_api_key()?;
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let client = LinearClient::new(api_key)?;
        client.me().await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn cmd_teams() -> Result<()> {
    let api_key = get_api_key()?;
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let client = LinearClient::new(api_key)?;
        client.teams().await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn cmd_issues(team: Option<String>, state: Option<String>, limit: i32) -> Result<()> {
    let api_key = get_api_key()?;
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let client = LinearClient::new(api_key)?;
        client
            .issues(team.as_deref(), state.as_deref(), None, limit)
            .await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn cmd_search(query: String, limit: i32) -> Result<()> {
    let api_key = get_api_key()?;
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let client = LinearClient::new(api_key)?;
        client.search(&query, limit).await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
