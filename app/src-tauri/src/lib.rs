//! FGP Manager - Menu bar app for managing FGP daemons.
//!
//! This is a Tauri 2.x macOS menu bar app that provides a GUI for:
//! - Starting/stopping FGP daemons
//! - Monitoring daemon health
//! - Installing daemons from marketplace
//! - Configuring MCP integration for AI agents

use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_positioner::{Position, WindowExt as PositionerExt};

// Re-export fgp-daemon types
use fgp_daemon::{
    lifecycle::{fgp_services_dir, is_service_running, service_socket_path, start_service, stop_service},
    FgpClient,
};

// ============================================================================
// Types
// ============================================================================

/// Information about a daemon service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub name: String,
    pub status: String,
    pub version: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub is_running: bool,
    pub has_manifest: bool,
}

/// Health status for a daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    pub status: String,
    pub version: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub dependencies: HashMap<String, DependencyHealth>,
}

/// Health status for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub ok: bool,
    pub latency_ms: Option<f64>,
    pub message: Option<String>,
}

/// Overall tray status
#[derive(Debug, Clone, Copy, PartialEq)]
enum TrayStatus {
    Healthy,
    Degraded,
    Error,
}

/// Package info from the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPackage {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub icon: String,
    pub author: String,
    pub repository: String,
    pub methods_count: u32,
    pub featured: bool,
    pub official: bool,
    pub category: String,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub installed_version: Option<String>,
    #[serde(default)]
    pub update_available: bool,
}

/// Registry response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub schema_version: u32,
    pub updated_at: String,
    pub packages: Vec<RegistryPackage>,
    pub categories: Vec<RegistryCategory>,
}

/// Category in registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
}

/// Installation progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub package: String,
    pub step: String,
    pub progress: u32,
    pub total: u32,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// List all installed FGP daemons with their status
#[tauri::command]
async fn list_daemons() -> Result<Vec<DaemonInfo>, String> {
    let services_dir = fgp_services_dir();

    if !services_dir.exists() {
        return Ok(vec![]);
    }

    let mut daemons = Vec::new();

    let entries = fs::read_dir(&services_dir).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let has_manifest = path.join("manifest.json").exists();
        let socket_path = service_socket_path(&name);

        let (status, version, uptime, is_running) = if socket_path.exists() {
            match FgpClient::new(&socket_path) {
                Ok(client) => match client.health() {
                    Ok(response) if response.ok => {
                        let result = response.result.unwrap_or_default();
                        let version = result["version"].as_str().map(String::from);
                        let uptime = result["uptime_seconds"].as_u64();
                        let status = result["status"]
                            .as_str()
                            .unwrap_or("running")
                            .to_string();
                        (status, version, uptime, true)
                    }
                    _ => ("not_responding".into(), None, None, false),
                },
                Err(_) => ("socket_error".into(), None, None, false),
            }
        } else {
            ("stopped".into(), None, None, false)
        };

        daemons.push(DaemonInfo {
            name,
            status,
            version,
            uptime_seconds: uptime,
            is_running,
            has_manifest,
        });
    }

    daemons.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(daemons)
}

/// Get detailed health info for a specific daemon
#[tauri::command]
async fn get_daemon_health(name: String) -> Result<HealthInfo, String> {
    let socket_path = service_socket_path(&name);

    if !socket_path.exists() {
        return Err(format!("Daemon '{}' is not running", name));
    }

    let client = FgpClient::new(&socket_path).map_err(|e| e.to_string())?;
    let response = client.health().map_err(|e| e.to_string())?;

    if response.ok {
        let result = response.result.unwrap_or_default();

        // Parse dependencies if present
        let mut dependencies = HashMap::new();
        if let Some(deps) = result["dependencies"].as_object() {
            for (key, value) in deps {
                dependencies.insert(
                    key.clone(),
                    DependencyHealth {
                        ok: value["ok"].as_bool().unwrap_or(false),
                        latency_ms: value["latency_ms"].as_f64(),
                        message: value["message"].as_str().map(String::from),
                    },
                );
            }
        }

        Ok(HealthInfo {
            status: result["status"].as_str().unwrap_or("unknown").to_string(),
            version: result["version"].as_str().map(String::from),
            uptime_seconds: result["uptime_seconds"].as_u64(),
            dependencies,
        })
    } else {
        Err(response
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "Unknown error".into()))
    }
}

