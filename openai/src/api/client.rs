//! OpenAI REST API client with connection pooling.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::models::{
    ApiError, ChatCompletionRequest, ChatCompletionResponse, CompletionRequest,
    CompletionResponse, EmbeddingRequest, EmbeddingResponse, Model, ModelsResponse,
};

const API_BASE_URL: &str = "https://api.openai.com/v1";

/// OpenAI API client with persistent connection pooling.
pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client.
    ///
    /// API key resolution:
    /// 1. Explicit api_key parameter
    /// 2. OPENAI_API_KEY environment variable
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = match api_key {
            Some(key) => key,
            None => Self::resolve_api_key()?,
        };

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(120)) // Longer timeout for completions
            .user_agent("fgp-openai/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, api_key })
    }

    /// Resolve API key from environment.
    fn resolve_api_key() -> Result<String> {
        std::env::var("OPENAI_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "No OpenAI API key found. Set OPENAI_API_KEY environment variable."
            )
        })
    }

    /// Make a POST request to the OpenAI API.
    async fn post<T, R>(&self, endpoint: &str, body: &T) -> Result<R>
    where
        T: serde::Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", API_BASE_URL, endpoint);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();

            // Try to parse as API error
            if let Ok(api_error) = serde_json::from_str::<ApiError>(&text) {
                bail!(
                    "OpenAI API error ({}): {}",
                    api_error.error.error_type,
                    api_error.error.message
                );
            }

            bail!("OpenAI API request failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse response")?;
        Ok(result)
    }

    /// Make a GET request to the OpenAI API.
    async fn get<R>(&self, endpoint: &str) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let url = format!("{}{}", API_BASE_URL, endpoint);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();

            if let Ok(api_error) = serde_json::from_str::<ApiError>(&text) {
                bail!(
                    "OpenAI API error ({}): {}",
                    api_error.error.error_type,
                    api_error.error.message
                );
            }

            bail!("OpenAI API request failed: {} - {}", status, text);
        }

        let result = response.json().await.context("Failed to parse response")?;
        Ok(result)
    }

    /// Check if the client can connect to OpenAI API.
    pub async fn ping(&self) -> Result<bool> {
        // Use models endpoint as a lightweight health check
        let result: ModelsResponse = self.get("/models").await?;
        Ok(!result.data.is_empty())
    }

    /// Create a chat completion.
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        self.post("/chat/completions", &request).await
    }

    /// Create a legacy completion.
    pub async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.post("/completions", &request).await
    }

    /// Create embeddings.
    pub async fn embedding(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        self.post("/embeddings", &request).await
    }

    /// List available models.
    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let response: ModelsResponse = self.get("/models").await?;
        Ok(response.data)
    }

    /// Get a specific model.
    pub async fn get_model(&self, model_id: &str) -> Result<Model> {
        self.get(&format!("/models/{}", model_id)).await
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
    fn test_resolve_api_key_from_env() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("OPENAI_API_KEY", "sk-test-key");

        let key = OpenAIClient::resolve_api_key().expect("should resolve key");
        assert_eq!(key, "sk-test-key");
    }

    #[test]
    fn test_resolve_api_key_missing() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::remove("OPENAI_API_KEY");

        let result = OpenAIClient::resolve_api_key();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OPENAI_API_KEY"));
    }

    #[test]
    fn test_client_creation_with_explicit_key() {
        let client = OpenAIClient::new(Some("sk-explicit-key".to_string()));
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_from_env() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = EnvGuard::set("OPENAI_API_KEY", "sk-env-key");

        let client = OpenAIClient::new(None);
        assert!(client.is_ok());
    }
}
