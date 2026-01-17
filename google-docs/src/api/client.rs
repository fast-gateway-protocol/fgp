//! Google Docs HTTP API client with connection pooling.
//!
//! Uses OAuth2 tokens from ~/.fgp/auth/google/token.json
//!
//! # CHANGELOG (recent first, max 5 entries)
//! 01/15/2026 - Initial implementation (Claude)

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::path::PathBuf;

use crate::models::{BatchUpdateResponse, Document, PlainTextExtract};

const DOCS_API_BASE: &str = "https://docs.googleapis.com/v1/documents";

/// Google Docs API client with persistent connection pooling.
pub struct DocsClient {
    client: Client,
    token_path: PathBuf,
}

impl DocsClient {
    /// Create a new Docs client.
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let token_path = home.join(".fgp/auth/google/token.json");

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("fgp-google-docs/1.0.0")
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

    /// Get document.
    pub async fn get_document(&self, document_id: &str) -> Result<Document> {
        let token = self.get_token()?;

        let url = format!("{}/{}", DOCS_API_BASE, document_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to get document")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Docs API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse document")
    }

    /// Get document as plain text.
    pub async fn get_text(&self, document_id: &str) -> Result<PlainTextExtract> {
        let doc = self.get_document(document_id).await?;

        let mut text = String::new();

        if let Some(body) = &doc.body {
            if let Some(content) = &body.content {
                for element in content {
                    if let Some(paragraph) = &element.paragraph {
                        if let Some(elements) = &paragraph.elements {
                            for pe in elements {
                                if let Some(text_run) = &pe.text_run {
                                    if let Some(content) = &text_run.content {
                                        text.push_str(content);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(PlainTextExtract {
            document_id: doc.document_id,
            title: doc.title,
            char_count: text.len(),
            text,
        })
    }

    /// Create a new document.
    pub async fn create_document(&self, title: &str) -> Result<Document> {
        let token = self.get_token()?;

        let body = serde_json::json!({
            "title": title
        });

        let response = self
            .client
            .post(DOCS_API_BASE)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to create document")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Docs API error: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse document")
    }

    /// Batch update document.
    pub async fn batch_update(
        &self,
        document_id: &str,
        requests: Vec<Value>,
    ) -> Result<BatchUpdateResponse> {
        let token = self.get_token()?;

        let url = format!("{}/{}:batchUpdate", DOCS_API_BASE, document_id);

        let body = serde_json::json!({
            "requests": requests
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to batch update")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Docs API error: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse batch update response")
    }

    /// Insert text at a location.
    pub async fn insert_text(
        &self,
        document_id: &str,
        text: &str,
        index: i64,
    ) -> Result<BatchUpdateResponse> {
        let request = serde_json::json!({
            "insertText": {
                "location": {
                    "index": index
                },
                "text": text
            }
        });

        self.batch_update(document_id, vec![request]).await
    }

    /// Delete content in a range.
    pub async fn delete_content(
        &self,
        document_id: &str,
        start_index: i64,
        end_index: i64,
    ) -> Result<BatchUpdateResponse> {
        let request = serde_json::json!({
            "deleteContentRange": {
                "range": {
                    "startIndex": start_index,
                    "endIndex": end_index
                }
            }
        });

        self.batch_update(document_id, vec![request]).await
    }

    /// Find and replace text.
    pub async fn replace_all_text(
        &self,
        document_id: &str,
        find: &str,
        replace: &str,
        match_case: bool,
    ) -> Result<BatchUpdateResponse> {
        let request = serde_json::json!({
            "replaceAllText": {
                "containsText": {
                    "text": find,
                    "matchCase": match_case
                },
                "replaceText": replace
            }
        });

        self.batch_update(document_id, vec![request]).await
    }

    /// Append text to end of document.
    pub async fn append_text(
        &self,
        document_id: &str,
        text: &str,
    ) -> Result<BatchUpdateResponse> {
        // Get document to find end index
        let doc = self.get_document(document_id).await?;

        // Find the end index (last content element)
        let mut end_index = 1; // Default to start after title
        if let Some(body) = &doc.body {
            if let Some(content) = &body.content {
                if let Some(last) = content.last() {
                    if let Some(idx) = last.end_index {
                        end_index = idx - 1; // Insert before the trailing newline
                    }
                }
            }
        }

        self.insert_text(document_id, text, end_index).await
    }
}