/// Start a daemon service
#[tauri::command]
async fn start_daemon(name: String) -> Result<(), String> {
    start_service(&name).map_err(|e| e.to_string())
}

/// Stop a daemon service
#[tauri::command]
async fn stop_daemon(name: String) -> Result<(), String> {
    stop_service(&name).map_err(|e| e.to_string())
}

/// Restart a daemon (stop then start)
#[tauri::command]
async fn restart_daemon(name: String) -> Result<(), String> {
    let _ = stop_service(&name);
    std::thread::sleep(Duration::from_millis(500));
    start_service(&name).map_err(|e| e.to_string())
}

/// Check if a specific daemon is running
#[tauri::command]
async fn is_daemon_running(name: String) -> bool {
    is_service_running(&name)
}

// ============================================================================
// Marketplace Commands
// ============================================================================

/// Fetch the package registry (bundled + remote)
#[tauri::command]
async fn fetch_registry(app: AppHandle) -> Result<Registry, String> {
    // First, try to load bundled registry
    let bundled_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources/registry.json");

    let registry_content = if bundled_path.exists() {
        fs::read_to_string(&bundled_path).map_err(|e| e.to_string())?
    } else {
        // Fallback: try to fetch from GitHub
        // For now, return error if bundled not found
        return Err("Registry not found. Please reinstall the app.".into());
    };

    let mut registry: Registry = serde_json::from_str(&registry_content)
        .map_err(|e| format!("Failed to parse registry: {}", e))?;

    // Check which packages are installed
    let services_dir = fgp_services_dir();
    for package in &mut registry.packages {
        let service_path = services_dir.join(&package.name);
        let manifest_path = service_path.join("manifest.json");

        if manifest_path.exists() {
            package.installed = true;

            // Read installed version from manifest
            if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_content) {
                    if let Some(version) = manifest["version"].as_str() {
                        package.installed_version = Some(version.to_string());
                        // Check if update is available
                        package.update_available = version != package.version;
                    }
                }
            }
        }
    }

    Ok(registry)
}

/// Get details about a specific package
#[tauri::command]
async fn get_package_details(name: String, app: AppHandle) -> Result<RegistryPackage, String> {
    let registry = fetch_registry(app).await?;

    registry
        .packages
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Package '{}' not found", name))
}

/// Install a package from the registry
#[tauri::command]
async fn install_package(name: String, app: AppHandle) -> Result<(), String> {
    // Emit progress: starting
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            package: name.clone(),
            step: "Fetching package info...".into(),
            progress: 0,
            total: 100,
        },
    );

    // Get package info
    let registry = fetch_registry(app.clone()).await?;
    let package = registry
        .packages
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Package '{}' not found", name))?;

    // For now, we'll clone from the repository
    // In production, we'd download the pre-built binary
    let services_dir = fgp_services_dir();
    let package_dir = services_dir.join(&name);

    // Emit progress: cloning
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            package: name.clone(),
            step: "Cloning repository...".into(),
            progress: 20,
            total: 100,
        },
    );

    // Clone if not exists
    if !package_dir.exists() {
        let output = std::process::Command::new("git")
            .args(["clone", &package.repository, package_dir.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to clone: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    // Emit progress: building
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            package: name.clone(),
            step: "Building daemon...".into(),
            progress: 50,
            total: 100,
        },
    );

    // Build the daemon
    let output = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&package_dir)
        .output()
        .map_err(|e| format!("Failed to build: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Emit progress: complete
    let _ = app.emit(
        "install-progress",
        InstallProgress {
            package: name.clone(),
            step: "Installation complete!".into(),
            progress: 100,
            total: 100,
        },
    );

    // Notify that daemons list should refresh
    if let Ok(daemons) = list_daemons().await {
        let _ = app.emit("daemons-updated", &daemons);
    }

    Ok(())
}

/// Uninstall a package
#[tauri::command]
async fn uninstall_package(name: String, app: AppHandle) -> Result<(), String> {
    // Stop the daemon if running
    let _ = stop_service(&name);

    // Remove the service directory
    let services_dir = fgp_services_dir();
    let package_dir = services_dir.join(&name);

    if package_dir.exists() {
        fs::remove_dir_all(&package_dir).map_err(|e| format!("Failed to remove package: {}", e))?;
    }

    // Notify that daemons list should refresh
    if let Ok(daemons) = list_daemons().await {
        let _ = app.emit("daemons-updated", &daemons);
    }

    Ok(())
}

// ============================================================================
// MCP Integration Commands
// ============================================================================

/// Information about an AI agent that can use FGP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub id: String,
    pub installed: bool,
    pub config_path: Option<String>,
    pub registered: bool,
}

