//! Photos FGP daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use fgp_daemon::service::{FgpService, HealthStatus, MethodInfo, ParamInfo};
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::connection::open_photos_db;
use crate::db::queries::{self, AssetKind};

/// Photos daemon service with hot database connection.
pub struct PhotosService {
    conn: Mutex<Connection>,
}

impl PhotosService {
    /// Create new Photos service with hot connection.
    pub fn new() -> Result<Self> {
        let conn = Mutex::new(open_photos_db()?);
        Ok(Self { conn })
    }

    // ========================================================================
    // Parameter Helpers
    // ========================================================================

    fn get_param_u32(params: &HashMap<String, Value>, key: &str, default: u32) -> u32 {
        params
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(default)
    }

    fn get_param_i64(params: &HashMap<String, Value>, key: &str) -> Option<i64> {
        params.get(key).and_then(|v| v.as_i64())
    }

    fn get_param_f64(params: &HashMap<String, Value>, key: &str) -> Option<f64> {
        params.get(key).and_then(|v| v.as_f64())
    }

    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    fn parse_kind(params: &HashMap<String, Value>) -> Option<AssetKind> {
        params.get("kind").and_then(|v| v.as_str()).and_then(|s| {
            match s.to_lowercase().as_str() {
                "photo" | "photos" => Some(AssetKind::Photo),
                "video" | "videos" => Some(AssetKind::Video),
                _ => None,
            }
        })
    }

    // ========================================================================
    // Handlers
    // ========================================================================

    /// Get recent photos/videos.
    /// Params: days (default 30), limit (default 50), kind (optional: photo/video)
    fn recent(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 30);
        let limit = Self::get_param_u32(&params, "limit", 50);
        let kind = Self::parse_kind(&params);

        let assets = queries::query_recent_assets(&self.conn.lock().unwrap(), days, limit, kind)?;

        Ok(serde_json::json!({
            "assets": assets,
            "count": assets.len(),
            "days": days,
        }))
    }

    /// Get favorite photos.
    /// Params: limit (default 50)
    fn favorites(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 50);

        let assets = queries::query_favorites(&self.conn.lock().unwrap(), limit)?;

        Ok(serde_json::json!({
            "assets": assets,
            "count": assets.len(),
        }))
    }

    /// Search photos by date range.
    /// Params: start (required ISO date), end (required ISO date), limit (default 100)
    fn by_date(&self, params: HashMap<String, Value>) -> Result<Value> {
        let start = Self::get_param_str(&params, "start")
            .ok_or_else(|| anyhow!("Missing required parameter: start"))?;
        let end = Self::get_param_str(&params, "end")
            .ok_or_else(|| anyhow!("Missing required parameter: end"))?;
        let limit = Self::get_param_u32(&params, "limit", 100);

        let assets =
            queries::query_by_date_range(&self.conn.lock().unwrap(), start, end, limit)?;

        Ok(serde_json::json!({
            "assets": assets,
            "count": assets.len(),
            "start": start,
            "end": end,
        }))
    }

    /// Search photos near a location.
    /// Params: lat (required), lon (required), radius (default 1 km), limit (default 50)
    fn by_location(&self, params: HashMap<String, Value>) -> Result<Value> {
        let lat = Self::get_param_f64(&params, "lat")
            .ok_or_else(|| anyhow!("Missing required parameter: lat"))?;
        let lon = Self::get_param_f64(&params, "lon")
            .ok_or_else(|| anyhow!("Missing required parameter: lon"))?;
        let radius = Self::get_param_f64(&params, "radius").unwrap_or(1.0);
        let limit = Self::get_param_u32(&params, "limit", 50);

        let assets =
            queries::query_by_location(&self.conn.lock().unwrap(), lat, lon, radius, limit)?;

        Ok(serde_json::json!({
            "assets": assets,
            "count": assets.len(),
            "location": {
                "lat": lat,
                "lon": lon,
                "radius_km": radius,
            },
        }))
    }

    /// List albums.
    /// Params: limit (default 50)
    fn albums(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 50);

        let albums = queries::query_albums(&self.conn.lock().unwrap(), limit)?;

        Ok(serde_json::json!({
            "albums": albums,
            "count": albums.len(),
        }))
    }

    /// Get photos in an album.
    /// Params: album_id (required), limit (default 100)
    fn album_photos(&self, params: HashMap<String, Value>) -> Result<Value> {
        let album_id = Self::get_param_i64(&params, "album_id")
            .ok_or_else(|| anyhow!("Missing required parameter: album_id"))?;
        let limit = Self::get_param_u32(&params, "limit", 100);

        let assets = queries::query_album_assets(&self.conn.lock().unwrap(), album_id, limit)?;

        Ok(serde_json::json!({
            "assets": assets,
            "count": assets.len(),
            "album_id": album_id,
        }))
    }

    /// List recognized people.
    /// Params: limit (default 50)
    fn people(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 50);

        let people = queries::query_people(&self.conn.lock().unwrap(), limit)?;

        Ok(serde_json::json!({
            "people": people,
            "count": people.len(),
        }))
    }

    /// Get photos of a person.
    /// Params: person_id (required), limit (default 50)
    fn person_photos(&self, params: HashMap<String, Value>) -> Result<Value> {
        let person_id = Self::get_param_i64(&params, "person_id")
            .ok_or_else(|| anyhow!("Missing required parameter: person_id"))?;
        let limit = Self::get_param_u32(&params, "limit", 50);

        let assets = queries::query_person_photos(&self.conn.lock().unwrap(), person_id, limit)?;

        Ok(serde_json::json!({
            "assets": assets,
            "count": assets.len(),
            "person_id": person_id,
        }))
    }

    /// Get library statistics.
    fn stats(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let stats = queries::query_stats(&self.conn.lock().unwrap())?;

        Ok(serde_json::json!({
            "total_assets": stats.total_assets,
            "photos": stats.photos,
            "videos": stats.videos,
            "favorites": stats.favorites,
            "hidden": stats.hidden,
            "with_location": stats.with_location,
            "albums": stats.albums,
            "people": stats.people,
        }))
    }
}

