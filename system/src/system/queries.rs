//! System information queries with caching.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

/// Cache TTL values
const HARDWARE_TTL_SECS: u64 = 3600; // 1 hour
const DISK_TTL_SECS: u64 = 300;      // 5 minutes
const NETWORK_TTL_SECS: u64 = 60;    // 1 minute
const PROCESS_TTL_SECS: u64 = 30;    // 30 seconds
const APPS_TTL_SECS: u64 = 600;      // 10 minutes
const BATTERY_TTL_SECS: u64 = 60;    // 1 minute

/// Hardware information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub model_name: Option<String>,
    pub model_identifier: Option<String>,
    pub chip: Option<String>,
    pub cores: Option<String>,
    pub memory: Option<String>,
    pub serial_number: Option<String>,
}

/// Disk information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub percent_used: f64,
}

/// Network interface information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub device: String,
    pub mac_address: Option<String>,
    pub ip_address: Option<String>,
}

/// Process information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub user: String,
}

/// Installed application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: Option<String>,
    pub path: String,
    pub bundle_id: Option<String>,
}

/// Battery information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub percent: u32,
    pub charging: bool,
    pub time_remaining: Option<String>,
    pub cycle_count: Option<u32>,
    pub health: Option<String>,
}

/// System statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub uptime_seconds: u64,
    pub load_average: [f64; 3],
    pub cpu_usage: f64,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub swap_used_gb: f64,
    pub swap_total_gb: f64,
}

/// Cached system queries.
pub struct SystemCache {
    hardware_cache: Cache<String, HardwareInfo>,
    disk_cache: Cache<String, Vec<DiskInfo>>,
    network_cache: Cache<String, Vec<NetworkInterface>>,
    process_cache: Cache<String, Vec<ProcessInfo>>,
    apps_cache: Cache<String, Vec<AppInfo>>,
    battery_cache: Cache<String, BatteryInfo>,
    stats_cache: Cache<String, SystemStats>,
}

impl SystemCache {
    /// Create a new system cache.
    pub fn new() -> Self {
        Self {
            hardware_cache: Cache::builder()
                .time_to_live(Duration::from_secs(HARDWARE_TTL_SECS))
                .max_capacity(1)
                .build(),
            disk_cache: Cache::builder()
                .time_to_live(Duration::from_secs(DISK_TTL_SECS))
                .max_capacity(1)
                .build(),
            network_cache: Cache::builder()
                .time_to_live(Duration::from_secs(NETWORK_TTL_SECS))
                .max_capacity(1)
                .build(),
            process_cache: Cache::builder()
                .time_to_live(Duration::from_secs(PROCESS_TTL_SECS))
                .max_capacity(1)
                .build(),
            apps_cache: Cache::builder()
                .time_to_live(Duration::from_secs(APPS_TTL_SECS))
                .max_capacity(1)
                .build(),
            battery_cache: Cache::builder()
                .time_to_live(Duration::from_secs(BATTERY_TTL_SECS))
                .max_capacity(1)
                .build(),
            stats_cache: Cache::builder()
                .time_to_live(Duration::from_secs(PROCESS_TTL_SECS))
                .max_capacity(1)
                .build(),
        }
    }

    /// Get hardware information (cached for 1 hour).
    pub fn hardware(&self) -> Result<HardwareInfo> {
        if let Some(cached) = self.hardware_cache.get("hardware") {
            return Ok(cached);
        }

        let info = query_hardware()?;
        self.hardware_cache.insert("hardware".into(), info.clone());
        Ok(info)
    }

    /// Get disk information (cached for 5 minutes).
    pub fn disks(&self) -> Result<Vec<DiskInfo>> {
        if let Some(cached) = self.disk_cache.get("disks") {
            return Ok(cached);
        }

        let info = query_disks()?;
        self.disk_cache.insert("disks".into(), info.clone());
        Ok(info)
    }

