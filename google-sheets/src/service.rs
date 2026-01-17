//! FGP service implementation for Google Sheets.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::SheetsClient;

/// FGP service for Google Sheets operations.
pub struct SheetsService {
    client: Arc<SheetsClient>,
    runtime: Runtime,
}

impl SheetsService {
    /// Create a new SheetsService.
    pub fn new() -> Result<Self> {
        let client = SheetsClient::new()?;
        let runtime = Runtime::new()?;

        Ok(Self {
            client: Arc::new(client),
            runtime,
        })
    }

    /// Helper to get a string parameter.
    fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Helper to get an i64 parameter.
    fn get_i64(params: &HashMap<String, Value>, key: &str) -> Option<i64> {
        params.get(key).and_then(|v| v.as_i64())
    }

    // ========================================================================
    // Spreadsheet operations
    // ========================================================================

    fn get_spreadsheet(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();

        let result = self.runtime.block_on(async move {
            client.get_spreadsheet(&spreadsheet_id).await
        })?;

        Ok(json!(result))
    }

    fn get_values(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;
        let range = Self::get_str(&params, "range")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: range"))?;

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();
        let range = range.to_string();

        let result = self.runtime.block_on(async move {
            client.get_values(&spreadsheet_id, &range).await
        })?;

        Ok(json!(result))
    }

    fn batch_get(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;
        let ranges = params.get("ranges")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: ranges"))?;

        let ranges: Vec<String> = ranges.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();

        let result = self.runtime.block_on(async move {
            let range_refs: Vec<&str> = ranges.iter().map(|s| s.as_str()).collect();
            client.batch_get(&spreadsheet_id, &range_refs).await
        })?;

        Ok(json!(result))
    }

    fn update_values(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;
        let range = Self::get_str(&params, "range")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: range"))?;
        let values = params.get("values")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: values"))?;

        let values: Vec<Vec<Value>> = values.iter()
            .filter_map(|row| row.as_array().map(|a| a.clone()))
            .collect();

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();
        let range = range.to_string();

        let result = self.runtime.block_on(async move {
            client.update_values(&spreadsheet_id, &range, values).await
        })?;

        Ok(json!(result))
    }

    fn append_values(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;
        let range = Self::get_str(&params, "range")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: range"))?;
        let values = params.get("values")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: values"))?;

        let values: Vec<Vec<Value>> = values.iter()
            .filter_map(|row| row.as_array().map(|a| a.clone()))
            .collect();

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();
        let range = range.to_string();

        let result = self.runtime.block_on(async move {
            client.append_values(&spreadsheet_id, &range, values).await
        })?;

        Ok(json!(result))
    }

    fn clear_values(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;
        let range = Self::get_str(&params, "range")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: range"))?;

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();
        let range = range.to_string();

        let result = self.runtime.block_on(async move {
            client.clear_values(&spreadsheet_id, &range).await
        })?;

        Ok(json!(result))
    }

    fn create_spreadsheet(&self, params: HashMap<String, Value>) -> Result<Value> {
        let title = Self::get_str(&params, "title")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;

        let client = self.client.clone();
        let title = title.to_string();

        let result = self.runtime.block_on(async move {
            client.create_spreadsheet(&title).await
        })?;

        Ok(json!(result))
    }

    fn add_sheet(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;
        let title = Self::get_str(&params, "title")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();
        let title = title.to_string();

        let result = self.runtime.block_on(async move {
            client.add_sheet(&spreadsheet_id, &title).await
        })?;

        Ok(result)
    }

    fn delete_sheet(&self, params: HashMap<String, Value>) -> Result<Value> {
        let spreadsheet_id = Self::get_str(&params, "spreadsheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: spreadsheet_id"))?;
        let sheet_id = Self::get_i64(&params, "sheet_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: sheet_id"))?;

        let client = self.client.clone();
        let spreadsheet_id = spreadsheet_id.to_string();

        let result = self.runtime.block_on(async move {
            client.delete_sheet(&spreadsheet_id, sheet_id).await
        })?;

        Ok(result)
    }
}

impl FgpService for SheetsService {
    fn name(&self) -> &str {
        "sheets"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "get" | "sheets.get" => self.get_spreadsheet(params),
            "values" | "sheets.values" => self.get_values(params),
            "batch_get" | "sheets.batch_get" => self.batch_get(params),
            "update" | "sheets.update" => self.update_values(params),
            "append" | "sheets.append" => self.append_values(params),
            "clear" | "sheets.clear" => self.clear_values(params),
            "create" | "sheets.create" => self.create_spreadsheet(params),
            "add_sheet" | "sheets.add_sheet" => self.add_sheet(params),
            "delete_sheet" | "sheets.delete_sheet" => self.delete_sheet(params),

            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo::new("sheets.get", "Get spreadsheet metadata")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .required(&["spreadsheet_id"])
                        .build(),
                ),

