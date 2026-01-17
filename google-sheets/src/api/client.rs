//! Google Sheets HTTP API client with connection pooling.
//!
//! Uses OAuth2 tokens from ~/.fgp/auth/google/token.json
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::path::PathBuf;

use crate::models::{
    AppendResponse, BatchGetResponse, ClearResponse, Spreadsheet, UpdateResponse, ValueRange,
};

const SHEETS_API_BASE: &str = "https://sheets.googleapis.com/v4/spreadsheets";

/// Google Sheets API client with persistent connection pooling.
pub struct SheetsClient {
    client: Client,
    token_path: PathBuf,
}

impl SheetsClient {
    /// Create a new Sheets client.
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let token_path = home.join(".fgp/auth/google/token.json");

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("fgp-google-sheets/1.0.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, token_path })
    }

    /// Get OAuth2 access token.
    fn get_token(&self) -> Result<String> {
        if !self.token_path.exists() {
            bail!(
                "Google OAuth token not found at {:?}. Run OAuth flow first.",
                self.token_path
            );
        }

        let content =
            std::fs::read_to_string(&self.token_path).context("Failed to read token file")?;
        let token: Value = serde_json::from_str(&content).context("Failed to parse token file")?;

        token["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No access_token in token file"))
    }

    /// Get spreadsheet metadata.
    pub async fn get_spreadsheet(&self, spreadsheet_id: &str) -> Result<Spreadsheet> {
        let token = self.get_token()?;

        let url = format!("{}/{}", SHEETS_API_BASE, spreadsheet_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to get spreadsheet")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse spreadsheet")
    }

    /// Get values from a range.
    pub async fn get_values(&self, spreadsheet_id: &str, range: &str) -> Result<ValueRange> {
        let token = self.get_token()?;

        let url = format!(
            "{}/{}/values/{}",
            SHEETS_API_BASE,
            spreadsheet_id,
            urlencoding::encode(range)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to get values")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse values")
    }

    /// Get values from multiple ranges.
    pub async fn batch_get(
        &self,
        spreadsheet_id: &str,
        ranges: &[&str],
    ) -> Result<BatchGetResponse> {
        let token = self.get_token()?;

        let ranges_param = ranges
            .iter()
            .map(|r| format!("ranges={}", urlencoding::encode(r)))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!(
            "{}/{}/values:batchGet?{}",
            SHEETS_API_BASE, spreadsheet_id, ranges_param
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to batch get values")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse batch values")
    }

    /// Update values in a range.
    pub async fn update_values(
        &self,
        spreadsheet_id: &str,
        range: &str,
        values: Vec<Vec<Value>>,
    ) -> Result<UpdateResponse> {
        let token = self.get_token()?;

        let url = format!(
            "{}/{}/values/{}?valueInputOption=USER_ENTERED",
            SHEETS_API_BASE,
            spreadsheet_id,
            urlencoding::encode(range)
        );

        let body = serde_json::json!({
            "range": range,
            "values": values
        });

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to update values")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse update response")
    }

    /// Append values to a range.
    pub async fn append_values(
        &self,
        spreadsheet_id: &str,
        range: &str,
        values: Vec<Vec<Value>>,
    ) -> Result<AppendResponse> {
        let token = self.get_token()?;

        let url = format!(
            "{}/{}/values/{}:append?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS",
            SHEETS_API_BASE,
            spreadsheet_id,
            urlencoding::encode(range)
        );

        let body = serde_json::json!({
            "range": range,
            "values": values
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to append values")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse append response")
    }

    /// Clear values in a range.
    pub async fn clear_values(&self, spreadsheet_id: &str, range: &str) -> Result<ClearResponse> {
        let token = self.get_token()?;

        let url = format!(
            "{}/{}/values/{}:clear",
            SHEETS_API_BASE,
            spreadsheet_id,
            urlencoding::encode(range)
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("Failed to clear values")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse clear response")
    }

    /// Create a new spreadsheet.
    pub async fn create_spreadsheet(&self, title: &str) -> Result<Spreadsheet> {
        let token = self.get_token()?;

        let body = serde_json::json!({
            "properties": {
                "title": title
            }
        });

        let response = self
            .client
            .post(SHEETS_API_BASE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to create spreadsheet")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse create response")
    }

    /// Add a new sheet to a spreadsheet.
    pub async fn add_sheet(&self, spreadsheet_id: &str, title: &str) -> Result<Value> {
        let token = self.get_token()?;

        let url = format!("{}/{}:batchUpdate", SHEETS_API_BASE, spreadsheet_id);

        let body = serde_json::json!({
            "requests": [{
                "addSheet": {
                    "properties": {
                        "title": title
                    }
                }
            }]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to add sheet")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse add response")
    }

    /// Delete a sheet from a spreadsheet.
    pub async fn delete_sheet(&self, spreadsheet_id: &str, sheet_id: i64) -> Result<Value> {
        let token = self.get_token()?;

        let url = format!("{}/{}:batchUpdate", SHEETS_API_BASE, spreadsheet_id);

        let body = serde_json::json!({
            "requests": [{
                "deleteSheet": {
                    "sheetId": sheet_id
                }
            }]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to delete sheet")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Sheets API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse delete response")
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                ' ' => result.push_str("%20"),
                ':' => result.push_str("%3A"),
                '!' => result.push_str("%21"),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}