    /// Get network interfaces (cached for 1 minute).
    pub fn network(&self) -> Result<Vec<NetworkInterface>> {
        if let Some(cached) = self.network_cache.get("network") {
            return Ok(cached);
        }

        let info = query_network()?;
        self.network_cache.insert("network".into(), info.clone());
        Ok(info)
    }

    /// Get running processes (cached for 30 seconds).
    pub fn processes(&self, limit: u32) -> Result<Vec<ProcessInfo>> {
        // Always query fresh for different limits
        let key = format!("processes_{}", limit);
        if let Some(cached) = self.process_cache.get(&key) {
            return Ok(cached);
        }

        let info = query_processes(limit)?;
        self.process_cache.insert(key, info.clone());
        Ok(info)
    }

    /// Get installed applications (cached for 10 minutes).
    pub fn apps(&self) -> Result<Vec<AppInfo>> {
        if let Some(cached) = self.apps_cache.get("apps") {
            return Ok(cached);
        }

        let info = query_apps()?;
        self.apps_cache.insert("apps".into(), info.clone());
        Ok(info)
    }

    /// Get battery information (cached for 1 minute).
    pub fn battery(&self) -> Result<BatteryInfo> {
        if let Some(cached) = self.battery_cache.get("battery") {
            return Ok(cached);
        }

        let info = query_battery()?;
        self.battery_cache.insert("battery".into(), info.clone());
        Ok(info)
    }

    /// Get system statistics (cached for 30 seconds).
    pub fn stats(&self) -> Result<SystemStats> {
        if let Some(cached) = self.stats_cache.get("stats") {
            return Ok(cached);
        }

        let info = query_stats()?;
        self.stats_cache.insert("stats".into(), info.clone());
        Ok(info)
    }

    /// Invalidate all caches.
    pub fn invalidate_all(&self) {
        self.hardware_cache.invalidate_all();
        self.disk_cache.invalidate_all();
        self.network_cache.invalidate_all();
        self.process_cache.invalidate_all();
        self.apps_cache.invalidate_all();
        self.battery_cache.invalidate_all();
        self.stats_cache.invalidate_all();
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "hardware": {
                "entries": self.hardware_cache.entry_count(),
                "ttl_secs": HARDWARE_TTL_SECS,
            },
            "disk": {
                "entries": self.disk_cache.entry_count(),
                "ttl_secs": DISK_TTL_SECS,
            },
            "network": {
                "entries": self.network_cache.entry_count(),
                "ttl_secs": NETWORK_TTL_SECS,
            },
            "processes": {
                "entries": self.process_cache.entry_count(),
                "ttl_secs": PROCESS_TTL_SECS,
            },
            "apps": {
                "entries": self.apps_cache.entry_count(),
                "ttl_secs": APPS_TTL_SECS,
            },
            "battery": {
                "entries": self.battery_cache.entry_count(),
                "ttl_secs": BATTERY_TTL_SECS,
            },
            "stats": {
                "entries": self.stats_cache.entry_count(),
                "ttl_secs": PROCESS_TTL_SECS,
            },
        })
    }
}

impl Default for SystemCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Query hardware information via system_profiler.
fn query_hardware() -> Result<HardwareInfo> {
    let output = Command::new("system_profiler")
        .arg("SPHardwareDataType")
        .output()
        .map_err(|e| anyhow!("Failed to run system_profiler: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(HardwareInfo {
        model_name: extract_field(&stdout, "Model Name:"),
        model_identifier: extract_field(&stdout, "Model Identifier:"),
        chip: extract_field(&stdout, "Chip:"),
        cores: extract_field(&stdout, "Total Number of Cores:"),
        memory: extract_field(&stdout, "Memory:"),
        serial_number: extract_field(&stdout, "Serial Number"),
    })
}