/// Detect installed AI agents (Claude Code, Cursor, etc.)
#[tauri::command]
async fn detect_agents() -> Result<Vec<AgentInfo>, String> {
    let mut agents = Vec::new();
    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    // Claude Code
    let claude_code_installed = std::process::Command::new("which")
        .arg("claude")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let claude_config_path = home.join(".claude.json");
    let claude_registered = if claude_config_path.exists() {
        fs::read_to_string(&claude_config_path)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .map(|config| {
                config["mcpServers"]["fgp"].is_object()
            })
            .unwrap_or(false)
    } else {
        false
    };

    agents.push(AgentInfo {
        name: "Claude Code".into(),
        id: "claude-code".into(),
        installed: claude_code_installed,
        config_path: Some(claude_config_path.to_string_lossy().into()),
        registered: claude_registered,
    });

    // Cursor
    let cursor_config_path = home.join(".cursor/mcp.json");
    let cursor_installed = cursor_config_path.parent().map(|p| p.exists()).unwrap_or(false);
    let cursor_registered = if cursor_config_path.exists() {
        fs::read_to_string(&cursor_config_path)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .map(|config| {
                config["mcpServers"]["fgp"].is_object()
            })
            .unwrap_or(false)
    } else {
        false
    };

    agents.push(AgentInfo {
        name: "Cursor".into(),
        id: "cursor".into(),
        installed: cursor_installed,
        config_path: Some(cursor_config_path.to_string_lossy().into()),
        registered: cursor_registered,
    });

    // Claude Desktop
    let claude_desktop_config = home.join("Library/Application Support/Claude/claude_desktop_config.json");
    let claude_desktop_installed = claude_desktop_config.parent().map(|p| p.exists()).unwrap_or(false);
    let claude_desktop_registered = if claude_desktop_config.exists() {
        fs::read_to_string(&claude_desktop_config)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .map(|config| {
                config["mcpServers"]["fgp"].is_object()
            })
            .unwrap_or(false)
    } else {
        false
    };

    agents.push(AgentInfo {
        name: "Claude Desktop".into(),
        id: "claude-desktop".into(),
        installed: claude_desktop_installed,
        config_path: Some(claude_desktop_config.to_string_lossy().into()),
        registered: claude_desktop_registered,
    });

    Ok(agents)
}

