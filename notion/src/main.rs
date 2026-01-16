//! FGP daemon for Notion pages, databases, and blocks.
//!
//! # Usage
//! ```bash
//! fgp-notion start           # Start daemon in background
//! fgp-notion start -f        # Start in foreground
//! fgp-notion stop            # Stop daemon
//! fgp-notion status          # Check daemon status
//! fgp-notion search "notes"  # Quick: search pages (no daemon)
//! ```

mod client;
mod service;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fgp_daemon::{cleanup_socket, FgpServer};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::client::NotionClient;
use crate::service::NotionService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/notion/daemon.sock";

/// Notion credentials config file structure.
#[derive(Debug, Deserialize, Serialize)]
struct NotionCredentials {
    api_key: String,
}

/// Resolve Notion API key from environment or config.
fn get_api_key() -> Result<String> {
    // Try NOTION_API_KEY env var first
    if let Ok(key) = std::env::var("NOTION_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // Also try NOTION_TOKEN (common alternative)
    if let Ok(key) = std::env::var("NOTION_TOKEN") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // Fall back to config file
    let config_path = shellexpand::tilde("~/.fgp/auth/notion/credentials.json").to_string();
    if let Ok(config_str) = std::fs::read_to_string(&config_path) {
        let creds: NotionCredentials = serde_json::from_str(&config_str)
            .context("Failed to parse Notion credentials")?;
        return Ok(creds.api_key);
    }

    anyhow::bail!(
        "No Notion API key found.\n\
         Set NOTION_API_KEY env var or create ~/.fgp/auth/notion/credentials.json\n\
         Create an integration at: https://www.notion.so/my-integrations"
    )
}

#[derive(Parser)]
#[command(name = "fgp-notion")]
#[command(about = "FGP daemon for Notion pages, databases, and blocks")]
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

    /// Get current bot info (quick, no daemon)
    Me,

    /// Search pages and databases (quick, no daemon)
    Search {
        /// Search query
        query: String,

        /// Filter by type (page or database)
        #[arg(short, long)]
        filter: Option<String>,

        /// Limit results
        #[arg(short, long, default_value = "10")]
        limit: i32,
    },

    /// Get page content (quick, no daemon)
    Page {
        /// Page ID
        page_id: String,
    },

    /// Get blocks for a page (quick, no daemon)
    Blocks {
        /// Page or block ID
        block_id: String,

        /// Fetch nested blocks recursively
        #[arg(short, long)]
        recursive: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { socket, foreground } => cmd_start(socket, foreground),
        Commands::Stop { socket } => cmd_stop(socket),
        Commands::Status { socket } => cmd_status(socket),
        Commands::Me => cmd_me(),
        Commands::Search { query, filter, limit } => cmd_search(query, filter, limit),
        Commands::Page { page_id } => cmd_page(page_id),
        Commands::Blocks { block_id, recursive } => cmd_blocks(block_id, recursive),
    }
}

fn cmd_start(socket: String, foreground: bool) -> Result<()> {
    let socket_path = shellexpand::tilde(&socket).to_string();

    if let Some(parent) = Path::new(&socket_path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    let api_key = get_api_key()?;
    let pid_file = format!("{}.pid", socket_path);

    println!("Starting fgp-notion daemon...");
    println!("Socket: {}", socket_path);

    if foreground {
        tracing_subscriber::fmt()
            .with_env_filter("fgp_notion=debug,fgp_daemon=debug")
            .init();

        let service = NotionService::new(api_key).context("Failed to create NotionService")?;
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
                    .with_env_filter("fgp_notion=debug,fgp_daemon=debug")
                    .init();

                let service =
                    NotionService::new(api_key).context("Failed to create NotionService")?;
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

    if !pid_matches_process(pid, "fgp-notion") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-notion daemon (PID: {})...", pid);

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
        let client = NotionClient::new(api_key)?;
        client.me().await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn cmd_search(query: String, filter: Option<String>, limit: i32) -> Result<()> {
    let api_key = get_api_key()?;
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let client = NotionClient::new(api_key)?;
        client.search(Some(&query), filter.as_deref(), limit).await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn cmd_page(page_id: String) -> Result<()> {
    let api_key = get_api_key()?;
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let client = NotionClient::new(api_key)?;
        client.page(&page_id).await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn cmd_blocks(block_id: String, recursive: bool) -> Result<()> {
    let api_key = get_api_key()?;
    let rt = tokio::runtime::Runtime::new()?;

    let result = rt.block_on(async {
        let client = NotionClient::new(api_key)?;
        client.blocks(&block_id, recursive).await
    })?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
