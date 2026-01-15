//! Notes FGP daemon service implementation.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use fgp_daemon::service::{FgpService, HealthStatus, MethodInfo, ParamInfo};
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::connection::open_notes_db;
use crate::db::queries;

/// Notes daemon service with hot database connection.
pub struct NotesService {
    conn: Mutex<Connection>,
}

impl NotesService {
    /// Create new Notes service with hot connection.
    pub fn new() -> Result<Self> {
        let conn = Mutex::new(open_notes_db()?);
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

    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    // ========================================================================
    // Handlers
    // ========================================================================

    /// List notes.
    fn list(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 50);
        let notes = queries::query_notes(&self.conn.lock().unwrap(), limit)?;

        Ok(serde_json::json!({
            "notes": notes,
            "count": notes.len(),
        }))
    }

    /// Get recent notes.
    fn recent(&self, params: HashMap<String, Value>) -> Result<Value> {
        let days = Self::get_param_u32(&params, "days", 7);
        let limit = Self::get_param_u32(&params, "limit", 50);
        let notes = queries::query_recent_notes(&self.conn.lock().unwrap(), days, limit)?;

        Ok(serde_json::json!({
            "notes": notes,
            "count": notes.len(),
            "days": days,
        }))
    }

    /// Search notes.
    fn search(&self, params: HashMap<String, Value>) -> Result<Value> {
        let query = Self::get_param_str(&params, "query")
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;
        let limit = Self::get_param_u32(&params, "limit", 50);
        let notes = queries::search_notes(&self.conn.lock().unwrap(), query, limit)?;

        Ok(serde_json::json!({
            "notes": notes,
            "count": notes.len(),
            "query": query,
        }))
    }

    /// Read a specific note.
    fn read(&self, params: HashMap<String, Value>) -> Result<Value> {
        let note_id = Self::get_param_i64(&params, "id")
            .ok_or_else(|| anyhow!("Missing required parameter: id"))?;
        let note = queries::get_note(&self.conn.lock().unwrap(), note_id)?;

        match note {
            Some(n) => Ok(serde_json::json!({
                "note": n,
            })),
            None => Err(anyhow!("Note not found: {}", note_id)),
        }
    }

    /// Get notes in a folder.
    fn by_folder(&self, params: HashMap<String, Value>) -> Result<Value> {
        let folder = Self::get_param_str(&params, "folder")
            .ok_or_else(|| anyhow!("Missing required parameter: folder"))?;
        let limit = Self::get_param_u32(&params, "limit", 50);
        let notes = queries::query_folder_notes(&self.conn.lock().unwrap(), folder, limit)?;

        Ok(serde_json::json!({
            "notes": notes,
            "count": notes.len(),
            "folder": folder,
        }))
    }

    /// Get pinned notes.
    fn pinned(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 50);
        let notes = queries::query_pinned_notes(&self.conn.lock().unwrap(), limit)?;

        Ok(serde_json::json!({
            "notes": notes,
            "count": notes.len(),
        }))
    }

    /// List folders.
    fn folders(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_u32(&params, "limit", 50);
        let folders = queries::query_folders(&self.conn.lock().unwrap(), limit)?;

        Ok(serde_json::json!({
            "folders": folders,
            "count": folders.len(),
        }))
    }

    /// Get library statistics.
    fn stats(&self, _params: HashMap<String, Value>) -> Result<Value> {
        let stats = queries::query_stats(&self.conn.lock().unwrap())?;

        Ok(serde_json::json!({
            "total_notes": stats.total_notes,
            "total_folders": stats.total_folders,
            "notes_with_checklists": stats.notes_with_checklists,
            "pinned_notes": stats.pinned_notes,
        }))
    }
}

impl FgpService for NotesService {
    fn name(&self) -> &str {
        "notes"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "notes.list" | "list" => self.list(params),
            "notes.recent" | "recent" => self.recent(params),
            "notes.search" | "search" => self.search(params),
            "notes.read" | "read" => self.read(params),
            "notes.by_folder" | "by_folder" => self.by_folder(params),
            "notes.pinned" | "pinned" => self.pinned(params),
            "notes.folders" | "folders" => self.folders(params),
            "notes.stats" | "stats" => self.stats(params),
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo {
                name: "list".into(),
                description: "List all notes".into(),
                params: vec![ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(50.into())),
                }],
            },
            MethodInfo {
                name: "recent".into(),
                description: "Get recently modified notes".into(),
                params: vec![
                    ParamInfo {
                        name: "days".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(7.into())),
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(50.into())),
                    },
                ],
            },
            MethodInfo {
                name: "search".into(),
                description: "Search notes by title/content".into(),
                params: vec![
                    ParamInfo {
                        name: "query".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(50.into())),
                    },
                ],
            },
            MethodInfo {
                name: "read".into(),
                description: "Read a specific note by ID".into(),
                params: vec![ParamInfo {
                    name: "id".into(),
                    param_type: "integer".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "by_folder".into(),
                description: "Get notes in a folder".into(),
                params: vec![
                    ParamInfo {
                        name: "folder".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(Value::Number(50.into())),
                    },
                ],
            },
            MethodInfo {
                name: "pinned".into(),
                description: "Get pinned notes".into(),
                params: vec![ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(50.into())),
                }],
            },
            MethodInfo {
                name: "folders".into(),
                description: "List folders".into(),
                params: vec![ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(Value::Number(50.into())),
                }],
            },
            MethodInfo {
                name: "stats".into(),
                description: "Get library statistics".into(),
                params: vec![],
            },
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let stats = queries::query_stats(&self.conn.lock().unwrap());
        let (ok, msg) = match stats {
            Ok(s) => (
                true,
                format!(
                    "{} notes, {} folders in library",
                    s.total_notes, s.total_folders
                ),
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
            notes = stats.total_notes,
            folders = stats.total_folders,
            pinned = stats.pinned_notes,
            "Notes daemon starting - library loaded"
        );
        Ok(())
    }
}
