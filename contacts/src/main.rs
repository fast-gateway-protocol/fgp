//! fgp-contacts - Fast Contacts gateway for macOS
//!
//! Direct SQLite access to AddressBook database.
//! Requires Full Disk Access permission.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::process::ExitCode;

mod daemon;
mod db;

use daemon::service::ContactsService;
use fgp_daemon::service::FgpService;

/// Fast Contacts gateway for macOS - direct SQLite queries.
#[derive(Parser, Debug)]
#[command(name = "fgp-contacts")]
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
    /// List all contacts
    List {
        /// Maximum contacts to return
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
    },

    /// Search contacts by name
    Search {
        /// Search query
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// Find contact by email address
    ByEmail {
        /// Email address to search
        email: String,
    },

    /// Find contact by phone number
    ByPhone {
        /// Phone number to search
        phone: String,
    },

    /// Get recently modified contacts
    Recent {
        /// Number of days to look back
        #[arg(short, long, default_value_t = 30)]
        days: u32,

        /// Maximum contacts to return
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// Get contact statistics
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
    // Initialize logging
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
            let service = ContactsService::new()?;
            let socket_path = fgp_daemon::service_socket_path("contacts");

            println!("Starting Contacts daemon at {}", socket_path.display());

            let server = fgp_daemon::FgpServer::new(service, &socket_path)?;
            server.serve()?;
            Ok(())
        }

        Command::Stop => {
            fgp_daemon::stop_service("contacts")?;
            println!("Contacts daemon stopped");
            Ok(())
        }

        Command::Health => {
            let service = ContactsService::new()?;
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
            let service = ContactsService::new()?;
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
                            println!("    {}{}: {}{}", param.name, req, param.param_type, default);
                        }
                    }
                    println!();
                }
            }
            Ok(())
        }

        Command::List { limit } => {
            let service = ContactsService::new()?;
            let mut params = HashMap::new();
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("list", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Search { query, limit } => {
            let service = ContactsService::new()?;
            let mut params = HashMap::new();
            params.insert("query".to_string(), serde_json::json!(query));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("search", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::ByEmail { email } => {
            let service = ContactsService::new()?;
            let mut params = HashMap::new();
            params.insert("email".to_string(), serde_json::json!(email));

            let result = service.dispatch("by_email", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::ByPhone { phone } => {
            let service = ContactsService::new()?;
            let mut params = HashMap::new();
            params.insert("phone".to_string(), serde_json::json!(phone));

            let result = service.dispatch("by_phone", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Recent { days, limit } => {
            let service = ContactsService::new()?;
            let mut params = HashMap::new();
            params.insert("days".to_string(), serde_json::json!(days));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("recent", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Stats => {
            let service = ContactsService::new()?;
            let result = service.dispatch("stats", HashMap::new())?;
            print_result(&result, cli.json)?;
            Ok(())
        }
    }
}

fn print_result(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else {
        // Pretty print based on result type
        if let Some(contacts) = result.get("contacts").and_then(|v| v.as_array()) {
            for contact in contacts {
                let name = contact.get("name").and_then(|v| v.as_str()).unwrap_or("(No Name)");
                let org = contact
                    .get("organization")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty());

                print!("{}", name);
                if let Some(org) = org {
                    print!(" ({})", org);
                }
                println!();

                // Emails
                if let Some(emails) = contact.get("emails").and_then(|v| v.as_array()) {
                    for email in emails {
                        let addr = email.get("address").and_then(|v| v.as_str()).unwrap_or("");
                        let label = email.get("label").and_then(|v| v.as_str()).unwrap_or("email");
                        println!("  📧 {}: {}", label, addr);
                    }
                }

                // Phones
                if let Some(phones) = contact.get("phones").and_then(|v| v.as_array()) {
                    for phone in phones {
                        let num = phone.get("number").and_then(|v| v.as_str()).unwrap_or("");
                        let label = phone.get("label").and_then(|v| v.as_str()).unwrap_or("phone");
                        println!("  📱 {}: {}", label, num);
                    }
                }

                println!();
            }
            if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
                println!("Total: {} contacts", count);
            }
        } else if result.get("found").is_some() {
            // Single contact lookup result
            let found = result.get("found").and_then(|v| v.as_bool()).unwrap_or(false);
            if found {
                if let Some(contact) = result.get("contact") {
                    let name = contact.get("name").and_then(|v| v.as_str()).unwrap_or("(No Name)");
                    println!("Found: {}", name);

                    if let Some(emails) = contact.get("emails").and_then(|v| v.as_array()) {
                        for email in emails {
                            let addr = email.get("address").and_then(|v| v.as_str()).unwrap_or("");
                            println!("  📧 {}", addr);
                        }
                    }

                    if let Some(phones) = contact.get("phones").and_then(|v| v.as_array()) {
                        for phone in phones {
                            let num = phone.get("number").and_then(|v| v.as_str()).unwrap_or("");
                            println!("  📱 {}", num);
                        }
                    }
                }
            } else {
                println!("Not found");
            }
        } else if result.get("total_contacts").is_some() {
            // Stats output
            let total = result.get("total_contacts").and_then(|v| v.as_i64()).unwrap_or(0);
            let with_email = result.get("with_email").and_then(|v| v.as_i64()).unwrap_or(0);
            let with_phone = result.get("with_phone").and_then(|v| v.as_i64()).unwrap_or(0);
            let with_org = result.get("with_organization").and_then(|v| v.as_i64()).unwrap_or(0);
            let groups = result.get("total_groups").and_then(|v| v.as_i64()).unwrap_or(0);

            println!("Contacts Statistics");
            println!("  Total contacts:     {}", total);
            println!("  With email:         {}", with_email);
            println!("  With phone:         {}", with_phone);
            println!("  With organization:  {}", with_org);
            println!("  Contact groups:     {}", groups);
        } else {
            // Fallback to JSON
            println!("{}", serde_json::to_string_pretty(result)?);
        }
    }
    Ok(())
}