/// Query disk information via df.
fn query_disks() -> Result<Vec<DiskInfo>> {
    let output = Command::new("df")
        .args(["-h", "-P"])
        .output()
        .map_err(|e| anyhow!("Failed to run df: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            // Skip pseudo-filesystems
            if parts[0].starts_with("/dev/") {
                let total = parse_size(parts[1]);
                let used = parse_size(parts[2]);
                let available = parse_size(parts[3]);
                let percent = parts[4].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);

                disks.push(DiskInfo {
                    device: parts[0].to_string(),
                    mount_point: parts[5..].join(" "),
                    filesystem: "apfs".to_string(), // macOS default
                    total_bytes: total,
                    used_bytes: used,
                    available_bytes: available,
                    percent_used: percent,
                });
            }
        }
    }

    Ok(disks)
}

/// Query network interfaces.
fn query_network() -> Result<Vec<NetworkInterface>> {
    let output = Command::new("networksetup")
        .arg("-listallhardwareports")
        .output()
        .map_err(|e| anyhow!("Failed to run networksetup: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();
    let mut current_name = String::new();
    let mut current_device = String::new();
    let mut current_mac = None;

    for line in stdout.lines() {
        if line.starts_with("Hardware Port:") {
            if !current_device.is_empty() {
                let ip = get_ip_for_interface(&current_device);
                interfaces.push(NetworkInterface {
                    name: current_name.clone(),
                    device: current_device.clone(),
                    mac_address: current_mac.take(),
                    ip_address: ip,
                });
            }
            current_name = line.trim_start_matches("Hardware Port:").trim().to_string();
        } else if line.starts_with("Device:") {
            current_device = line.trim_start_matches("Device:").trim().to_string();
        } else if line.starts_with("Ethernet Address:") {
            current_mac = Some(line.trim_start_matches("Ethernet Address:").trim().to_string());
        }
    }

    // Don't forget the last interface
    if !current_device.is_empty() {
        let ip = get_ip_for_interface(&current_device);
        interfaces.push(NetworkInterface {
            name: current_name,
            device: current_device,
            mac_address: current_mac,
            ip_address: ip,
        });
    }

    Ok(interfaces)
}

/// Get IP address for an interface.
fn get_ip_for_interface(device: &str) -> Option<String> {
    let output = Command::new("ipconfig")
        .args(["getifaddr", device])
        .output()
        .ok()?;

    if output.status.success() {
        let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !ip.is_empty() {
            return Some(ip);
        }
    }
    None
}

/// Query running processes via ps.
fn query_processes(limit: u32) -> Result<Vec<ProcessInfo>> {
    let output = Command::new("ps")
        .args(["-axo", "pid,pcpu,pmem,user,comm", "-r"])
        .output()
        .map_err(|e| anyhow!("Failed to run ps: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in stdout.lines().skip(1).take(limit as usize) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            processes.push(ProcessInfo {
                pid: parts[0].parse().unwrap_or(0),
                cpu_percent: parts[1].parse().unwrap_or(0.0),
                memory_percent: parts[2].parse().unwrap_or(0.0),
                user: parts[3].to_string(),
                name: parts[4..].join(" "),
            });
        }
    }

    Ok(processes)
}