/// Register FGP MCP server with an AI agent
#[tauri::command]
async fn register_mcp(agent_id: String, app: AppHandle) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    // Get path to bundled MCP server
    let mcp_server_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources/mcp/fgp-mcp-server.py");

    if !mcp_server_path.exists() {
        return Err("MCP server not found in app bundle".into());
    }

    let mcp_server_str = mcp_server_path.to_string_lossy().to_string();

    // MCP server config
    let mcp_config = serde_json::json!({
        "command": "python3",
        "args": [mcp_server_str]
    });

    match agent_id.as_str() {
        "claude-code" => {
            // Use claude CLI to register
            let output = std::process::Command::new("claude")
                .args(["mcp", "add", "fgp", "--", "python3", &mcp_server_str])
                .output()
                .map_err(|e| format!("Failed to run claude CLI: {}", e))?;

            if !output.status.success() {
                // Try removing first then adding
                let _ = std::process::Command::new("claude")
                    .args(["mcp", "remove", "fgp"])
                    .output();

                let output = std::process::Command::new("claude")
                    .args(["mcp", "add", "fgp", "--", "python3", &mcp_server_str])
                    .output()
                    .map_err(|e| format!("Failed to run claude CLI: {}", e))?;

                if !output.status.success() {
                    return Err(format!(
                        "Failed to register: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }
        "cursor" => {
            let config_path = home.join(".cursor/mcp.json");
            update_mcp_config(&config_path, &mcp_config)?;
        }
        "claude-desktop" => {
            let config_path = home.join("Library/Application Support/Claude/claude_desktop_config.json");
            update_mcp_config(&config_path, &mcp_config)?;
        }
        _ => return Err(format!("Unknown agent: {}", agent_id)),
    }

    Ok(())
}

/// Unregister FGP MCP server from an AI agent
#[tauri::command]
async fn unregister_mcp(agent_id: String) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    match agent_id.as_str() {
        "claude-code" => {
            let _ = std::process::Command::new("claude")
                .args(["mcp", "remove", "fgp"])
                .output();
        }
        "cursor" => {
            let config_path = home.join(".cursor/mcp.json");
            remove_from_mcp_config(&config_path)?;
        }
        "claude-desktop" => {
            let config_path = home.join("Library/Application Support/Claude/claude_desktop_config.json");
            remove_from_mcp_config(&config_path)?;
        }
        _ => return Err(format!("Unknown agent: {}", agent_id)),
    }

    Ok(())
}

/// Get MCP server configuration for manual setup
#[tauri::command]
async fn get_mcp_config(app: AppHandle) -> Result<String, String> {
    let mcp_server_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources/mcp/fgp-mcp-server.py");

    let config = serde_json::json!({
        "mcpServers": {
            "fgp": {
                "command": "python3",
                "args": [mcp_server_path.to_string_lossy()]
            }
        }
    });

    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

/// Helper: Update MCP config file
fn update_mcp_config(config_path: &std::path::Path, mcp_config: &serde_json::Value) -> Result<(), String> {
    let mut config: serde_json::Value = if config_path.exists() {
        let content = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Ensure mcpServers object exists
    if !config["mcpServers"].is_object() {
        config["mcpServers"] = serde_json::json!({});
    }

    // Add FGP server
    config["mcpServers"]["fgp"] = mcp_config.clone();

    // Write back
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())
}

/// Helper: Remove FGP from MCP config file
fn remove_from_mcp_config(config_path: &std::path::Path) -> Result<(), String> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let mut config: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));

    if let Some(servers) = config["mcpServers"].as_object_mut() {
        servers.remove("fgp");
    }

    fs::write(config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())
}

// ============================================================================
// Auto-Start (launchd) Commands
// ============================================================================

const LAUNCHD_LABEL: &str = "com.fgp.manager";

/// Get the path to the LaunchAgent plist
fn get_launchd_plist_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home.join("Library/LaunchAgents").join(format!("{}.plist", LAUNCHD_LABEL)))
}

/// Check if auto-start is enabled
#[tauri::command]
async fn is_autostart_enabled() -> Result<bool, String> {
    let plist_path = get_launchd_plist_path()?;
    Ok(plist_path.exists())
}

/// Enable auto-start on login
#[tauri::command]
async fn enable_autostart(app: AppHandle) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let launch_agents_dir = home.join("Library/LaunchAgents");

    // Create LaunchAgents directory if it doesn't exist
    if !launch_agents_dir.exists() {
        fs::create_dir_all(&launch_agents_dir).map_err(|e| e.to_string())?;
    }

    // Get the app executable path
    let app_path = std::env::current_exe().map_err(|e| e.to_string())?;

    // For bundled app, we want the .app bundle path, not the binary inside
    // /path/to/FGP Manager.app/Contents/MacOS/fgp-manager -> /path/to/FGP Manager.app
    let app_bundle_path = app_path
        .parent() // MacOS
        .and_then(|p| p.parent()) // Contents
        .and_then(|p| p.parent()) // FGP Manager.app
        .ok_or("Could not determine app bundle path")?;

    // Generate plist content
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/open</string>
        <string>-a</string>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>{}/Library/Logs/fgp-manager.log</string>
    <key>StandardErrorPath</key>
    <string>{}/Library/Logs/fgp-manager.log</string>
</dict>
</plist>
"#,
        LAUNCHD_LABEL,
        app_bundle_path.display(),
        home.display(),
        home.display()
    );

    let plist_path = get_launchd_plist_path()?;
    fs::write(&plist_path, plist_content).map_err(|e| format!("Failed to write plist: {}", e))?;

    // Load the launch agent
    let output = std::process::Command::new("launchctl")
        .args(["load", plist_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to run launchctl: {}", e))?;

    if !output.status.success() {
        // If already loaded, that's fine
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already loaded") {
            tracing::warn!("launchctl load warning: {}", stderr);
        }
    }

    tracing::info!("Auto-start enabled");
    Ok(())
}

