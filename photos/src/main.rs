//! fgp-photos - Fast Photos gateway for macOS
//!
//! Direct SQLite access to Photos library database.
//! Requires Full Disk Access permission.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::process::ExitCode;

mod daemon;
mod db;

use daemon::service::PhotosService;
use fgp_daemon::service::FgpService;

/// Fast Photos gateway for macOS - direct SQLite queries.
#[derive(Parser, Debug)]
#[command(name = "fgp-photos")]
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
    /// Get recent photos/videos
    Recent {
        /// Number of days to look back
        #[arg(short, long, default_value_t = 30)]
        days: u32,

        /// Maximum items to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,

        /// Filter by type: photo, video
        #[arg(short, long)]
        kind: Option<String>,
    },

    /// Get favorited photos/videos
    Favorites {
        /// Maximum items to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Search photos by date range
    ByDate {
        /// Start date (ISO format)
        start: String,

        /// End date (ISO format)
        end: String,

        /// Maximum items to return
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
    },

    /// Search photos near a location
    ByLocation {
        /// Latitude
        lat: f64,

        /// Longitude
        lon: f64,

        /// Search radius in km
        #[arg(short, long, default_value_t = 1.0)]
        radius: f64,

        /// Maximum items to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// List photo albums
    Albums {
        /// Maximum albums to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Get photos in an album
    AlbumPhotos {
        /// Album ID
        album_id: i64,

        /// Maximum items to return
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
    },

    /// List recognized people
    People {
        /// Maximum people to return
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },

    /// Get photos of a person
    PersonPhotos {
        /// Person ID
        person_id: i64,

        /// Maximum items to return
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
            let service = PhotosService::new()?;
            let socket_path = fgp_daemon::service_socket_path("photos");

            println!("Starting Photos daemon at {}", socket_path.display());

            let server = fgp_daemon::FgpServer::new(service, &socket_path)?;
            server.serve()?;
            Ok(())
        }

        Command::Stop => {
            fgp_daemon::stop_service("photos")?;
            println!("Photos daemon stopped");
            Ok(())
        }

        Command::Health => {
            let service = PhotosService::new()?;
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
            let service = PhotosService::new()?;
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

        Command::Recent { days, limit, kind } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("days".to_string(), serde_json::json!(days));
            params.insert("limit".to_string(), serde_json::json!(limit));
            if let Some(k) = kind {
                params.insert("kind".to_string(), serde_json::json!(k));
            }

            let result = service.dispatch("recent", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Favorites { limit } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("favorites", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::ByDate { start, end, limit } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("start".to_string(), serde_json::json!(start));
            params.insert("end".to_string(), serde_json::json!(end));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("by_date", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::ByLocation {
            lat,
            lon,
            radius,
            limit,
        } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("lat".to_string(), serde_json::json!(lat));
            params.insert("lon".to_string(), serde_json::json!(lon));
            params.insert("radius".to_string(), serde_json::json!(radius));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("by_location", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Albums { limit } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("albums", params)?;
            print_albums(&result, cli.json)?;
            Ok(())
        }

        Command::AlbumPhotos { album_id, limit } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("album_id".to_string(), serde_json::json!(album_id));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("album_photos", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::People { limit } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("people", params)?;
            print_people(&result, cli.json)?;
            Ok(())
        }

        Command::PersonPhotos { person_id, limit } => {
            let service = PhotosService::new()?;
            let mut params = HashMap::new();
            params.insert("person_id".to_string(), serde_json::json!(person_id));
            params.insert("limit".to_string(), serde_json::json!(limit));

            let result = service.dispatch("person_photos", params)?;
            print_result(&result, cli.json)?;
            Ok(())
        }

        Command::Stats => {
            let service = PhotosService::new()?;
            let result = service.dispatch("stats", HashMap::new())?;
            print_stats(&result, cli.json)?;
            Ok(())
        }
    }
}

fn print_result(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else if let Some(assets) = result.get("assets").and_then(|v| v.as_array()) {
        for asset in assets {
            let filename = asset
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("(unknown)");
            let date = asset.get("date_created").and_then(|v| v.as_str()).unwrap_or("");
            let kind = asset
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let favorite = asset.get("favorite").and_then(|v| v.as_bool()).unwrap_or(false);
            let fav_icon = if favorite { "⭐ " } else { "" };

            println!("{}{} ({})", fav_icon, filename, kind);
            println!("  📅 {}", date);

            if let (Some(lat), Some(lon)) = (
                asset.get("latitude").and_then(|v| v.as_f64()),
                asset.get("longitude").and_then(|v| v.as_f64()),
            ) {
                println!("  📍 {:.4}, {:.4}", lat, lon);
            }
            println!();
        }
        if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
            println!("Total: {} items", count);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(result)?);
    }
    Ok(())
}

fn print_albums(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else if let Some(albums) = result.get("albums").and_then(|v| v.as_array()) {
        for (i, album) in albums.iter().enumerate() {
            let title = album.get("title").and_then(|v| v.as_str()).unwrap_or("(unnamed)");
            let count = album.get("asset_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let id = album.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            println!("{}. {} ({} items) [id: {}]", i + 1, title, count, id);
        }
        if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
            println!("\nTotal: {} albums", count);
        }
    } else {
        println!("{}", serde_json::to_string_pretty(result)?);
    }
    Ok(())
}

fn print_people(result: &serde_json::Value, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else if let Some(people) = result.get("people").and_then(|v| v.as_array()) {
        for (i, person) in people.iter().enumerate() {
            let name = person
                .get("display_name")
                .and_then(|v| v.as_str())
                .unwrap_or("(unnamed)");
            let count = person.get("face_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let id = person.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            println!("{}. {} ({} photos) [id: {}]", i + 1, name, count, id);
        }
        if let Some(count) = result.get("count").and_then(|v| v.as_i64()) {
            println!("\nTotal: {} people", count);
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
        let total = result.get("total_assets").and_then(|v| v.as_i64()).unwrap_or(0);
        let photos = result.get("photos").and_then(|v| v.as_i64()).unwrap_or(0);
        let videos = result.get("videos").and_then(|v| v.as_i64()).unwrap_or(0);
        let favorites = result.get("favorites").and_then(|v| v.as_i64()).unwrap_or(0);
        let hidden = result.get("hidden").and_then(|v| v.as_i64()).unwrap_or(0);
        let with_location = result.get("with_location").and_then(|v| v.as_i64()).unwrap_or(0);
        let albums = result.get("albums").and_then(|v| v.as_i64()).unwrap_or(0);
        let people = result.get("people").and_then(|v| v.as_i64()).unwrap_or(0);

        println!("Photos Library Statistics");
        println!("  Total assets:    {:>8}", total);
        println!("  📷 Photos:       {:>8}", photos);
        println!("  🎬 Videos:       {:>8}", videos);
        println!("  ⭐ Favorites:    {:>8}", favorites);
        println!("  👁️ Hidden:       {:>8}", hidden);
        println!("  📍 With location:{:>8}", with_location);
        println!("  📁 Albums:       {:>8}", albums);
        println!("  👤 People:       {:>8}", people);
    }
    Ok(())
}
