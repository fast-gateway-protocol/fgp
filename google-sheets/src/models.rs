//! Data models for Google Sheets API.
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use serde::{Deserialize, Serialize};

/// A spreadsheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spreadsheet {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(default)]
    pub properties: Option<SpreadsheetProperties>,
    #[serde(default)]
    pub sheets: Option<Vec<Sheet>>,
    #[serde(default, rename = "spreadsheetUrl")]
    pub spreadsheet_url: Option<String>,
}

/// Spreadsheet properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetProperties {
    pub title: String,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default, rename = "timeZone")]
    pub time_zone: Option<String>,
}

/// A single sheet in a spreadsheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub properties: SheetProperties,
}

/// Sheet properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetProperties {
    #[serde(rename = "sheetId")]
    pub sheet_id: i64,
    pub title: String,
    #[serde(default)]
    pub index: Option<i64>,
    #[serde(default, rename = "sheetType")]
    pub sheet_type: Option<String>,
    #[serde(default, rename = "gridProperties")]
    pub grid_properties: Option<GridProperties>,
}

/// Grid properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridProperties {
    #[serde(default, rename = "rowCount")]
    pub row_count: Option<i64>,
    #[serde(default, rename = "columnCount")]
    pub column_count: Option<i64>,
}

/// Value range for reading/writing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    pub range: String,
    #[serde(default, rename = "majorDimension")]
    pub major_dimension: Option<String>,
    #[serde(default)]
    pub values: Option<Vec<Vec<serde_json::Value>>>,
}

/// Response from batch get.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(default, rename = "valueRanges")]
    pub value_ranges: Option<Vec<ValueRange>>,
}

/// Response from update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(default, rename = "updatedRange")]
    pub updated_range: Option<String>,
    #[serde(default, rename = "updatedRows")]
    pub updated_rows: Option<i64>,
    #[serde(default, rename = "updatedColumns")]
    pub updated_columns: Option<i64>,
    #[serde(default, rename = "updatedCells")]
    pub updated_cells: Option<i64>,
}

/// Response from append.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(default, rename = "tableRange")]
    pub table_range: Option<String>,
    #[serde(default)]
    pub updates: Option<UpdateResponse>,
}

/// Response from clear.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearResponse {
    #[serde(rename = "spreadsheetId")]
    pub spreadsheet_id: String,
    #[serde(default, rename = "clearedRange")]
    pub cleared_range: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn spreadsheet_defaults_optional_fields() {
        let value = json!({
            "spreadsheetId": "sheet_1"
        });
        let sheet: Spreadsheet = serde_json::from_value(value).expect("deserialize");

        assert_eq!(sheet.spreadsheet_id, "sheet_1");
        assert!(sheet.properties.is_none());
        assert!(sheet.sheets.is_none());
        assert!(sheet.spreadsheet_url.is_none());
    }

    #[test]
    fn sheet_properties_map_grid() {
        let value = json!({
            "sheetId": 1,
            "title": "Sheet1",
            "gridProperties": {
                "rowCount": 10,
                "columnCount": 5
            }
        });
        let props: SheetProperties = serde_json::from_value(value).expect("deserialize");

        let grid = props.grid_properties.expect("grid");
        assert_eq!(grid.row_count, Some(10));
        assert_eq!(grid.column_count, Some(5));
    }

    #[test]
    fn value_range_defaults() {
        let value = json!({
            "range": "A1:B2"
        });
        let range: ValueRange = serde_json::from_value(value).expect("deserialize");

        assert_eq!(range.range, "A1:B2");
        assert!(range.values.is_none());
        assert!(range.major_dimension.is_none());
    }

    #[test]
    fn update_response_maps_fields() {
        let value = json!({
            "spreadsheetId": "sheet_1",
            "updatedRange": "A1:A1",
            "updatedRows": 1,
            "updatedColumns": 1,
            "updatedCells": 1
        });
        let response: UpdateResponse = serde_json::from_value(value).expect("deserialize");

        assert_eq!(response.spreadsheet_id, "sheet_1");
        assert_eq!(response.updated_range.as_deref(), Some("A1:A1"));
        assert_eq!(response.updated_rows, Some(1));
    }
}