/// Disable auto-start on login
#[tauri::command]
async fn disable_autostart() -> Result<(), String> {
    let plist_path = get_launchd_plist_path()?;

    if plist_path.exists() {
        // Unload the launch agent first
        let _ = std::process::Command::new("launchctl")
            .args(["unload", plist_path.to_str().unwrap()])
            .output();

        // Remove the plist file
        fs::remove_file(&plist_path).map_err(|e| format!("Failed to remove plist: {}", e))?;
    }

    tracing::info!("Auto-start disabled");
    Ok(())
}

// ============================================================================
// Tray Icon Management
// ============================================================================

/// Calculate overall status from daemon states
fn calculate_tray_status(daemons: &[DaemonInfo]) -> TrayStatus {
    if daemons.is_empty() {
        return TrayStatus::Error;
    }

    let running_count = daemons.iter().filter(|d| d.is_running).count();
    let total_count = daemons.len();

    if running_count == 0 {
        TrayStatus::Error
    } else if running_count < total_count {
        TrayStatus::Degraded
    } else {
        TrayStatus::Healthy
    }
}

/// Update tray icon tooltip with status summary
fn update_tray_tooltip(app: &AppHandle, daemons: &[DaemonInfo]) {
    let running = daemons.iter().filter(|d| d.is_running).count();
    let total = daemons.len();
    let tooltip = format!("FGP Manager - {} of {} daemons running", running, total);

    if let Some(tray) = app.tray_by_id("fgp-tray") {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

// ============================================================================
// Window Management
// ============================================================================

/// Create and show the popover panel window
fn create_popover(app: &AppHandle) {
    // Check if popover already exists
    if let Some(window) = app.get_webview_window("popover") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = PositionerExt::move_window(&window, Position::TrayCenter);
            let _ = window.show();
            let _ = window.set_focus();
        }
        return;
    }

    // Create new popover window
    let window = WebviewWindowBuilder::new(app, "popover", WebviewUrl::default())
        .title("FGP Manager")
        .inner_size(320.0, 400.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(true)
        .build();

    if let Ok(window) = window {
        // Apply macOS vibrancy effect
        #[cfg(target_os = "macos")]
        {
            use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
            let _ = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(12.0));
        }

        // Position under tray and show
        let _ = PositionerExt::move_window(&window, Position::TrayCenter);
        let _ = window.show();

        // Auto-hide on focus loss
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                let _ = window_clone.hide();
            }
        });
    }
}

/// Show the settings window
fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("/settings".into()))
        .title("FGP Manager Settings")
        .inner_size(500.0, 600.0)
        .resizable(true)
        .build();
}

/// Show the marketplace window
fn show_marketplace(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("marketplace") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "marketplace", WebviewUrl::App("/marketplace".into()))
        .title("FGP Marketplace")
        .inner_size(800.0, 600.0)
        .resizable(true)
        .build();
}

// ============================================================================
// Background Monitoring
// ============================================================================

/// Start background daemon health monitoring
async fn start_monitoring(app: AppHandle, running: Arc<AtomicBool>) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    while running.load(Ordering::SeqCst) {
        interval.tick().await;

        // Fetch daemon status
        if let Ok(daemons) = list_daemons().await {
            // Update tray tooltip
            update_tray_tooltip(&app, &daemons);

            // Emit event to frontend for real-time updates
            let _ = app.emit("daemons-updated", &daemons);
        }
    }
}

