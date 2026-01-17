//! Screen Time SQLite queries and data access.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Timelike, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Mac Absolute Time epoch offset (2001-01-01 00:00:00 UTC)
const MAC_ABSOLUTE_TIME_OFFSET: i64 = 978307200;

/// Convert Mac Absolute Time to Unix timestamp
fn mac_to_unix(mac_time: f64) -> i64 {
    (mac_time as i64) + MAC_ABSOLUTE_TIME_OFFSET
}

/// Convert Unix timestamp to Mac Absolute Time
fn unix_to_mac(unix_time: i64) -> f64 {
    (unix_time - MAC_ABSOLUTE_TIME_OFFSET) as f64
}

/// Format seconds as human-readable duration
fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Default knowledgeC.db path.
pub fn default_db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library")
        .join("Application Support")
        .join("Knowledge")
        .join("knowledgeC.db")
}

/// App usage session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSession {
    pub bundle_id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_seconds: i64,
}

/// Per-app usage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageSummary {
    pub bundle_id: String,
    pub total_seconds: i64,
    pub total_formatted: String,
    pub session_count: i64,
}

/// Daily usage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String,
    pub total_seconds: i64,
    pub total_formatted: String,
    pub breakdown: Vec<AppUsageSummary>,
}

/// Screen Time data store.
pub struct ScreenTimeStore {
    conn: Connection,
}

impl ScreenTimeStore {
    /// Open a read-only connection to knowledgeC.db.
    pub fn new() -> Result<Self> {
        let db_path = default_db_path();
        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("Failed to open knowledgeC.db at {:?}", db_path))?;

