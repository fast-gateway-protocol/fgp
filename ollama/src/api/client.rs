//! Ollama HTTP API client with connection pooling.

use anyhow::{bail, Context, Result};
use reqwest::Client;

use crate::models::{
    ChatRequest, ChatResponse, EmbeddingRequest, EmbeddingResponse, GenerateRequest,
    GenerateResponse, ListResponse, ModelInfo, PullRequest, PullResponse, ShowRequest,
    ShowResponse,
};

const DEFAULT_HOST: &str = "http://localhost:11434";

/// Ollama API client with persistent connection pooling.
pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    /// Create a new Ollama client.
    ///
    /// Host resolution order:
    /// 1. Explicit host parameter
    /// 2. OLLAMA_HOST environment variable
    /// 3. Default: http://localhost:11434
    pub fn new(host: Option<String>) -> Result<Self> {
        let base_url = match host {
            Some(h) => h,
            None => Self::resolve_host(),
        };

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(300)) // Long timeout for generation
            .user_agent("fgp-ollama/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, base_url })
    }

    /// Resolve Ollama host from environment or default.
    fn resolve_host() -> String {
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string())
    }

    /// Check if Ollama is running and reachable.
    pub async fn ping(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// List installed models.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Failed to list models: {} - {}", status, text);
        }

        let list: ListResponse = response.json().await.context("Failed to parse response")?;
        Ok(list.models)
    }

    /// Show model details.
    pub async fn show_model(&self, name: &str) -> Result<ShowResponse> {
        let url = format!("{}/api/show", self.base_url);
        let request = ShowRequest {
            name: name.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Failed to show model '{}': {} - {}", name, status, text);
        }

        let show: ShowResponse = response.json().await.context("Failed to parse response")?;
        Ok(show)
    }

    /// Pull a model from the registry.
    pub async fn pull_model(&self, name: &str) -> Result<PullResponse> {
        let url = format!("{}/api/pull", self.base_url);
        let request = PullRequest {
            name: name.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Failed to pull model '{}': {} - {}", name, status, text);
        }

        let pull: PullResponse = response.json().await.context("Failed to parse response")?;
        Ok(pull)
    }

    /// Generate text from a prompt.
    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!(
                "Failed to generate with model '{}': {} - {}",
                request.model,
                status,
                text
            );
        }

        let gen: GenerateResponse = response.json().await.context("Failed to parse response")?;
        Ok(gen)
    }

    /// Chat completion with message history.
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!(
                "Failed to chat with model '{}': {} - {}",
                request.model,
                status,
                text
            );
        }

        let chat: ChatResponse = response.json().await.context("Failed to parse response")?;
        Ok(chat)
    }

    /// Generate embeddings for text.
    pub async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let url = format!("{}/api/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!(
                "Failed to generate embeddings with model '{}': {} - {}",
                request.model,
                status,
                text
            );
        }

        let embed: EmbeddingResponse = response.json().await.context("Failed to parse response")?;
        Ok(embed)
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests modify global env vars so they can't run in parallel.
    // Use serial_test crate or run with --test-threads=1 for reliability.
    // For now, we test the explicit host path which is deterministic.

    #[test]
    fn test_client_with_explicit_host_overrides_env() {
        // Even if OLLAMA_HOST is set, explicit host should win
        let client = OllamaClient::new(Some("http://explicit:9999".to_string())).unwrap();
        assert_eq!(client.base_url(), "http://explicit:9999");
    }

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new(None).unwrap();
        assert!(client.base_url().starts_with("http://"));
    }

    #[test]
    fn test_client_with_custom_host() {
        let client = OllamaClient::new(Some("http://example.com:11434".to_string())).unwrap();
        assert_eq!(client.base_url(), "http://example.com:11434");
    }
}
