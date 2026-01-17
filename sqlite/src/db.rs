//! Database connection manager for SQLite.
//!
//! Manages a pool of SQLite connections keyed by database path.
//! Thread-safe access using parking_lot::Mutex per connection.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use parking_lot::Mutex;
use rusqlite::{params_from_iter, Connection, OpenFlags, Row, types::ValueRef};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A thread-safe wrapper around a SQLite connection.
struct SafeConnection {
    conn: Mutex<Connection>,
}

/// Thread-safe database connection manager.
pub struct ConnectionManager {
    /// Open connections keyed by canonical path
    connections: Mutex<HashMap<PathBuf, Arc<SafeConnection>>>,
}

impl ConnectionManager {
    /// Create a new connection manager.
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Expand tilde and canonicalize path.
    fn resolve_path(path: &str) -> Result<PathBuf> {
        let expanded = shellexpand::tilde(path).to_string();
        let path = Path::new(&expanded);

        // For new databases, we can't canonicalize yet - just expand
        if path.exists() {
            path.canonicalize()
                .with_context(|| format!("Failed to resolve path: {}", expanded))
        } else {
            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {:?}", parent))?;
                }
            }
            Ok(path.to_path_buf())
        }
    }

    /// Open or get an existing connection to a database.
    pub fn open(&self, path: &str) -> Result<Value> {
        let expanded = shellexpand::tilde(path).to_string();
        let path_buf = Path::new(&expanded).to_path_buf();

        // Create parent directory if needed
        if let Some(parent) = path_buf.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {:?}", parent))?;
            }
        }

        // If file exists, canonicalize for lookup
        let lookup_path = if path_buf.exists() {
            path_buf.canonicalize().unwrap_or_else(|_| path_buf.clone())
        } else {
            path_buf.clone()
        };

        // Check if already open
        {
            let connections = self.connections.lock();
            if connections.contains_key(&lookup_path) {
                return Ok(json!({
                    "path": lookup_path.to_string_lossy(),
                    "status": "already_open"
                }));
            }
        }

        // Open new connection (this creates the file if it doesn't exist)
        let conn = Connection::open_with_flags(
            &path_buf,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| format!("Failed to open database: {:?}", path_buf))?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .context("Failed to set WAL mode")?;

        // Now that the file exists, get the canonical path for storage
        let canonical = path_buf
            .canonicalize()
            .unwrap_or_else(|_| path_buf.clone());

        // Store connection with canonical path
        {
            let mut connections = self.connections.lock();
            connections.insert(
                canonical.clone(),
                Arc::new(SafeConnection {
                    conn: Mutex::new(conn),
                }),
            );
        }

        Ok(json!({
            "path": canonical.to_string_lossy(),
            "status": "opened"
        }))
    }

    /// Close a database connection.
    pub fn close(&self, path: &str) -> Result<Value> {
        let resolved = Self::resolve_path(path)?;

        let mut connections = self.connections.lock();
        if connections.remove(&resolved).is_some() {
            Ok(json!({
                "path": resolved.to_string_lossy(),
                "status": "closed"
            }))
        } else {
            Ok(json!({
                "path": resolved.to_string_lossy(),
                "status": "not_open"
            }))
        }
    }

    /// Get a connection, opening it if needed.
    fn get_connection(&self, path: &str) -> Result<Arc<SafeConnection>> {
        let resolved = Self::resolve_path(path)?;

        // Check if already open
        {
            let connections = self.connections.lock();
            if let Some(conn) = connections.get(&resolved) {
                return Ok(conn.clone());
            }
        }

        // Open new connection
        let conn = Connection::open_with_flags(
            &resolved,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| format!("Failed to open database: {:?}", resolved))?;

        // Enable WAL mode
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .context("Failed to set WAL mode")?;

        let safe_conn = Arc::new(SafeConnection {
            conn: Mutex::new(conn),
        });

        // Store connection
        {
            let mut connections = self.connections.lock();
            connections.insert(resolved, safe_conn.clone());
        }

        Ok(safe_conn)
    }

    /// Execute a query and return results as JSON.
    pub fn query(&self, path: &str, sql: &str, params: &[Value]) -> Result<Value> {
        let conn_wrapper = self.get_connection(path)?;
        let conn = conn_wrapper.conn.lock();

        let mut stmt = conn.prepare(sql).context("Failed to prepare statement")?;

        // Get column names
        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(String::from)
            .collect();
        let column_count = column_names.len();

        // Convert params to rusqlite-compatible types
        let sqlite_params = Self::convert_params(params);

        // Execute query
        let rows = stmt
            .query_map(params_from_iter(sqlite_params.iter()), |row| {
                Ok(Self::row_to_json(row, &column_names, column_count))
            })
            .context("Failed to execute query")?;

        // Collect results
        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result.context("Failed to read row")?);
        }

        Ok(json!({
            "columns": column_names,
            "rows": results,
            "count": results.len()
        }))
    }

    /// Execute a statement (INSERT/UPDATE/DELETE) and return affected rows.
    pub fn execute(&self, path: &str, sql: &str, params: &[Value]) -> Result<Value> {
        let conn_wrapper = self.get_connection(path)?;
        let conn = conn_wrapper.conn.lock();

        // Convert params to rusqlite-compatible types
        let sqlite_params = Self::convert_params(params);

        let affected = conn
            .execute(sql, params_from_iter(sqlite_params.iter()))
            .context("Failed to execute statement")?;

        let last_insert_id = conn.last_insert_rowid();

        Ok(json!({
            "affected_rows": affected,
            "last_insert_id": last_insert_id
        }))
    }

    /// List tables in the database.
    pub fn tables(&self, path: &str) -> Result<Value> {
        let conn_wrapper = self.get_connection(path)?;
        let conn = conn_wrapper.conn.lock();

        let mut stmt = conn
            .prepare(
                "SELECT name, type FROM sqlite_master
                 WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'
                 ORDER BY type, name",
            )
            .context("Failed to prepare statement")?;

        let tables: Vec<Value> = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let obj_type: String = row.get(1)?;
                Ok(json!({
                    "name": name,
                    "type": obj_type
                }))
            })
            .context("Failed to query tables")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(json!({
            "tables": tables,
            "count": tables.len()
        }))
    }

    /// Get schema for a table.
    pub fn schema(&self, path: &str, table: &str) -> Result<Value> {
        let conn_wrapper = self.get_connection(path)?;
        let conn = conn_wrapper.conn.lock();

        // Get column info using PRAGMA
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info(\"{}\")", table))
            .context("Failed to prepare PRAGMA")?;

        let columns: Vec<Value> = stmt
            .query_map([], |row| {
                let cid: i32 = row.get(0)?;
                let name: String = row.get(1)?;
                let col_type: String = row.get(2)?;
                let notnull: i32 = row.get(3)?;
                let default: Option<String> = row.get(4)?;
                let pk: i32 = row.get(5)?;

                Ok(json!({
                    "cid": cid,
                    "name": name,
                    "type": col_type,
                    "notnull": notnull != 0,
                    "default": default,
                    "primary_key": pk != 0
                }))
            })
            .context("Failed to query schema")?
            .filter_map(|r| r.ok())
            .collect();

        if columns.is_empty() {
            anyhow::bail!("Table not found: {}", table);
        }

        // Get indexes
        let mut idx_stmt = conn
            .prepare(&format!("PRAGMA index_list(\"{}\")", table))
            .context("Failed to prepare index PRAGMA")?;

        let indexes: Vec<Value> = idx_stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let unique: i32 = row.get(2)?;
                let origin: String = row.get(3)?;
                Ok(json!({
                    "name": name,
                    "unique": unique != 0,
                    "origin": origin
                }))
            })
            .context("Failed to query indexes")?
            .filter_map(|r| r.ok())
            .collect();

        // Get the CREATE statement
        let create_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name=?",
                [table],
                |row| row.get(0),
            )
            .ok();

        Ok(json!({
            "table": table,
            "columns": columns,
            "indexes": indexes,
            "sql": create_sql
        }))
    }

    /// Get connection count (for health checks).
    pub fn connection_count(&self) -> usize {
        self.connections.lock().len()
    }

    /// List open databases.
    pub fn list_open(&self) -> Vec<String> {
        self.connections
            .lock()
            .keys()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Convert JSON values to rusqlite-compatible SqliteValue enum.
    fn convert_params(params: &[Value]) -> Vec<SqliteValue> {
        params.iter().map(Self::value_to_sqlite).collect()
    }

    /// Convert a JSON value to a rusqlite-compatible SqliteValue.
    fn value_to_sqlite(value: &Value) -> SqliteValue {
        match value {
            Value::Null => SqliteValue::Null,
            Value::Bool(b) => SqliteValue::Integer(if *b { 1 } else { 0 }),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    SqliteValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    SqliteValue::Real(f)
                } else {
                    SqliteValue::Text(n.to_string())
                }
            }
            Value::String(s) => SqliteValue::Text(s.clone()),
            Value::Array(_) | Value::Object(_) => {
                // Serialize complex types as JSON strings
                SqliteValue::Text(serde_json::to_string(value).unwrap_or_default())
            }
        }
    }

    /// Convert a SQLite row to JSON.
    fn row_to_json(row: &Row, column_names: &[String], column_count: usize) -> Value {
        let mut obj = serde_json::Map::new();

        for i in 0..column_count {
            let name = &column_names[i];
            let value = Self::column_to_json(row, i);
            obj.insert(name.clone(), value);
        }

        Value::Object(obj)
    }

    /// Convert a SQLite column value to JSON.
    fn column_to_json(row: &Row, idx: usize) -> Value {
        // Use ValueRef to get the raw SQLite type
        match row.get_ref(idx) {
            Ok(ValueRef::Null) => Value::Null,
            Ok(ValueRef::Integer(i)) => json!(i),
            Ok(ValueRef::Real(f)) => json!(f),
            Ok(ValueRef::Text(bytes)) => {
                match std::str::from_utf8(bytes) {
                    Ok(s) => json!(s),
                    Err(_) => json!(BASE64.encode(bytes)),
                }
            }
            Ok(ValueRef::Blob(bytes)) => {
                // Encode BLOB as base64
                json!({
                    "_type": "blob",
                    "encoding": "base64",
                    "data": BASE64.encode(bytes)
                })
            }
            Err(_) => Value::Null,
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Enum to hold SQLite-compatible values that implement ToSql.
#[derive(Debug, Clone)]
enum SqliteValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
}

impl rusqlite::ToSql for SqliteValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use rusqlite::types::ToSqlOutput;
        match self {
            SqliteValue::Null => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Null)),
            SqliteValue::Integer(i) => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Integer(*i))),
            SqliteValue::Real(f) => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Real(*f))),
            SqliteValue::Text(s) => Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(s.clone()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_open_and_close() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let path_str = db_path.to_string_lossy().to_string();

        let manager = ConnectionManager::new();

        // Open database
        let result = manager.open(&path_str).unwrap();
        assert_eq!(result["status"], "opened");

        // Get the resolved path from the result for subsequent calls
        let resolved_path = result["path"].as_str().unwrap().to_string();

        // Already open - use the resolved path
        let result = manager.open(&resolved_path).unwrap();
        assert_eq!(result["status"], "already_open");

        // Close
        let result = manager.close(&resolved_path).unwrap();
        assert_eq!(result["status"], "closed");

        // Close again
        let result = manager.close(&resolved_path).unwrap();
        assert_eq!(result["status"], "not_open");
    }

    #[test]
    fn test_query_and_execute() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let path_str = db_path.to_string_lossy().to_string();

        let manager = ConnectionManager::new();
        manager.open(&path_str).unwrap();

        // Create table
        manager
            .execute(
                &path_str,
                "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)",
                &[],
            )
            .unwrap();

        // Insert
        let result = manager
            .execute(
                &path_str,
                "INSERT INTO users (name, age) VALUES (?, ?)",
                &[json!("Alice"), json!(30)],
            )
            .unwrap();
        assert_eq!(result["affected_rows"], 1);
        assert_eq!(result["last_insert_id"], 1);

        // Insert another
        manager
            .execute(
                &path_str,
                "INSERT INTO users (name, age) VALUES (?, ?)",
                &[json!("Bob"), json!(25)],
            )
            .unwrap();

        // Query
        let result = manager
            .query(&path_str, "SELECT * FROM users ORDER BY id", &[])
            .unwrap();
        assert_eq!(result["count"], 2);

        let rows = result["rows"].as_array().unwrap();
        assert_eq!(rows[0]["name"], "Alice");
        assert_eq!(rows[0]["age"], 30);
        assert_eq!(rows[1]["name"], "Bob");
    }

    #[test]
    fn test_tables_and_schema() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let path_str = db_path.to_string_lossy().to_string();

        let manager = ConnectionManager::new();
        manager.open(&path_str).unwrap();

        manager
            .execute(
                &path_str,
                "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
                &[],
            )
            .unwrap();

        // List tables
        let result = manager.tables(&path_str).unwrap();
        assert_eq!(result["count"], 1);
        let tables = result["tables"].as_array().unwrap();
        assert_eq!(tables[0]["name"], "users");

        // Get schema
        let result = manager.schema(&path_str, "users").unwrap();
        let columns = result["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0]["name"], "id");
        assert!(columns[0]["primary_key"].as_bool().unwrap());
        assert_eq!(columns[1]["name"], "name");
        assert!(columns[1]["notnull"].as_bool().unwrap());
    }
}
