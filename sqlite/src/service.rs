//! FGP service implementation for SQLite.

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::db::ConnectionManager;

/// FGP service for SQLite database operations.
pub struct SqliteService {
    manager: ConnectionManager,
}

impl SqliteService {
    /// Create a new SqliteService.
    pub fn new() -> Self {
        Self {
            manager: ConnectionManager::new(),
        }
    }

    /// Helper to get a string parameter.
    fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Helper to get a required string parameter.
    fn require_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Result<&'a str> {
        Self::get_str(params, key)
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: {}", key))
    }

    /// Helper to get params array.
    fn get_params(params: &HashMap<String, Value>) -> Vec<Value> {
        params
            .get("params")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    }

    // ========================================================================
    // Method implementations
    // ========================================================================

    fn handle_open(&self, params: HashMap<String, Value>) -> Result<Value> {
        let path = Self::require_str(&params, "path")?;
        self.manager.open(path)
    }

    fn handle_close(&self, params: HashMap<String, Value>) -> Result<Value> {
        let path = Self::require_str(&params, "path")?;
        self.manager.close(path)
    }

    fn handle_query(&self, params: HashMap<String, Value>) -> Result<Value> {
        let path = Self::require_str(&params, "path")?;
        let sql = Self::require_str(&params, "sql")?;
        let query_params = Self::get_params(&params);
        self.manager.query(path, sql, &query_params)
    }

    fn handle_execute(&self, params: HashMap<String, Value>) -> Result<Value> {
        let path = Self::require_str(&params, "path")?;
        let sql = Self::require_str(&params, "sql")?;
        let query_params = Self::get_params(&params);
        self.manager.execute(path, sql, &query_params)
    }

    fn handle_tables(&self, params: HashMap<String, Value>) -> Result<Value> {
        let path = Self::require_str(&params, "path")?;
        self.manager.tables(path)
    }

    fn handle_schema(&self, params: HashMap<String, Value>) -> Result<Value> {
        let path = Self::require_str(&params, "path")?;
        let table = Self::require_str(&params, "table")?;
        self.manager.schema(path, table)
    }
}

impl Default for SqliteService {
    fn default() -> Self {
        Self::new()
    }
}