        Ok(Self { conn })
    }

    /// Check if we have access to the database.
    pub fn check_access() -> bool {
        Self::new().is_ok()
    }

    /// Get app usage for a specific bundle ID within a time range.
    pub fn app_usage(
        &self,
        bundle_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<UsageSession>> {
        let start_mac = unix_to_mac(start.timestamp());
        let end_mac = unix_to_mac(end.timestamp());

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                ZVALUESTRING,
                ZSTARTDATE,
                ZENDDATE
            FROM ZOBJECT
            WHERE ZSTREAMNAME = '/app/usage'
              AND ZVALUESTRING = ?1
              AND ZSTARTDATE >= ?2
              AND ZENDDATE <= ?3
            ORDER BY ZSTARTDATE DESC
            "#,
        )?;

        let sessions = stmt
            .query_map([bundle_id, &start_mac.to_string(), &end_mac.to_string()], |row| {
                let bundle: String = row.get(0)?;
                let start_mac: f64 = row.get(1)?;
                let end_mac: f64 = row.get(2)?;
                Ok((bundle, start_mac, end_mac))
            })?
            .filter_map(|r| r.ok())
            .map(|(bundle, start_mac, end_mac)| {
                let start_unix = mac_to_unix(start_mac);
                let end_unix = mac_to_unix(end_mac);
                UsageSession {
                    bundle_id: bundle,
                    start: Utc.timestamp_opt(start_unix, 0).unwrap(),
                    end: Utc.timestamp_opt(end_unix, 0).unwrap(),
                    duration_seconds: end_unix - start_unix,
                }
            })
            .collect();

        Ok(sessions)
    }

    /// Get daily total screen time with breakdown.
    pub fn daily_total(&self, date: NaiveDate) -> Result<DailyUsage> {
        // Start of day in local timezone, converted to UTC
        let start_of_day = date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(chrono::Local)
            .unwrap()
            .with_timezone(&Utc);
        let end_of_day = start_of_day + Duration::days(1);

        let start_mac = unix_to_mac(start_of_day.timestamp());
        let end_mac = unix_to_mac(end_of_day.timestamp());

        // Get per-app breakdown
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                ZVALUESTRING,
                SUM(ZENDDATE - ZSTARTDATE) as total_seconds,
                COUNT(*) as session_count
            FROM ZOBJECT
            WHERE ZSTREAMNAME = '/app/usage'
              AND ZSTARTDATE >= ?1
              AND ZENDDATE <= ?2
              AND ZVALUESTRING IS NOT NULL
            GROUP BY ZVALUESTRING
            ORDER BY total_seconds DESC
            "#,
        )?;

        let breakdown: Vec<AppUsageSummary> = stmt
            .query_map([start_mac, end_mac], |row| {
                let bundle: String = row.get(0)?;
                let total_secs: i64 = row.get(1)?;
                let count: i64 = row.get(2)?;
                Ok((bundle, total_secs, count))
            })?
            .filter_map(|r| r.ok())
            .map(|(bundle, secs, count)| AppUsageSummary {
                bundle_id: bundle,
                total_seconds: secs,
                total_formatted: format_duration(secs),
                session_count: count,
            })
            .collect();

        let total_seconds: i64 = breakdown.iter().map(|a| a.total_seconds).sum();

        Ok(DailyUsage {
            date: date.format("%Y-%m-%d").to_string(),
            total_seconds,
            total_formatted: format_duration(total_seconds),
            breakdown,
        })
    }

    /// Get top N most used apps within a time range.
    pub fn most_used(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<AppUsageSummary>> {
        let start_mac = unix_to_mac(start.timestamp());
        let end_mac = unix_to_mac(end.timestamp());

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                ZVALUESTRING,
                SUM(ZENDDATE - ZSTARTDATE) as total_seconds,
                COUNT(*) as session_count
            FROM ZOBJECT
            WHERE ZSTREAMNAME = '/app/usage'
              AND ZSTARTDATE >= ?1
              AND ZENDDATE <= ?2
              AND ZVALUESTRING IS NOT NULL
            GROUP BY ZVALUESTRING
            ORDER BY total_seconds DESC
            LIMIT ?3
            "#,
        )?;

        let apps: Vec<AppUsageSummary> = stmt
            .query_map(
                rusqlite::params![start_mac, end_mac, limit as i64],
                |row| {
                    let bundle: String = row.get(0)?;
                    let total_secs: i64 = row.get(1)?;
                    let count: i64 = row.get(2)?;
                    Ok((bundle, total_secs, count))
                },
            )?
            .filter_map(|r| r.ok())
            .map(|(bundle, secs, count)| AppUsageSummary {
                bundle_id: bundle,
                total_seconds: secs,
                total_formatted: format_duration(secs),
                session_count: count,
            })
            .collect();

        Ok(apps)
    }

    /// Get weekly summary (last 7 days).
    pub fn weekly_summary(&self) -> Result<Vec<DailyUsage>> {
        let today = chrono::Local::now().date_naive();
        let mut summaries = Vec::new();

        for i in 0..7 {
            let date = today - Duration::days(i);
            let daily = self.daily_total(date)?;
            summaries.push(daily);
        }

        Ok(summaries)
    }

    /// Get hourly breakdown for a specific date.
    pub fn usage_timeline(&self, date: NaiveDate) -> Result<HashMap<u32, i64>> {
        let start_of_day = date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(chrono::Local)
            .unwrap()
            .with_timezone(&Utc);
        let end_of_day = start_of_day + Duration::days(1);

        let start_mac = unix_to_mac(start_of_day.timestamp());
        let end_mac = unix_to_mac(end_of_day.timestamp());

        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                ZSTARTDATE,
                ZENDDATE
            FROM ZOBJECT
            WHERE ZSTREAMNAME = '/app/usage'
              AND ZSTARTDATE >= ?1
              AND ZENDDATE <= ?2
            "#,
        )?;

        let mut hourly: HashMap<u32, i64> = (0..24).map(|h| (h, 0i64)).collect();

        let rows: Vec<(f64, f64)> = stmt
            .query_map([start_mac, end_mac], |row| {
                let start: f64 = row.get(0)?;
                let end: f64 = row.get(1)?;
                Ok((start, end))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (start_mac, end_mac) in rows {
            let start_time = Utc.timestamp_opt(mac_to_unix(start_mac), 0).unwrap();
            let hour = start_time
                .with_timezone(&chrono::Local)
                .hour();
            let duration = (end_mac - start_mac) as i64;
            *hourly.entry(hour).or_insert(0) += duration;
        }

        Ok(hourly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
    use rusqlite::params;

    #[test]
    fn test_time_roundtrip() {
        let unix = 1_700_000_000i64;
        let mac = unix_to_mac(unix);
        let roundtrip = mac_to_unix(mac);
        assert_eq!(roundtrip, unix);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90), "1m");
    }

    #[test]
    fn test_default_db_path() {
        let path = default_db_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("Library/Application Support/Knowledge/knowledgeC.db"));
    }

    fn setup_store(rows: Vec<(&str, DateTime<Utc>, DateTime<Utc>)>) -> ScreenTimeStore {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            r#"
            CREATE TABLE ZOBJECT (
                ZVALUESTRING TEXT,
                ZSTARTDATE INTEGER,
                ZENDDATE INTEGER,
                ZSTREAMNAME TEXT
            );
            "#,
        )
        .expect("create schema");

        for (bundle, start, end) in rows {
            let start_mac = unix_to_mac(start.timestamp()) as i64;
            let end_mac = unix_to_mac(end.timestamp()) as i64;
            conn.execute(
                "INSERT INTO ZOBJECT (ZVALUESTRING, ZSTARTDATE, ZENDDATE, ZSTREAMNAME) VALUES (?, ?, ?, ?)",
                params![bundle, start_mac, end_mac, "/app/usage"],
            )
            .expect("insert row");
        }

        ScreenTimeStore { conn }
    }

    #[test]
    fn test_app_usage_returns_sessions() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 1, 1, 9, 0, 0).unwrap();
        let store = setup_store(vec![("com.example.app", start, end)]);

        let sessions = store
            .app_usage(
                "com.example.app",
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap(),
            )
            .expect("app usage");

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].duration_seconds, 3600);
    }

    #[test]
    fn test_daily_total_breakdown() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let start_of_day = date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let start_a = start_of_day + Duration::hours(9);
        let end_a = start_a + Duration::minutes(10);
        let start_b = start_of_day + Duration::hours(10);
        let end_b = start_b + Duration::minutes(20);

        let store = setup_store(vec![
            ("com.example.a", start_a, end_a),
            ("com.example.b", start_b, end_b),
        ]);

        let daily = store.daily_total(date).expect("daily total");

        assert_eq!(daily.total_seconds, 1800);
        assert_eq!(daily.breakdown.len(), 2);
        assert_eq!(daily.breakdown[0].total_seconds, 1200);
        assert_eq!(daily.breakdown[1].total_seconds, 600);
        assert_eq!(daily.total_formatted, "30m");
    }

    #[test]
    fn test_most_used_orders_by_duration() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let start = date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);
        let end = start + Duration::days(1);
        let store = setup_store(vec![
            (
                "com.example.a",
                start + Duration::hours(8),
                start + Duration::hours(9),
            ),
            (
                "com.example.b",
                start + Duration::hours(10),
                start + Duration::hours(10) + Duration::minutes(30),
            ),
        ]);

        let apps = store.most_used(start, end, 2).expect("most used");

        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].bundle_id, "com.example.a");
        assert_eq!(apps[0].total_seconds, 3600);
    }

    #[test]
    fn test_usage_timeline_groups_by_hour() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let start = date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc)
            + Duration::hours(14);
        let end = start + Duration::minutes(45);
        let store = setup_store(vec![("com.example.app", start, end)]);

        let timeline = store.usage_timeline(date).expect("timeline");

        assert_eq!(timeline.get(&14).copied().unwrap_or(0), 2700);
    }
}