// ============================================================================
// App Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting FGP Manager");

    let running = Arc::new(AtomicBool::new(true));

    tauri::Builder::default()
        // Plugins
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // Setup
        .setup(move |app| {
            // macOS: Hide dock icon (menu bar only)
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Build right-click menu
            let quit = MenuItem::with_id(app, "quit", "Quit FGP Manager", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let marketplace =
                MenuItem::with_id(app, "marketplace", "Marketplace...", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;

            let menu = Menu::with_items(app, &[&marketplace, &settings, &separator, &quit])?;

            // Build tray icon
            TrayIconBuilder::with_id("fgp-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("FGP Manager")
                .on_menu_event(|app, event| {
                    match event.id.0.as_str() {
                        "quit" => {
                            app.exit(0);
                        }
                        "settings" => {
                            show_settings(app);
                        }
                        "marketplace" => {
                            show_marketplace(app);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // Required for positioner to track tray position
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                    // Handle left click to toggle popover
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        create_popover(tray.app_handle());
                    }
                })
                .build(app)?;

            // Start background monitoring
            let app_handle = app.handle().clone();
            let running_clone = running.clone();
            tauri::async_runtime::spawn(async move {
                start_monitoring(app_handle, running_clone).await;
            });

            tracing::info!("FGP Manager setup complete");
            Ok(())
        })
        // Commands
        .invoke_handler(tauri::generate_handler![
            list_daemons,
            get_daemon_health,
            start_daemon,
            stop_daemon,
            restart_daemon,
            is_daemon_running,
            // Marketplace
            fetch_registry,
            get_package_details,
            install_package,
            uninstall_package,
            // MCP Integration
            detect_agents,
            register_mcp,
            unregister_mcp,
            get_mcp_config,
            // Auto-Start
            is_autostart_enabled,
            enable_autostart,
            disable_autostart,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Handle app events
            if let RunEvent::ExitRequested { api, .. } = event {
                // Prevent exit on window close (stay in tray)
                api.prevent_exit();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_tray_status, get_launchd_plist_path, remove_from_mcp_config,
        update_mcp_config, DaemonInfo, TrayStatus,
    };
    use serde_json::{json, Value};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("fgp-manager-{}-{}.json", name, stamp))
    }

    fn daemon(name: &str, is_running: bool) -> DaemonInfo {
        DaemonInfo {
            name: name.to_string(),
            status: if is_running { "running" } else { "stopped" }.to_string(),
            version: None,
            uptime_seconds: None,
            is_running,
            has_manifest: true,
        }
    }

    #[test]
    fn update_mcp_config_sets_fgp_entry() {
        let path = temp_path("mcp");
        let mcp = json!({"command": "python3", "args": ["/tmp/fgp.py"]});
        update_mcp_config(&path, &mcp).expect("update");

        let content = fs::read_to_string(&path).expect("read");
        let config: Value = serde_json::from_str(&content).expect("json");
        assert_eq!(config["mcpServers"]["fgp"], mcp);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn update_mcp_config_preserves_existing_servers() {
        let path = temp_path("mcp-existing");
        let existing = json!({"mcpServers": {"other": {"command": "node"}}});
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(&path, serde_json::to_string_pretty(&existing).unwrap()).expect("write");

        let mcp = json!({"command": "python3", "args": ["/tmp/fgp.py"]});
        update_mcp_config(&path, &mcp).expect("update");

        let content = fs::read_to_string(&path).expect("read");
        let config: Value = serde_json::from_str(&content).expect("json");
        assert!(config["mcpServers"]["other"].is_object());
        assert_eq!(config["mcpServers"]["fgp"], mcp);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn remove_from_mcp_config_removes_only_fgp() {
        let path = temp_path("mcp-remove");
        let config = json!({
            "mcpServers": {
                "fgp": {"command": "python3"},
                "other": {"command": "node"}
            }
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).expect("write");

        remove_from_mcp_config(&path).expect("remove");
        let content = fs::read_to_string(&path).expect("read");
        let updated: Value = serde_json::from_str(&content).expect("json");
        assert!(updated["mcpServers"]["fgp"].is_null());
        assert!(updated["mcpServers"]["other"].is_object());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn calculate_tray_status_reflects_running_counts() {
        assert_eq!(calculate_tray_status(&[]), TrayStatus::Error);

        let none_running = vec![daemon("a", false), daemon("b", false)];
        assert_eq!(calculate_tray_status(&none_running), TrayStatus::Error);

        let partial = vec![daemon("a", true), daemon("b", false)];
        assert_eq!(calculate_tray_status(&partial), TrayStatus::Degraded);

        let all_running = vec![daemon("a", true), daemon("b", true)];
        assert_eq!(calculate_tray_status(&all_running), TrayStatus::Healthy);
    }

    #[test]
    fn launchd_path_ends_with_plist_name() {
        if let Ok(path) = get_launchd_plist_path() {
            assert!(path.ends_with("Library/LaunchAgents/com.fgp.manager.plist"));
        }
    }
}