            MethodInfo::new("sheets.values", "Get values from a range")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .property("range", SchemaBuilder::string().description("A1 notation range (e.g., Sheet1!A1:B10)"))
                        .required(&["spreadsheet_id", "range"])
                        .build(),
                )
                .example("Get values", json!({"spreadsheet_id": "abc123", "range": "Sheet1!A1:C10"})),

            MethodInfo::new("sheets.batch_get", "Get values from multiple ranges")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .property("ranges", SchemaBuilder::array().description("List of A1 notation ranges"))
                        .required(&["spreadsheet_id", "ranges"])
                        .build(),
                ),

            MethodInfo::new("sheets.update", "Update values in a range")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .property("range", SchemaBuilder::string().description("A1 notation range"))
                        .property("values", SchemaBuilder::array().description("2D array of values"))
                        .required(&["spreadsheet_id", "range", "values"])
                        .build(),
                )
                .example("Update values", json!({
                    "spreadsheet_id": "abc123",
                    "range": "Sheet1!A1:B2",
                    "values": [["Name", "Age"], ["Alice", 30]]
                })),

            MethodInfo::new("sheets.append", "Append values to a range")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .property("range", SchemaBuilder::string().description("A1 notation range"))
                        .property("values", SchemaBuilder::array().description("2D array of values"))
                        .required(&["spreadsheet_id", "range", "values"])
                        .build(),
                ),

            MethodInfo::new("sheets.clear", "Clear values in a range")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .property("range", SchemaBuilder::string().description("A1 notation range"))
                        .required(&["spreadsheet_id", "range"])
                        .build(),
                ),

            MethodInfo::new("sheets.create", "Create a new spreadsheet")
                .schema(
                    SchemaBuilder::object()
                        .property("title", SchemaBuilder::string().description("Spreadsheet title"))
                        .required(&["title"])
                        .build(),
                ),

            MethodInfo::new("sheets.add_sheet", "Add a new sheet to spreadsheet")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .property("title", SchemaBuilder::string().description("New sheet title"))
                        .required(&["spreadsheet_id", "title"])
                        .build(),
                ),

            MethodInfo::new("sheets.delete_sheet", "Delete a sheet from spreadsheet")
                .schema(
                    SchemaBuilder::object()
                        .property("spreadsheet_id", SchemaBuilder::string().description("Spreadsheet ID"))
                        .property("sheet_id", SchemaBuilder::integer().description("Sheet ID (not index)"))
                        .required(&["spreadsheet_id", "sheet_id"])
                        .build(),
                ),
        ]
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let home = dirs::home_dir().unwrap_or_default();
        let token_path = home.join(".fgp/auth/google/token.json");

        checks.insert(
            "oauth_token".into(),
            if token_path.exists() {
                HealthStatus::healthy()
            } else {
                HealthStatus::unhealthy("OAuth token not found - run auth flow")
            },
        );

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_service() -> SheetsService {
        SheetsService::new().expect("service")
    }

    #[test]
    fn test_param_helpers() {
        let mut params = HashMap::new();
        params.insert("spreadsheet_id".to_string(), Value::String("sheet".to_string()));
        params.insert("sheet_id".to_string(), Value::from(7));

        assert_eq!(SheetsService::get_str(&params, "spreadsheet_id"), Some("sheet"));
        assert_eq!(SheetsService::get_str(&params, "missing"), None);
        assert_eq!(SheetsService::get_i64(&params, "sheet_id"), Some(7));
        assert_eq!(SheetsService::get_i64(&params, "missing"), None);
    }

    #[test]
    fn test_method_list_required_fields() {
        let service = test_service();
        let methods = service.method_list();

        let update = methods
            .iter()
            .find(|m| m.name == "sheets.update")
            .expect("sheets.update");
        let update_schema = update.schema.as_ref().expect("schema");
        let required = update_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "spreadsheet_id"));
        assert!(required.iter().any(|v| v == "range"));
        assert!(required.iter().any(|v| v == "values"));

        let delete_sheet = methods
            .iter()
            .find(|m| m.name == "sheets.delete_sheet")
            .expect("sheets.delete_sheet");
        let delete_schema = delete_sheet.schema.as_ref().expect("schema");
        let required = delete_schema
            .get("required")
            .and_then(Value::as_array)
            .expect("required");
        assert!(required.iter().any(|v| v == "sheet_id"));
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let service = test_service();
        let result = service.dispatch("sheets.unknown", HashMap::new());
        assert!(result.is_err());
    }
}
