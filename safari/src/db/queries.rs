//! Safari database queries.
//!
//! Safari uses Core Data timestamps (seconds since 2001-01-01 00:00:00 UTC).
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;

/// Core Data epoch: 2001-01-01 00:00:00 UTC
const CORE_DATA_EPOCH: i64 = 978307200;

/// Convert Unix timestamp to Core Data timestamp.
pub fn unix_to_core_data(unix_ts: i64) -> f64 {
    (unix_ts - CORE_DATA_EPOCH) as f64
}

/// Convert Core Data timestamp to Unix timestamp.
pub fn core_data_to_unix(cd_ts: f64) -> i64 {
    (cd_ts as i64) + CORE_DATA_EPOCH
}

/// Convert Core Data timestamp to ISO 8601 string.
pub fn core_data_to_iso(cd_ts: f64) -> String {
    let unix_ts = core_data_to_unix(cd_ts);
    chrono::DateTime::from_timestamp(unix_ts, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get Core Data timestamp for N days ago.
pub fn days_ago_core_data(days: u32) -> f64 {
    let now = chrono::Utc::now().timestamp();
    let cutoff = now - (days as i64 * 24 * 3600);
    unix_to_core_data(cutoff)
}

// ============================================================================
// History Query Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HistoryItem {
    pub url: String,
    pub title: Option<String>,
    pub visit_time: String,
    pub visit_count: i64,
    pub domain: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TopSite {
    pub url: String,
    pub domain: Option<String>,
    pub visit_count: i64,
    pub last_visit: String,
}

#[derive(Debug, Serialize)]
pub struct CloudTab {
    pub title: Option<String>,
    pub url: String,
    pub device_name: Option<String>,
    pub is_pinned: bool,
}

#[derive(Debug, Serialize)]
pub struct CloudDevice {
    pub device_uuid: String,
    pub device_name: Option<String>,
    pub tab_count: i64,
}

// ============================================================================
// History Queries
// ============================================================================

/// Get recent history items with visit times.
pub fn query_recent_history(conn: &Connection, days: u32, limit: u32) -> Result<Vec<HistoryItem>> {
    let cutoff = days_ago_core_data(days);

    let mut stmt = conn.prepare(
        r#"
        SELECT
            hi.url,
            hv.title,
            hv.visit_time,
            hi.visit_count,
            hi.domain_expansion
        FROM history_visits hv
        JOIN history_items hi ON hv.history_item = hi.id
        WHERE hv.visit_time > ?
        ORDER BY hv.visit_time DESC
        LIMIT ?
        "#,
    )?;

    let rows = stmt.query_map(params![cutoff, limit], |row| {
        let visit_time: f64 = row.get(2)?;
        Ok(HistoryItem {
            url: row.get(0)?,
            title: row.get(1)?,
            visit_time: core_data_to_iso(visit_time),
            visit_count: row.get(3)?,
            domain: row.get(4)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Search history by URL or title.
pub fn query_search_history(
    conn: &Connection,
    query: &str,
    days: u32,
    limit: u32,
) -> Result<Vec<HistoryItem>> {
    let cutoff = days_ago_core_data(days);
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        r#"
        SELECT
            hi.url,
            hv.title,
            hv.visit_time,
            hi.visit_count,
            hi.domain_expansion
        FROM history_visits hv
        JOIN history_items hi ON hv.history_item = hi.id
        WHERE hv.visit_time > ?
          AND (hi.url LIKE ? OR hv.title LIKE ?)
        ORDER BY hv.visit_time DESC
        LIMIT ?
        "#,
    )?;

    let rows = stmt.query_map(params![cutoff, &search_pattern, &search_pattern, limit], |row| {
        let visit_time: f64 = row.get(2)?;
        Ok(HistoryItem {
            url: row.get(0)?,
            title: row.get(1)?,
            visit_time: core_data_to_iso(visit_time),
            visit_count: row.get(3)?,
            domain: row.get(4)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Get top visited sites by visit count.
pub fn query_top_sites(conn: &Connection, days: u32, limit: u32) -> Result<Vec<TopSite>> {
    let cutoff = days_ago_core_data(days);

    let mut stmt = conn.prepare(
        r#"
        SELECT
            hi.url,
            hi.domain_expansion,
            COUNT(*) as period_visits,
            MAX(hv.visit_time) as last_visit
        FROM history_visits hv
        JOIN history_items hi ON hv.history_item = hi.id
        WHERE hv.visit_time > ?
        GROUP BY hi.id
        ORDER BY period_visits DESC
        LIMIT ?
        "#,
    )?;

    let rows = stmt.query_map(params![cutoff, limit], |row| {
        let last_visit: f64 = row.get(3)?;
        Ok(TopSite {
            url: row.get(0)?,
            domain: row.get(1)?,
            visit_count: row.get(2)?,
            last_visit: core_data_to_iso(last_visit),
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Get visit count statistics.
pub fn query_history_stats(conn: &Connection, days: u32) -> Result<(i64, i64, i64)> {
    let cutoff = days_ago_core_data(days);

    let mut stmt = conn.prepare(
        r#"
        SELECT
            COUNT(*) as total_visits,
            COUNT(DISTINCT history_item) as unique_pages,
            COUNT(DISTINCT DATE(visit_time + 978307200, 'unixepoch')) as active_days
        FROM history_visits
        WHERE visit_time > ?
        "#,
    )?;

    stmt.query_row(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(Into::into)
}

// ============================================================================
// Cloud Tabs Queries
// ============================================================================

/// Get cloud tabs from all devices.
pub fn query_cloud_tabs(conn: &Connection) -> Result<Vec<CloudTab>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            ct.title,
            ct.url,
            cd.device_name,
            ct.is_pinned
        FROM cloud_tabs ct
        LEFT JOIN cloud_tab_devices cd ON ct.device_uuid = cd.device_uuid
        ORDER BY cd.device_name, ct.title
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CloudTab {
            title: row.get(0)?,
            url: row.get(1)?,
            device_name: row.get(2)?,
            is_pinned: row.get(3)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Get cloud tab devices with tab counts.
pub fn query_cloud_devices(conn: &Connection) -> Result<Vec<CloudDevice>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            cd.device_uuid,
            cd.device_name,
            COUNT(ct.tab_uuid) as tab_count
        FROM cloud_tab_devices cd
        LEFT JOIN cloud_tabs ct ON cd.device_uuid = ct.device_uuid
        GROUP BY cd.device_uuid
        ORDER BY cd.device_name
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CloudDevice {
            device_uuid: row.get(0)?,
            device_name: row.get(1)?,
            tab_count: row.get(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_conversion() {
        // 2024-01-01 00:00:00 UTC
        let unix_ts = 1704067200;
        let cd_ts = unix_to_core_data(unix_ts);
        let roundtrip = core_data_to_unix(cd_ts);
        assert_eq!(unix_ts, roundtrip);
    }

    #[test]
    fn test_days_ago() {
        let cutoff = days_ago_core_data(7);
        // Should be a reasonable Core Data timestamp (positive, not too large)
        assert!(cutoff > 0.0);
        assert!(cutoff < 1000000000.0);
    }
}
