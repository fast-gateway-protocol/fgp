//! fgp-notes - Fast Notes gateway for macOS
//!
//! Direct SQLite access to Notes library database with protobuf parsing.
//! Requires Full Disk Access permission.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::process::ExitCode;

mod daemon;
mod db;

use daemon::service::NotesService;
use fgp_daemon::service::FgpService;

/// Fast Notes gateway for macOS - direct SQLite queries with protobuf parsing.
#[derive(Parser, Debug)]
#[command(name = "fgp-notes")]
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
    /// List all notes
    List {
        /// Maximum notes to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Get recently modified notes
    Recent {
        /// Days to look back
        #[arg(short, long, default_value_t = 7)]
        days: u32,

        /// Maximum notes to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Search notes by title/content
    Search {
        /// Search query
        query: String,

        /// Maximum notes to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Read a specific note by ID
    Read {
        /// Note ID
        id: i64,
    },

    /// Get notes in a folder
    ByFolder {
        /// Folder name
        folder: String,

        /// Maximum notes to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Get pinned notes
    Pinned {
        /// Maximum notes to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// List folders
    Folders {
        /// Maximum folders to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Get library statistics
    Stats,

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
            let service = NotesService::new()?;
            let socket_path = fgp_daemon::service_socket_path("notes");

            println!("Starting Notes daemon at {}", socket_path.display());

            let server = fgp_daemon::FgpServer::new(service, &socket_path)?;
            server.serve()?;
            Ok(())
        }

        Command::Stop => {
            fgp_daemon::stop_service("notes")?;
            println!("Notes daemon stopped");
            Ok(())
        }

        Command::Health => {
            let service = NotesService::new()?;
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
            let service = NotesService::new()?;
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

        Command::List { limit } => {
            let service = NotesService::new()?;
            let mut params = HashMap::new();
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("list", params)?;
            print_notes(&result, cli.json)?;
            Ok(())
        }

        Command::Recent { days, limit } => {
            let service = NotesService::new()?;
            let mut params = HashMap::new();
            params.insert("days".to_string(), serde_json::json!(days));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("recent", params)?;
            print_notes(&result, cli.json)?;
            Ok(())
        }

        Command::Search { query, limit } => {
            let service = NotesService::new()?;
            let mut params = HashMap::new();
            params.insert("query".to_string(), serde_json::json!(query));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("search", params)?;
            print_notes(&result, cli.json)?;
            Ok(())
        }

        Command::Read { id } => {
            let service = NotesService::new()?;
            let mut params = HashMap::new();
            params.insert("id".to_string(), serde_json::json!(id));

            let result = service.dispatch("read", params)?;
            print_note(&result, cli.json)?;
            Ok(())
        }

        Command::ByFolder { folder, limit } => {
            let service = NotesService::new()?;
            let mut params = HashMap::new();
            params.insert("folder".to_string(), serde_json::json!(folder));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("by_folder", params)?;
            print_notes(&result, cli.json)?;
            Ok(())
        }

        Command::Pinned { limit } => {
            let service = NotesService::new()?;
            let mut params = HashMap::new();
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("pinned", params)?;
            print_notes(&result, cli.json)?;
            Ok(())
        }

        Command::Folders { limit } => {
            let service = NotesService::new()?;
            let mut params = HashMap::new();
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("folders", params)?;
            print_folders(&result, cli.json)?;
            Ok(())
        }

        Command::Stats => {
            let service = NotesService::new()?;
            let result = service.dispatch("stats", HashMap::new())?;
            print_stats(&result, cli.json)?;
            Ok(())
        }
    }
}

fn print_notes(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else if let Some(notes) = result.get("notes").and_then(|v| v.as_array()) {
        for note in notes {
            let title = note
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("(untitled)");
            let id = note.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let modified = note.get("modified").and_then(|v| v.as_str()).unwrap_or("");
            let folder = note.get("folder").and_then(|v| v.as_str());
            let is_pinned = note.get("is_pinned").and_then(|v| v.as_bool()).unwrap_or(false);
            let has_checklist = note
                .get("has_checklist")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let pinned_icon = if is_pinned { "📌 " } else { "" };
            let checklist_icon = if has_checklist { "☑️ " } else { "" };

            println!("{}{}[{}] {}", pinned_icon, checklist_icon, id, title);
            if let Some(f) = folder {
                print!("  📁 {}", f);
            }
            if !modified.is_empty() {
                print!("  📅 {}", &modified[..10]);
            }
            println!();

            // Show snippet preview
            if let Some(snippet) = note.get("snippet").and_then(|v| v.as_str()) {
                let preview: String = snippet.chars().take(80).collect();
                if !preview.is_empty() {
                    println!("  {}", preview);
                }
            }
            println!();
        }
        if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
            println!("Total: {} notes", count);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(result)?);
    }
    Ok(())
}

fn print_note(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else if let Some(note) = result.get("note") {
        let title = note
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("(untitled)");
        let id = note.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let created = note.get("created").and_then(|v| v.as_str()).unwrap_or("");
        let modified = note.get("modified").and_then(|v| v.as_str()).unwrap_or("");
        let folder = note.get("folder").and_then(|v| v.as_str());
        let body = note.get("body").and_then(|v| v.as_str());

        println!("=== {} ===", title);
        println!("ID: {}", id);
        if let Some(f) = folder {
            println!("Folder: {}", f);
        }
        if !created.is_empty() {
            println!("Created: {}", created);
        }
        if !modified.is_empty() {
            println!("Modified: {}", modified);
        }
        println!();

        if let Some(text) = body {
            println!("{}", text);
        } else if let Some(snippet) = note.get("snippet").and_then(|v| v.as_str()) {
            println!("{}", snippet);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(result)?);
    }
    Ok(())
}

fn print_folders(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else if let Some(folders) = result.get("folders").and_then(|v| v.as_array()) {
        for (i, folder) in folders.iter().enumerate() {
            let name = folder
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("(unnamed)");
            let count = folder.get("note_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let id = folder.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            println!("{}. {} ({} notes) [id: {}]", i + 1, name, count, id);
        }
        if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
            println!("\nTotal: {} folders", count);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(result)?);
    }
    Ok(())
}

fn print_stats(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else {
        let total = result
            .get("total_notes")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let folders = result
            .get("total_folders")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let checklists = result
            .get("notes_with_checklists")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let pinned = result
            .get("pinned_notes")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        println!("Notes Library Statistics");
        println!("  📝 Total notes:       {:>6}", total);
        println!("  📁 Folders:           {:>6}", folders);
        println!("  ☑️  With checklists:   {:>6}", checklists);
        println!("  📌 Pinned:            {:>6}", pinned);
    }
    Ok(())
}