/// Query installed applications.
fn query_apps() -> Result<Vec<AppInfo>> {
    let output = Command::new("mdfind")
        .args(["kMDItemContentType == 'com.apple.application-bundle'"])
        .output()
        .map_err(|e| anyhow!("Failed to run mdfind: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();

    for path in stdout.lines() {
        if path.ends_with(".app") {
            let name = std::path::Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Get bundle info
            let (version, bundle_id) = get_app_info(path);

            apps.push(AppInfo {
                name,
                version,
                path: path.to_string(),
                bundle_id,
            });
        }
    }

    // Sort by name
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(apps)
}

/// Get app version and bundle ID from Info.plist.
fn get_app_info(app_path: &str) -> (Option<String>, Option<String>) {
    let plist_path = format!("{}/Contents/Info.plist", app_path);

    let version = Command::new("defaults")
        .args(["read", &plist_path, "CFBundleShortVersionString"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let bundle_id = Command::new("defaults")
        .args(["read", &plist_path, "CFBundleIdentifier"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    (version, bundle_id)
}

/// Query battery information.
fn query_battery() -> Result<BatteryInfo> {
    let output = Command::new("pmset")
        .args(["-g", "batt"])
        .output()
        .map_err(|e| anyhow!("Failed to run pmset: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse: "Now drawing from 'Battery Power'"
    // " -InternalBattery-0 (id=...)	82%; charging; 0:30 remaining present: true"
    let charging = stdout.contains("charging") || stdout.contains("AC Power");
    let mut percent = 100u32;
    let mut time_remaining = None;

    for line in stdout.lines() {
        if line.contains("InternalBattery") || line.contains('%') {
            // Extract percentage
            if let Some(pct_pos) = line.find('%') {
                let start = line[..pct_pos].rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
                if let Ok(p) = line[start..pct_pos].trim().parse::<u32>() {
                    percent = p;
                }
            }
            // Extract time remaining
            if let Some(remaining) = line.find("remaining") {
                let before = &line[..remaining];
                if let Some(time_start) = before.rfind(';') {
                    time_remaining = Some(before[time_start + 1..].trim().to_string());
                }
            }
        }
    }

    // Get battery health/cycle count from system_profiler
    let health_output = Command::new("system_profiler")
        .arg("SPPowerDataType")
        .output()
        .ok();

    let (cycle_count, health) = if let Some(output) = health_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let cycle = extract_field(&stdout, "Cycle Count:")
            .and_then(|s| s.parse().ok());
        let health = extract_field(&stdout, "Condition:");
        (cycle, health)
    } else {
        (None, None)
    };

    Ok(BatteryInfo {
        percent,
        charging,
        time_remaining,
        cycle_count,
        health,
    })
}

/// Query system statistics.
fn query_stats() -> Result<SystemStats> {
    // Get uptime
    let uptime_output = Command::new("sysctl")
        .args(["-n", "kern.boottime"])
        .output()
        .map_err(|e| anyhow!("Failed to get uptime: {}", e))?;

    let uptime_str = String::from_utf8_lossy(&uptime_output.stdout);
    let uptime_seconds = parse_boottime(&uptime_str);

    // Get load average
    let load_output = Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .map_err(|e| anyhow!("Failed to get load: {}", e))?;

    let load_str = String::from_utf8_lossy(&load_output.stdout);
    let load_average = parse_load_average(&load_str);

    // Get memory info via vm_stat
    let vm_output = Command::new("vm_stat")
        .output()
        .map_err(|e| anyhow!("Failed to get vm_stat: {}", e))?;

    let vm_str = String::from_utf8_lossy(&vm_output.stdout);
    let (memory_used_gb, memory_total_gb) = parse_vm_stat(&vm_str);

    // Get swap info
    let swap_output = Command::new("sysctl")
        .args(["-n", "vm.swapusage"])
        .output()
        .ok();

    let (swap_used_gb, swap_total_gb) = swap_output
        .map(|o| parse_swap(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or((0.0, 0.0));

    // Estimate CPU usage from load average
    let cpu_usage = load_average[0] / num_cpus() as f64 * 100.0;

    Ok(SystemStats {
        uptime_seconds,
        load_average,
        cpu_usage: cpu_usage.min(100.0),
        memory_used_gb,
        memory_total_gb,
        swap_used_gb,
        swap_total_gb,
    })
}

/// Get number of CPUs.
fn num_cpus() -> usize {
    Command::new("sysctl")
        .args(["-n", "hw.ncpu"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(4)
}

/// Extract a field value from system_profiler output.
fn extract_field(output: &str, field: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains(field) {
            return Some(line.split(':').nth(1)?.trim().to_string());
        }
    }
    None
}

/// Parse human-readable size (1G, 500M, etc.) to bytes.
fn parse_size(size: &str) -> u64 {
    let size = size.trim();
    let multiplier: u64 = match size.chars().last() {
        Some('T') | Some('t') => 1024_u64 * 1024 * 1024 * 1024,
        Some('G') | Some('g') => 1024_u64 * 1024 * 1024,
        Some('M') | Some('m') => 1024_u64 * 1024,
        Some('K') | Some('k') => 1024_u64,
        _ => 1,
    };

    let num_str: String = size.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    let num: f64 = num_str.parse().unwrap_or(0.0);
    (num * multiplier as f64) as u64
}

/// Parse boottime from sysctl output.
fn parse_boottime(s: &str) -> u64 {
    // Format: { sec = 1736934543, usec = 0 }
    if let Some(sec_start) = s.find("sec = ") {
        let after = &s[sec_start + 6..];
        if let Some(sec_end) = after.find(',') {
            if let Ok(sec) = after[..sec_end].trim().parse::<u64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return now.saturating_sub(sec);
            }
        }
    }
    0
}

/// Parse load average from sysctl output.
fn parse_load_average(s: &str) -> [f64; 3] {
    // Format: { 1.23 0.45 0.67 }
    let nums: Vec<f64> = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .filter_map(|n| n.parse().ok())
        .collect();

    [
        nums.first().copied().unwrap_or(0.0),
        nums.get(1).copied().unwrap_or(0.0),
        nums.get(2).copied().unwrap_or(0.0),
    ]
}

/// Parse vm_stat output for memory usage.
fn parse_vm_stat(s: &str) -> (f64, f64) {
    let page_size: u64 = 16384; // M-series Macs use 16KB pages
    let mut pages_active = 0u64;
    let mut pages_wired = 0u64;
    let mut pages_compressed = 0u64;
    let mut pages_free = 0u64;
    let mut pages_speculative = 0u64;

    for line in s.lines() {
        let value: u64 = line
            .split(':')
            .nth(1)
            .and_then(|v| v.trim().trim_end_matches('.').parse().ok())
            .unwrap_or(0);

        if line.starts_with("Pages active") {
            pages_active = value;
        } else if line.starts_with("Pages wired") {
            pages_wired = value;
        } else if line.starts_with("Pages occupied by compressor") {
            pages_compressed = value;
        } else if line.starts_with("Pages free") {
            pages_free = value;
        } else if line.starts_with("Pages speculative") {
            pages_speculative = value;
        }
    }

    let used_pages = pages_active + pages_wired + pages_compressed;
    let total_pages = used_pages + pages_free + pages_speculative;

    let used_gb = (used_pages * page_size) as f64 / 1024.0 / 1024.0 / 1024.0;
    let total_gb = (total_pages * page_size) as f64 / 1024.0 / 1024.0 / 1024.0;

    (used_gb, total_gb)
}

/// Parse swap usage from sysctl output.
fn parse_swap(s: &str) -> (f64, f64) {
    // Format: total = 6144.00M  used = 3712.75M  free = 2431.25M
    let mut total = 0.0f64;
    let mut used = 0.0f64;

    for part in s.split_whitespace() {
        if part.ends_with('M') || part.ends_with('G') {
            let num_str: String = part.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            let num: f64 = num_str.parse().unwrap_or(0.0);
            let value = if part.ends_with('G') { num } else { num / 1024.0 };

            if total == 0.0 {
                total = value;
            } else if used == 0.0 {
                used = value;
                break;
            }
        }
    }

    (used, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_query() {
        let info = query_hardware().unwrap();
        assert!(info.chip.is_some() || info.model_name.is_some());
    }

    #[test]
    fn test_disk_query() {
        let disks = query_disks().unwrap();
        assert!(!disks.is_empty());
    }

    #[test]
    fn test_cache() {
        let cache = SystemCache::new();

        // First call should query
        let hw1 = cache.hardware().unwrap();
        // Second call should use cache
        let hw2 = cache.hardware().unwrap();

        assert_eq!(hw1.chip, hw2.chip);
    }
}
