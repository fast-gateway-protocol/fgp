//! Resend REST API client with connection pooling.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::models::{
    ApiError, BatchEmailResponse, Domain, DomainsResponse, Email, SendEmailRequest,
    SendEmailResponse,
};

const API_BASE_URL: &str = "https://api.resend.com";

/// Resend API client with persistent connection pooling.
pub struct ResendClient {
    client: Client,
    api_key: String,
}

impl ResendClient {
    /// Create a new Resend client.
    ///
    /// API key resolution:
    /// 1. Explicit api_key parameter
    /// 2. RESEND_API_KEY environment variable
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = match api_key {
            Some(key) => key,
            None => Self::resolve_api_key()?,
        };

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("fgp-resend/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, api_key })
    }

    /// Resolve API key from environment.
    fn resolve_api_key() -> Result<String> {
        std::env::var("RESEND_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "No Resend API key found. Set RESEND_API_KEY environment variable.\n\
                 Get your API key from https://resend.com/api-keys"
            )
        })
    }

    /// Make a GET request to the Resend API.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE_URL, path);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send GET request")?;

        self.handle_response(response).await
    }

    /// Make a POST request to the Resend API.
    async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", API_BASE_URL, path);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to send POST request")?;

        self.handle_response(response).await
    }

    /// Handle API response, checking for errors.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        let text = response.text().await.context("Failed to read response")?;

        if !status.is_success() {
            // Try to parse as API error
            if let Ok(api_error) = serde_json::from_str::<ApiError>(&text) {
                bail!(
                    "Resend API error ({}): {}",
                    api_error.status_code.unwrap_or(status.as_u16() as i32),
                    api_error.message
                );
            }
            bail!("Resend API request failed: {} - {}", status, text);
        }

        serde_json::from_str(&text).map_err(|e| {
            anyhow::anyhow!(
                "JSON parse error: {} | Raw: {}",
                e,
                &text[..text.len().min(500)]
            )
        })
    }

    /// Check if the API is reachable by listing domains.
    pub async fn ping(&self) -> Result<bool> {
        // Simple health check by hitting the domains endpoint
        let result: Result<DomainsResponse> = self.get("/domains").await;
        Ok(result.is_ok())
    }

    /// Send a single email.
    pub async fn send_email(&self, request: SendEmailRequest) -> Result<SendEmailResponse> {
        self.post("/emails", &request).await
    }

    /// Send batch emails (up to 100).
    pub async fn send_batch(&self, emails: Vec<SendEmailRequest>) -> Result<BatchEmailResponse> {
        if emails.is_empty() {
            bail!("Batch must contain at least one email");
        }
        if emails.len() > 100 {
            bail!("Batch cannot exceed 100 emails");
        }

        self.post("/emails/batch", &emails).await
    }

    /// Get email by ID.
    pub async fn get_email(&self, email_id: &str) -> Result<Email> {
        self.get(&format!("/emails/{}", email_id)).await
    }

    /// List verified domains.
    pub async fn list_domains(&self) -> Result<Vec<Domain>> {
        let response: DomainsResponse = self.get("/domains").await?;
        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn resolve_api_key_from_env() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("RESEND_API_KEY", "re_test_key");

        let key = ResendClient::resolve_api_key().expect("key");
        assert_eq!(key, "re_test_key");
    }

    #[test]
    fn resolve_api_key_missing() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::remove("RESEND_API_KEY");

        let result = ResendClient::resolve_api_key();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("RESEND_API_KEY"));
    }

    #[test]
    fn new_client_with_explicit_key() {
        let client = ResendClient::new(Some("re_explicit_key".to_string()));
        assert!(client.is_ok());
    }
}