impl FgpService for PhotosService {
    fn name(&self) -> &str {
        "photos"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "photos.recent" | "recent" => self.recent(params),
            "photos.favorites" | "favorites" => self.favorites(params),
            "photos.by_date" | "by_date" => self.by_date(params),
            "photos.by_location" | "by_location" => self.by_location(params),
            "photos.albums" | "albums" => self.albums(params),
            "photos.album_photos" | "album_photos" => self.album_photos(params),
            "photos.people" | "people" => self.people(params),
            "photos.person_photos" | "person_photos" => self.person_photos(params),
            "photos.stats" | "stats" => self.stats(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("recent", "Get recent photos/videos")
                .param(ParamInfo { name: "days".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(30.into())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) })
                .param(ParamInfo { name: "kind".into(), param_type: "string".into(), required: false, default: None }),
            MethodInfo::new("favorites", "Get favorited photos/videos")
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("by_date", "Search photos by date range")
                .param(ParamInfo { name: "start".into(), param_type: "string".into(), required: true, default: None })
                .param(ParamInfo { name: "end".into(), param_type: "string".into(), required: true, default: None })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(100.into())) }),
            MethodInfo::new("by_location", "Search photos near a location")
                .param(ParamInfo { name: "lat".into(), param_type: "number".into(), required: true, default: None })
                .param(ParamInfo { name: "lon".into(), param_type: "number".into(), required: true, default: None })
                .param(ParamInfo { name: "radius".into(), param_type: "number".into(), required: false, default: Some(Value::Number(serde_json::Number::from_f64(1.0).unwrap())) })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("albums", "List photo albums")
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("album_photos", "Get photos in an album")
                .param(ParamInfo { name: "album_id".into(), param_type: "integer".into(), required: true, default: None })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(100.into())) }),
            MethodInfo::new("people", "List recognized people")
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("person_photos", "Get photos of a person")
                .param(ParamInfo { name: "person_id".into(), param_type: "integer".into(), required: true, default: None })
                .param(ParamInfo { name: "limit".into(), param_type: "integer".into(), required: false, default: Some(Value::Number(50.into())) }),
            MethodInfo::new("stats", "Get library statistics"),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let stats = queries::query_stats(&self.conn.lock().unwrap());
        let (ok, msg) = match stats {
            Ok(s) => (
                true,
                format!("{} photos, {} videos in library", s.photos, s.videos),
            ),
            Err(e) => (false, format!("Database error: {}", e)),
        };

        checks.insert(
            "database".into(),
            HealthStatus {
                ok,
                latency_ms: None,
                message: Some(msg),
            },
        );

        checks
    }

    fn on_start(&self) -> Result<()> {
        let stats = queries::query_stats(&self.conn.lock().unwrap())?;
        tracing::info!(
            photos = stats.photos,
            videos = stats.videos,
            albums = stats.albums,
            people = stats.people,
            "Photos daemon starting - library loaded"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PhotosService;
    use fgp_daemon::service::FgpService;
    use rusqlite::Connection;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn test_service() -> PhotosService {
        let conn = Connection::open_in_memory().expect("in memory db");
        PhotosService {
            conn: Mutex::new(conn),
        }
    }

    #[test]
    fn get_param_helpers_read_values() {
        let mut params = HashMap::new();
        params.insert("days".to_string(), json!(12));
        params.insert("album_id".to_string(), json!(42));
        params.insert("lat".to_string(), json!(37.3));
        params.insert("start".to_string(), json!("2026-01-01"));

        assert_eq!(PhotosService::get_param_u32(&params, "days", 7), 12);
        assert_eq!(PhotosService::get_param_u32(&params, "missing", 7), 7);
        assert_eq!(PhotosService::get_param_i64(&params, "album_id"), Some(42));
        assert_eq!(PhotosService::get_param_i64(&params, "missing"), None);
        assert_eq!(PhotosService::get_param_f64(&params, "lat"), Some(37.3));
        assert_eq!(PhotosService::get_param_f64(&params, "missing"), None);
        assert_eq!(PhotosService::get_param_str(&params, "start"), Some("2026-01-01"));
        assert_eq!(PhotosService::get_param_str(&params, "missing"), None);
    }

    #[test]
    fn parse_kind_handles_known_and_unknown_values() {
        let mut params = HashMap::new();
        params.insert("kind".to_string(), json!("photo"));
        assert!(PhotosService::parse_kind(&params).is_some());

        params.insert("kind".to_string(), json!("videos"));
        assert!(PhotosService::parse_kind(&params).is_some());

        params.insert("kind".to_string(), json!("other"));
        assert!(PhotosService::parse_kind(&params).is_none());
    }

    #[test]
    fn method_list_includes_defaults_and_required_fields() {
        let methods = test_service().method_list();

        let recent_method = methods.iter().find(|m| m.name == "recent").expect("recent");
        let days_param = recent_method
            .params
            .iter()
            .find(|p| p.name == "days")
            .expect("days");
        let limit_param = recent_method
            .params
            .iter()
            .find(|p| p.name == "limit")
            .expect("limit");
        assert_eq!(days_param.default, Some(json!(30)));
        assert_eq!(limit_param.default, Some(json!(50)));

        let by_date_method = methods.iter().find(|m| m.name == "by_date").expect("by_date");
        let start_param = by_date_method
            .params
            .iter()
            .find(|p| p.name == "start")
            .expect("start");
        let end_param = by_date_method
            .params
            .iter()
            .find(|p| p.name == "end")
            .expect("end");
        assert!(start_param.required);
        assert!(end_param.required);

        let by_location_method = methods
            .iter()
            .find(|m| m.name == "by_location")
            .expect("by_location");
        let lat_param = by_location_method
            .params
            .iter()
            .find(|p| p.name == "lat")
            .expect("lat");
        let lon_param = by_location_method
            .params
            .iter()
            .find(|p| p.name == "lon")
            .expect("lon");
        assert!(lat_param.required);
        assert!(lon_param.required);

        let album_method = methods
            .iter()
            .find(|m| m.name == "album_photos")
            .expect("album_photos");
        let album_id = album_method
            .params
            .iter()
            .find(|p| p.name == "album_id")
            .expect("album_id");
        assert!(album_id.required);
    }

    #[test]
    fn dispatch_rejects_unknown_method() {
        let service = test_service();
        let err = service
            .dispatch("photos.nope", HashMap::new())
            .expect_err("unknown method");
        assert!(err.to_string().contains("Unknown method"));
    }
}