impl FgpService for SqliteService {
    fn name(&self) -> &str {
        "sqlite"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "open" | "sqlite.open" => self.handle_open(params),
            "close" | "sqlite.close" => self.handle_close(params),
            "query" | "sqlite.query" => self.handle_query(params),
            "execute" | "sqlite.execute" => self.handle_execute(params),
            "tables" | "sqlite.tables" => self.handle_tables(params),
            "schema" | "sqlite.schema" => self.handle_schema(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // sqlite.open - Open/create a database
            MethodInfo::new("sqlite.open", "Open or create a SQLite database file")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "path",
                            SchemaBuilder::string()
                                .description("Path to the SQLite database file (supports ~)"),
                        )
                        .required(&["path"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "path",
                            SchemaBuilder::string().description("Resolved absolute path"),
                        )
                        .property(
                            "status",
                            SchemaBuilder::string()
                                .enum_values(&["opened", "already_open"])
                                .description("Connection status"),
                        )
                        .build(),
                )
                .example("Open a database", json!({"path": "~/data/mydb.sqlite"})),
            // sqlite.close - Close a database connection
            MethodInfo::new("sqlite.close", "Close a database connection")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "path",
                            SchemaBuilder::string()
                                .description("Path to the SQLite database file"),
                        )
                        .required(&["path"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("path", SchemaBuilder::string())
                        .property(
                            "status",
                            SchemaBuilder::string().enum_values(&["closed", "not_open"]),
                        )
                        .build(),
                )
                .example("Close a database", json!({"path": "~/data/mydb.sqlite"})),
            // sqlite.query - Execute SELECT query
            MethodInfo::new("sqlite.query", "Execute a SELECT query and return rows as JSON")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "path",
                            SchemaBuilder::string()
                                .description("Path to the SQLite database file"),
                        )
                        .property(
                            "sql",
                            SchemaBuilder::string().description("SQL SELECT statement"),
                        )
                        .property(
                            "params",
                            SchemaBuilder::array()
                                .description("Query parameters (for ? placeholders)"),
                        )
                        .required(&["path", "sql"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "columns",
                            SchemaBuilder::array()
                                .items(SchemaBuilder::string())
                                .description("Column names"),
                        )
                        .property(
                            "rows",
                            SchemaBuilder::array()
                                .items(SchemaBuilder::object())
                                .description("Result rows as objects"),
                        )
                        .property(
                            "count",
                            SchemaBuilder::integer().description("Number of rows returned"),
                        )
                        .build(),
                )
                .example(
                    "Simple query",
                    json!({"path": "~/data/test.db", "sql": "SELECT * FROM users LIMIT 10"}),
                )
                .example(
                    "Query with parameters",
                    json!({
                        "path": "~/data/test.db",
                        "sql": "SELECT * FROM users WHERE age > ? AND name LIKE ?",
                        "params": [21, "A%"]
                    }),
                )
                .errors(&["SQLITE_ERROR", "NO_SUCH_TABLE"]),
            // sqlite.execute - Execute INSERT/UPDATE/DELETE
            MethodInfo::new(
                "sqlite.execute",
                "Execute an INSERT, UPDATE, or DELETE statement",
            )
            .schema(
                SchemaBuilder::object()
                    .property(
                        "path",
                        SchemaBuilder::string()
                            .description("Path to the SQLite database file"),
                    )
                    .property(
                        "sql",
                        SchemaBuilder::string().description("SQL statement to execute"),
                    )
                    .property(
                        "params",
                        SchemaBuilder::array()
                            .description("Statement parameters (for ? placeholders)"),
                    )
                    .required(&["path", "sql"])
                    .build(),
            )
            .returns(
                SchemaBuilder::object()
                    .property(
                        "affected_rows",
                        SchemaBuilder::integer().description("Number of rows affected"),
                    )
                    .property(
                        "last_insert_id",
                        SchemaBuilder::integer()
                            .description("ROWID of last inserted row (for INSERT)"),
                    )
                    .build(),
            )
            .example(
                "Insert a row",
                json!({
                    "path": "~/data/test.db",
                    "sql": "INSERT INTO users (name, age) VALUES (?, ?)",
                    "params": ["Alice", 30]
                }),
            )
            .example(
                "Update rows",
                json!({
                    "path": "~/data/test.db",
                    "sql": "UPDATE users SET age = age + 1 WHERE name = ?",
                    "params": ["Alice"]
                }),
            )
            .example(
                "Create a table",
                json!({
                    "path": "~/data/test.db",
                    "sql": "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)"
                }),
            )
            .errors(&["SQLITE_ERROR", "SQLITE_CONSTRAINT"]),
            // sqlite.tables - List tables
            MethodInfo::new("sqlite.tables", "List all tables and views in the database")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "path",
                            SchemaBuilder::string()
                                .description("Path to the SQLite database file"),
                        )
                        .required(&["path"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "tables",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("name", SchemaBuilder::string())
                                    .property(
                                        "type",
                                        SchemaBuilder::string()
                                            .enum_values(&["table", "view"]),
                                    ),
                            ),
                        )
                        .property("count", SchemaBuilder::integer())
                        .build(),
                )
                .example("List tables", json!({"path": "~/data/test.db"})),
            // sqlite.schema - Get table schema
            MethodInfo::new("sqlite.schema", "Get the schema (columns, indexes) for a table")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "path",
                            SchemaBuilder::string()
                                .description("Path to the SQLite database file"),
                        )
                        .property(
                            "table",
                            SchemaBuilder::string().description("Table name"),
                        )
                        .required(&["path", "table"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("table", SchemaBuilder::string())
                        .property(
                            "columns",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("cid", SchemaBuilder::integer())
                                    .property("name", SchemaBuilder::string())
                                    .property(
                                        "type",
                                        SchemaBuilder::string().description("SQLite type"),
                                    )
                                    .property("notnull", SchemaBuilder::boolean())
                                    .property_raw("default", json!({}))
                                    .property("primary_key", SchemaBuilder::boolean()),
                            ),
                        )
                        .property(
                            "indexes",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("name", SchemaBuilder::string())
                                    .property("unique", SchemaBuilder::boolean())
                                    .property("origin", SchemaBuilder::string()),
                            ),
                        )
                        .property(
                            "sql",
                            SchemaBuilder::string().description("CREATE TABLE statement"),
                        )
                        .build(),
                )
                .example(
                    "Get table schema",
                    json!({"path": "~/data/test.db", "table": "users"}),
                )
                .errors(&["NO_SUCH_TABLE"]),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("SqliteService starting");
        Ok(())
    }

    fn on_stop(&self) -> Result<()> {
        tracing::info!(
            "SqliteService stopping, closing {} connections",
            self.manager.connection_count()
        );
        Ok(())
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let connection_count = self.manager.connection_count();
        let open_dbs = self.manager.list_open();

        checks.insert(
            "sqlite".into(),
            HealthStatus {
                ok: true,
                latency_ms: None,
                message: Some(format!(
                    "{} open connections: {:?}",
                    connection_count,
                    open_dbs
                        .iter()
                        .take(5)
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                )),
            },
        );

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_and_version() {
        let service = SqliteService::new();
        assert_eq!(service.name(), "sqlite");
        assert!(!service.version().is_empty());
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = SqliteService::new();
        let err = service
            .dispatch("sqlite.nope", HashMap::new())
            .expect_err("expected unknown method error");
        assert!(err.to_string().contains("Unknown method"));
    }

    #[test]
    fn test_dispatch_requires_path() {
        let service = SqliteService::new();

        // Open without path
        let err = service
            .dispatch("sqlite.open", HashMap::new())
            .expect_err("expected missing path error");
        assert!(err.to_string().contains("path"));

        // Query without path
        let mut params = HashMap::new();
        params.insert("sql".to_string(), json!("SELECT 1"));
        let err = service
            .dispatch("sqlite.query", params)
            .expect_err("expected missing path error");
        assert!(err.to_string().contains("path"));
    }

    #[test]
    fn test_method_list_has_all_methods() {
        let service = SqliteService::new();
        let methods = service.method_list();

        let method_names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();

        assert!(method_names.contains(&"sqlite.open"));
        assert!(method_names.contains(&"sqlite.close"));
        assert!(method_names.contains(&"sqlite.query"));
        assert!(method_names.contains(&"sqlite.execute"));
        assert!(method_names.contains(&"sqlite.tables"));
        assert!(method_names.contains(&"sqlite.schema"));
    }

    #[test]
    fn test_health_check() {
        let service = SqliteService::new();
        let health = service.health_check();

        assert!(health.contains_key("sqlite"));
        assert!(health["sqlite"].ok);
    }
}
