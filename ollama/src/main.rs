//! FGP daemon for local Ollama LLM operations.
//!
//! Provides fast access to local Ollama models for text generation,
//! chat completion, and embeddings.
//!
//! # Usage
//! ```bash
//! fgp-ollama start           # Start daemon in background
//! fgp-ollama start -f        # Start in foreground
//! fgp-ollama stop            # Stop daemon
//! fgp-ollama status          # Check daemon status
//! ```
//!
//! # Configuration
//! Host resolution order:
//! 1. OLLAMA_HOST environment variable
//! 2. Default: http://localhost:11434
//!
//! # Methods
//! - `ollama.generate` - Text generation with a prompt
//! - `ollama.chat` - Chat completion with message history
//! - `ollama.embed` - Generate embeddings for text
//! - `ollama.list` - List installed models
//! - `ollama.show` - Show model details
//! - `ollama.pull` - Pull a model from the registry
//! - `ollama.health` - Check if Ollama is running
//!
//! # Test
//! ```bash
//! fgp call ollama.health
//! fgp call ollama.list
//! fgp call ollama.generate -p '{"model": "llama2", "prompt": "Hello!"}'
//! fgp call ollama.chat -p '{"model": "llama2", "messages": [{"role": "user", "content": "Hi"}]}'
//! ```

mod api;
mod models;
mod service;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fgp_daemon::{cleanup_socket, FgpServer};
use std::path::Path;
use std::process::Command;

use crate::service::OllamaService;

const DEFAULT_SOCKET: &str = "~/.fgp/services/ollama/daemon.sock";

#[derive(Parser)]
#[command(name = "fgp-ollama")]
#[command(about = "FGP daemon for local Ollama LLM operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the FGP daemon
    Start {
        /// Socket path (default: ~/.fgp/services/ollama/daemon.sock)
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

    println!("Starting fgp-ollama daemon...");
    println!("Socket: {}", socket_path);
    println!();
    println!("Available methods:");
    println!("  ollama.generate  - Text generation with a prompt");
    println!("  ollama.chat      - Chat completion with message history");
    println!("  ollama.embed     - Generate embeddings for text");
    println!("  ollama.list      - List installed models");
    println!("  ollama.show      - Show model details");
    println!("  ollama.pull      - Pull a model from the registry");
    println!("  ollama.health    - Check if Ollama is running");
    println!();
    println!("Test with:");
    println!("  fgp call ollama.health");
    println!("  fgp call ollama.list");
    println!();

    if foreground {
        // Foreground mode - initialize logging and run directly
        tracing_subscriber::fmt()
            .with_env_filter("fgp_ollama=debug,fgp_daemon=debug")
            .init();

        let service = OllamaService::new(None).context("Failed to create OllamaService")?;
        let server =
            FgpServer::new(service, &socket_path).context("Failed to create FGP server")?;
        server.serve().context("Server error")?;
    } else {
        // Background mode - daemonize first, THEN create service
        // Tokio runtime must be created AFTER fork
        use daemonize::Daemonize;

        let daemonize = Daemonize::new()
            .pid_file(&pid_file)
            .working_directory("/tmp");

        match daemonize.start() {
            Ok(_) => {
                // Child process: initialize logging and run server
                tracing_subscriber::fmt()
                    .with_env_filter("fgp_ollama=debug,fgp_daemon=debug")
                    .init();

                let service = OllamaService::new(None).context("Failed to create OllamaService")?;
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

    if !pid_matches_process(pid, "fgp-ollama") {
        anyhow::bail!("Refusing to stop PID {}: unexpected process", pid);
    }

    println!("Stopping fgp-ollama daemon (PID: {})...", pid);

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
