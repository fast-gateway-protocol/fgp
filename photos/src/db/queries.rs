//! Photos database queries.
//!
//! Core Data timestamps are seconds since 2001-01-01 00:00:00 UTC.
//! GPS coordinates of -180.0 indicate no location data.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;

/// Core Data epoch: 2001-01-01 00:00:00 UTC
const CORE_DATA_EPOCH: i64 = 978307200;

/// Invalid GPS coordinate marker
const INVALID_GPS: f64 = -180.0;

/// Convert Core Data timestamp to ISO 8601 string.
pub fn core_data_to_iso(cd_ts: f64) -> String {
    let unix_ts = (cd_ts as i64) + CORE_DATA_EPOCH;
    chrono::DateTime::from_timestamp(unix_ts, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get Core Data timestamp for N days ago.
pub fn days_ago_core_data(days: u32) -> f64 {
    let now = chrono::Utc::now().timestamp();
    let cutoff = now - (days as i64 * 24 * 3600);
    (cutoff - CORE_DATA_EPOCH) as f64
}

// ============================================================================
// Photo/Asset Types
// ============================================================================

#[derive(Debug, Serialize, Clone)]
pub struct PhotoAsset {
    pub id: i64,
    pub uuid: String,
    pub filename: Option<String>,
    pub directory: Option<String>,
    pub date_created: String,
    pub kind: AssetKind,
    pub width: i64,
    pub height: i64,
    pub favorite: bool,
    pub hidden: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub duration: Option<f64>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub enum AssetKind {
    Photo,
    Video,
    Unknown,
}

impl From<i64> for AssetKind {
    fn from(kind: i64) -> Self {
        match kind {
            0 => AssetKind::Photo,
            1 => AssetKind::Video,
            _ => AssetKind::Unknown,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Album {
    pub id: i64,
    pub title: String,
    pub uuid: Option<String>,
    pub asset_count: i64,
    pub kind: i64,
    pub creation_date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Person {
    pub id: i64,
    pub display_name: Option<String>,
    pub face_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PhotoStats {
    pub total_assets: i64,
    pub photos: i64,
    pub videos: i64,
    pub favorites: i64,
    pub hidden: i64,
    pub with_location: i64,
    pub albums: i64,
    pub people: i64,
}

// ============================================================================
// Asset Queries
// ============================================================================

/// Get recent photos/videos.
pub fn query_recent_assets(conn: &Connection, days: u32, limit: u32, kind: Option<AssetKind>) -> Result<Vec<PhotoAsset>> {
    let cutoff = days_ago_core_data(days);

    let kind_filter = match kind {
        Some(AssetKind::Photo) => "AND ZKIND = 0",
        Some(AssetKind::Video) => "AND ZKIND = 1",
        _ => "",
    };

    let query = format!(
        r#"
        SELECT
            Z_PK,
            ZUUID,
            ZFILENAME,
            ZDIRECTORY,
            ZDATECREATED,
            ZKIND,
            ZWIDTH,
            ZHEIGHT,
            ZFAVORITE,
            ZHIDDEN,
            ZLATITUDE,
            ZLONGITUDE,
            ZDURATION
        FROM ZASSET
        WHERE ZTRASHEDSTATE = 0
          AND ZVISIBILITYSTATE = 0
          AND ZDATECREATED > ?
          {}
        ORDER BY ZDATECREATED DESC
        LIMIT ?
        "#,
        kind_filter
    );

    let mut stmt = conn.prepare(&query)?;

    let assets = stmt
        .query_map(params![cutoff, limit], |row| {
            let lat: f64 = row.get(10)?;
            let lon: f64 = row.get(11)?;
            let date: f64 = row.get(4)?;

            Ok(PhotoAsset {
                id: row.get(0)?,
                uuid: row.get(1)?,
                filename: row.get(2)?,
                directory: row.get(3)?,
                date_created: core_data_to_iso(date),
                kind: AssetKind::from(row.get::<_, i64>(5)?),
                width: row.get(6)?,
                height: row.get(7)?,
                favorite: row.get::<_, i64>(8)? == 1,
                hidden: row.get::<_, i64>(9)? == 1,
                latitude: if lat != INVALID_GPS { Some(lat) } else { None },
                longitude: if lon != INVALID_GPS { Some(lon) } else { None },
                duration: row.get(12).ok(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

/// Get favorite photos/videos.
pub fn query_favorites(conn: &Connection, limit: u32) -> Result<Vec<PhotoAsset>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZUUID,
            ZFILENAME,
            ZDIRECTORY,
            ZDATECREATED,
            ZKIND,
            ZWIDTH,
            ZHEIGHT,
            ZFAVORITE,
            ZHIDDEN,
            ZLATITUDE,
            ZLONGITUDE,
            ZDURATION
        FROM ZASSET
        WHERE ZTRASHEDSTATE = 0
          AND ZVISIBILITYSTATE = 0
          AND ZFAVORITE = 1
        ORDER BY ZDATECREATED DESC
        LIMIT ?
        "#,
    )?;

    let assets = stmt
        .query_map(params![limit], |row| {
            let lat: f64 = row.get(10)?;
            let lon: f64 = row.get(11)?;
            let date: f64 = row.get(4)?;

            Ok(PhotoAsset {
                id: row.get(0)?,
                uuid: row.get(1)?,
                filename: row.get(2)?,
                directory: row.get(3)?,
                date_created: core_data_to_iso(date),
                kind: AssetKind::from(row.get::<_, i64>(5)?),
                width: row.get(6)?,
                height: row.get(7)?,
                favorite: true,
                hidden: row.get::<_, i64>(9)? == 1,
                latitude: if lat != INVALID_GPS { Some(lat) } else { None },
                longitude: if lon != INVALID_GPS { Some(lon) } else { None },
                duration: row.get(12).ok(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

/// Search photos by date range.
pub fn query_by_date_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
    limit: u32,
) -> Result<Vec<PhotoAsset>> {
    // Parse ISO dates to Core Data timestamps
    let start_ts = chrono::DateTime::parse_from_rfc3339(start_date)
        .map(|dt| (dt.timestamp() - CORE_DATA_EPOCH) as f64)
        .unwrap_or(0.0);

    let end_ts = chrono::DateTime::parse_from_rfc3339(end_date)
        .map(|dt| (dt.timestamp() - CORE_DATA_EPOCH) as f64)
        .unwrap_or(f64::MAX);

    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZUUID,
            ZFILENAME,
            ZDIRECTORY,
            ZDATECREATED,
            ZKIND,
            ZWIDTH,
            ZHEIGHT,
            ZFAVORITE,
            ZHIDDEN,
            ZLATITUDE,
            ZLONGITUDE,
            ZDURATION
        FROM ZASSET
        WHERE ZTRASHEDSTATE = 0
          AND ZVISIBILITYSTATE = 0
          AND ZDATECREATED >= ?
          AND ZDATECREATED <= ?
        ORDER BY ZDATECREATED DESC
        LIMIT ?
        "#,
    )?;

    let assets = stmt
        .query_map(params![start_ts, end_ts, limit], |row| {
            let lat: f64 = row.get(10)?;
            let lon: f64 = row.get(11)?;
            let date: f64 = row.get(4)?;

            Ok(PhotoAsset {
                id: row.get(0)?,
                uuid: row.get(1)?,
                filename: row.get(2)?,
                directory: row.get(3)?,
                date_created: core_data_to_iso(date),
                kind: AssetKind::from(row.get::<_, i64>(5)?),
                width: row.get(6)?,
                height: row.get(7)?,
                favorite: row.get::<_, i64>(8)? == 1,
                hidden: row.get::<_, i64>(9)? == 1,
                latitude: if lat != INVALID_GPS { Some(lat) } else { None },
                longitude: if lon != INVALID_GPS { Some(lon) } else { None },
                duration: row.get(12).ok(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

/// Search photos near a location (within radius_km).
pub fn query_by_location(
    conn: &Connection,
    lat: f64,
    lon: f64,
    radius_km: f64,
    limit: u32,
) -> Result<Vec<PhotoAsset>> {
    // Approximate degrees per km at the given latitude
    let lat_deg_per_km = 1.0 / 111.0;
    let lon_deg_per_km = 1.0 / (111.0 * (lat.to_radians().cos()));

    let lat_delta = radius_km * lat_deg_per_km;
    let lon_delta = radius_km * lon_deg_per_km;

    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZUUID,
            ZFILENAME,
            ZDIRECTORY,
            ZDATECREATED,
            ZKIND,
            ZWIDTH,
            ZHEIGHT,
            ZFAVORITE,
            ZHIDDEN,
            ZLATITUDE,
            ZLONGITUDE,
            ZDURATION
        FROM ZASSET
        WHERE ZTRASHEDSTATE = 0
          AND ZVISIBILITYSTATE = 0
          AND ZLATITUDE != -180.0
          AND ZLATITUDE BETWEEN ? AND ?
          AND ZLONGITUDE BETWEEN ? AND ?
        ORDER BY ZDATECREATED DESC
        LIMIT ?
        "#,
    )?;

    let assets = stmt
        .query_map(
            params![
                lat - lat_delta,
                lat + lat_delta,
                lon - lon_delta,
                lon + lon_delta,
                limit
            ],
            |row| {
                let lat: f64 = row.get(10)?;
                let lon: f64 = row.get(11)?;
                let date: f64 = row.get(4)?;

                Ok(PhotoAsset {
                    id: row.get(0)?,
                    uuid: row.get(1)?,
                    filename: row.get(2)?,
                    directory: row.get(3)?,
                    date_created: core_data_to_iso(date),
                    kind: AssetKind::from(row.get::<_, i64>(5)?),
                    width: row.get(6)?,
                    height: row.get(7)?,
                    favorite: row.get::<_, i64>(8)? == 1,
                    hidden: row.get::<_, i64>(9)? == 1,
                    latitude: Some(lat),
                    longitude: Some(lon),
                    duration: row.get(12).ok(),
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

// ============================================================================
// Album Queries
// ============================================================================

/// Get all user albums.
pub fn query_albums(conn: &Connection, limit: u32) -> Result<Vec<Album>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZTITLE,
            ZUUID,
            ZCACHEDCOUNT,
            ZKIND,
            ZCREATIONDATE
        FROM ZGENERICALBUM
        WHERE ZTITLE IS NOT NULL
          AND ZTRASHEDSTATE = 0
          AND ZKIND = 2
        ORDER BY ZCACHEDCOUNT DESC
        LIMIT ?
        "#,
    )?;

    let albums = stmt
        .query_map(params![limit], |row| {
            let creation_date: Option<f64> = row.get(5).ok();
            Ok(Album {
                id: row.get(0)?,
                title: row.get(1)?,
                uuid: row.get(2)?,
                asset_count: row.get(3)?,
                kind: row.get(4)?,
                creation_date: creation_date.map(core_data_to_iso),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(albums)
}

/// Get assets in a specific album.
pub fn query_album_assets(conn: &Connection, album_id: i64, limit: u32) -> Result<Vec<PhotoAsset>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            a.Z_PK,
            a.ZUUID,
            a.ZFILENAME,
            a.ZDIRECTORY,
            a.ZDATECREATED,
            a.ZKIND,
            a.ZWIDTH,
            a.ZHEIGHT,
            a.ZFAVORITE,
            a.ZHIDDEN,
            a.ZLATITUDE,
            a.ZLONGITUDE,
            a.ZDURATION
        FROM ZASSET a
        JOIN Z_33ASSETS j ON j.Z_3ASSETS = a.Z_PK
        WHERE j.Z_33ALBUMS = ?
          AND a.ZTRASHEDSTATE = 0
          AND a.ZVISIBILITYSTATE = 0
        ORDER BY a.ZDATECREATED DESC
        LIMIT ?
        "#,
    )?;

    let assets = stmt
        .query_map(params![album_id, limit], |row| {
            let lat: f64 = row.get(10)?;
            let lon: f64 = row.get(11)?;
            let date: f64 = row.get(4)?;

            Ok(PhotoAsset {
                id: row.get(0)?,
                uuid: row.get(1)?,
                filename: row.get(2)?,
                directory: row.get(3)?,
                date_created: core_data_to_iso(date),
                kind: AssetKind::from(row.get::<_, i64>(5)?),
                width: row.get(6)?,
                height: row.get(7)?,
                favorite: row.get::<_, i64>(8)? == 1,
                hidden: row.get::<_, i64>(9)? == 1,
                latitude: if lat != INVALID_GPS { Some(lat) } else { None },
                longitude: if lon != INVALID_GPS { Some(lon) } else { None },
                duration: row.get(12).ok(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

// ============================================================================
// People Queries
// ============================================================================

/// Get recognized people.
pub fn query_people(conn: &Connection, limit: u32) -> Result<Vec<Person>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZDISPLAYNAME,
            ZFACECOUNT
        FROM ZPERSON
        WHERE ZFACECOUNT > 0
        ORDER BY ZFACECOUNT DESC
        LIMIT ?
        "#,
    )?;

    let people = stmt
        .query_map(params![limit], |row| {
            Ok(Person {
                id: row.get(0)?,
                display_name: row.get(1)?,
                face_count: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(people)
}

/// Get photos of a specific person.
pub fn query_person_photos(conn: &Connection, person_id: i64, limit: u32) -> Result<Vec<PhotoAsset>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT DISTINCT
            a.Z_PK,
            a.ZUUID,
            a.ZFILENAME,
            a.ZDIRECTORY,
            a.ZDATECREATED,
            a.ZKIND,
            a.ZWIDTH,
            a.ZHEIGHT,
            a.ZFAVORITE,
            a.ZHIDDEN,
            a.ZLATITUDE,
            a.ZLONGITUDE,
            a.ZDURATION
        FROM ZASSET a
        JOIN ZDETECTEDFACE f ON f.ZASSETFORFACE = a.Z_PK
        WHERE f.ZPERSONFORFACE = ?
          AND a.ZTRASHEDSTATE = 0
          AND a.ZVISIBILITYSTATE = 0
        ORDER BY a.ZDATECREATED DESC
        LIMIT ?
        "#,
    )?;

    let assets = stmt
        .query_map(params![person_id, limit], |row| {
            let lat: f64 = row.get(10)?;
            let lon: f64 = row.get(11)?;
            let date: f64 = row.get(4)?;

            Ok(PhotoAsset {
                id: row.get(0)?,
                uuid: row.get(1)?,
                filename: row.get(2)?,
                directory: row.get(3)?,
                date_created: core_data_to_iso(date),
                kind: AssetKind::from(row.get::<_, i64>(5)?),
                width: row.get(6)?,
                height: row.get(7)?,
                favorite: row.get::<_, i64>(8)? == 1,
                hidden: row.get::<_, i64>(9)? == 1,
                latitude: if lat != INVALID_GPS { Some(lat) } else { None },
                longitude: if lon != INVALID_GPS { Some(lon) } else { None },
                duration: row.get(12).ok(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(assets)
}

// ============================================================================
// Statistics Queries
// ============================================================================

/// Get library statistics.
pub fn query_stats(conn: &Connection) -> Result<PhotoStats> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZASSET WHERE ZTRASHEDSTATE = 0 AND ZVISIBILITYSTATE = 0",
        [],
        |row| row.get(0),
    )?;

    let photos: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZASSET WHERE ZTRASHEDSTATE = 0 AND ZVISIBILITYSTATE = 0 AND ZKIND = 0",
        [],
        |row| row.get(0),
    )?;

    let videos: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZASSET WHERE ZTRASHEDSTATE = 0 AND ZVISIBILITYSTATE = 0 AND ZKIND = 1",
        [],
        |row| row.get(0),
    )?;

    let favorites: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZASSET WHERE ZTRASHEDSTATE = 0 AND ZVISIBILITYSTATE = 0 AND ZFAVORITE = 1",
        [],
        |row| row.get(0),
    )?;

    let hidden: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZASSET WHERE ZTRASHEDSTATE = 0 AND ZHIDDEN = 1",
        [],
        |row| row.get(0),
    )?;

    let with_location: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZASSET WHERE ZTRASHEDSTATE = 0 AND ZVISIBILITYSTATE = 0 AND ZLATITUDE != -180.0",
        [],
        |row| row.get(0),
    )?;

    let albums: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZGENERICALBUM WHERE ZTITLE IS NOT NULL AND ZTRASHEDSTATE = 0 AND ZKIND = 2",
        [],
        |row| row.get(0),
    )?;

    let people: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZPERSON WHERE ZFACECOUNT > 0",
        [],
        |row| row.get(0),
    )?;

    Ok(PhotoStats {
        total_assets: total,
        photos,
        videos,
        favorites,
        hidden,
        with_location,
        albums,
        people,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_asset_kind_conversion() {
        assert_eq!(AssetKind::from(0), AssetKind::Photo);
        assert_eq!(AssetKind::from(1), AssetKind::Video);
        assert_eq!(AssetKind::from(99), AssetKind::Unknown);
    }

    #[test]
    fn test_days_ago() {
        let cutoff = days_ago_core_data(7);
        assert!(cutoff > 0.0);
    }

    #[test]
    fn test_query_stats_counts() {
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute_batch(
            r#"
            CREATE TABLE ZASSET (
                ZTRASHEDSTATE INTEGER,
                ZVISIBILITYSTATE INTEGER,
                ZKIND INTEGER,
                ZFAVORITE INTEGER,
                ZHIDDEN INTEGER,
                ZLATITUDE REAL
            );
            CREATE TABLE ZGENERICALBUM (
                ZTITLE TEXT,
                ZTRASHEDSTATE INTEGER,
                ZKIND INTEGER
            );
            CREATE TABLE ZPERSON (
                ZFACECOUNT INTEGER
            );
            "#,
        )
        .expect("create schema");

        conn.execute(
            "INSERT INTO ZASSET (ZTRASHEDSTATE, ZVISIBILITYSTATE, ZKIND, ZFAVORITE, ZHIDDEN, ZLATITUDE) VALUES (0, 0, 0, 1, 0, 12.0)",
            [],
        )
        .expect("insert asset 1");
        conn.execute(
            "INSERT INTO ZASSET (ZTRASHEDSTATE, ZVISIBILITYSTATE, ZKIND, ZFAVORITE, ZHIDDEN, ZLATITUDE) VALUES (0, 0, 1, 0, 0, -180.0)",
            [],
        )
        .expect("insert asset 2");
        conn.execute(
            "INSERT INTO ZASSET (ZTRASHEDSTATE, ZVISIBILITYSTATE, ZKIND, ZFAVORITE, ZHIDDEN, ZLATITUDE) VALUES (1, 0, 0, 0, 1, 0.0)",
            [],
        )
        .expect("insert asset 3");

        conn.execute(
            "INSERT INTO ZGENERICALBUM (ZTITLE, ZTRASHEDSTATE, ZKIND) VALUES ('Album', 0, 2)",
            [],
        )
        .expect("insert album");
        conn.execute("INSERT INTO ZPERSON (ZFACECOUNT) VALUES (3)", [])
            .expect("insert person");

        let stats = query_stats(&conn).expect("stats");

        assert_eq!(stats.total_assets, 2);
        assert_eq!(stats.photos, 1);
        assert_eq!(stats.videos, 1);
        assert_eq!(stats.favorites, 1);
        assert_eq!(stats.hidden, 0);
        assert_eq!(stats.with_location, 1);
        assert_eq!(stats.albums, 1);
        assert_eq!(stats.people, 1);
    }
}
