//! FGP service implementation for Ollama.

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::OllamaClient;
use crate::models::{ChatMessage, ChatRequest, EmbeddingRequest, GenerateOptions, GenerateRequest};

/// FGP service for Ollama operations.
pub struct OllamaService {
    client: Arc<OllamaClient>,
    runtime: Runtime,
}

impl OllamaService {
    /// Create a new OllamaService.
    ///
    /// Host is resolved from:
    /// 1. OLLAMA_HOST environment variable
    /// 2. Default: http://localhost:11434
    pub fn new(host: Option<String>) -> Result<Self> {
        let client = OllamaClient::new(host)?;
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

    /// Helper to get an f64 parameter.
    fn get_f64(params: &HashMap<String, Value>, key: &str) -> Option<f64> {
        params.get(key).and_then(|v| v.as_f64())
    }

    /// Helper to get an i32 parameter.
    fn get_i32(params: &HashMap<String, Value>, key: &str) -> Option<i32> {
        params.get(key).and_then(|v| v.as_i64()).map(|v| v as i32)
    }

    /// Parse options from params.
    fn parse_options(params: &HashMap<String, Value>) -> Option<GenerateOptions> {
        let options = params.get("options")?;
        let obj = options.as_object()?;

        Some(GenerateOptions {
            temperature: obj.get("temperature").and_then(|v| v.as_f64()),
            top_p: obj.get("top_p").and_then(|v| v.as_f64()),
            top_k: obj.get("top_k").and_then(|v| v.as_i64()).map(|v| v as i32),
            num_predict: obj
                .get("num_predict")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32),
            num_ctx: obj.get("num_ctx").and_then(|v| v.as_i64()).map(|v| v as i32),
            seed: obj.get("seed").and_then(|v| v.as_i64()),
            stop: obj.get("stop").and_then(|v| {
                v.as_array()
                    .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            }),
            repeat_penalty: obj.get("repeat_penalty").and_then(|v| v.as_f64()),
        })
    }

    // ========================================================================
    // Method implementations
    // ========================================================================

    fn health(&self) -> Result<Value> {
        let client = self.client.clone();
        let ok = self.runtime.block_on(async move { client.ping().await })?;

        Ok(json!({
            "status": if ok { "healthy" } else { "unhealthy" },
            "ollama_running": ok,
            "host": self.client.base_url(),
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    fn list_models(&self) -> Result<Value> {
        let client = self.client.clone();
        let models = self
            .runtime
            .block_on(async move { client.list_models().await })?;

        Ok(json!({
            "models": models,
            "count": models.len(),
        }))
    }

    fn show_model(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .or_else(|| Self::get_str(&params, "model"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

        let client = self.client.clone();
        let name = name.to_string();

        let info = self
            .runtime
            .block_on(async move { client.show_model(&name).await })?;

        Ok(json!(info))
    }

    fn pull_model(&self, params: HashMap<String, Value>) -> Result<Value> {
        let name = Self::get_str(&params, "name")
            .or_else(|| Self::get_str(&params, "model"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

        let client = self.client.clone();
        let name = name.to_string();

        let result = self
            .runtime
            .block_on(async move { client.pull_model(&name).await })?;

        Ok(json!({
            "success": true,
            "status": result.status,
            "digest": result.digest,
        }))
    }

    fn generate(&self, params: HashMap<String, Value>) -> Result<Value> {
        let model = Self::get_str(&params, "model")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: model"))?;
        let prompt = Self::get_str(&params, "prompt")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: prompt"))?;

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: Self::get_str(&params, "system").map(String::from),
            template: Self::get_str(&params, "template").map(String::from),
            context: params.get("context").and_then(|v| {
                v.as_array()
                    .map(|arr| arr.iter().filter_map(|n| n.as_i64()).collect())
            }),
            options: Self::parse_options(&params),
            stream: false,
        };

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.generate(request).await })?;

        Ok(json!({
            "model": response.model,
            "response": response.response,
            "done": response.done,
            "context": response.context,
            "total_duration_ns": response.total_duration,
            "eval_count": response.eval_count,
        }))
    }

    fn chat(&self, params: HashMap<String, Value>) -> Result<Value> {
        let model = Self::get_str(&params, "model")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: model"))?;

        let messages_value = params
            .get("messages")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: messages"))?;

        let messages_array = messages_value
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("messages must be an array"))?;

        let messages: Vec<ChatMessage> = messages_array
            .iter()
            .map(|m| {
                let role = m
                    .get("role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("user")
                    .to_string();
                let content = m
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let images = m.get("images").and_then(|i| {
                    i.as_array()
                        .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                });

                ChatMessage {
                    role,
                    content,
                    images,
                }
            })
            .collect();

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            options: Self::parse_options(&params),
            stream: false,
        };

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.chat(request).await })?;

        Ok(json!({
            "model": response.model,
            "message": {
                "role": response.message.role,
                "content": response.message.content,
            },
            "done": response.done,
            "total_duration_ns": response.total_duration,
            "eval_count": response.eval_count,
        }))
    }

    fn embed(&self, params: HashMap<String, Value>) -> Result<Value> {
        let model = Self::get_str(&params, "model")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: model"))?;
        let prompt = Self::get_str(&params, "prompt")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: prompt"))?;

        let request = EmbeddingRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            options: Self::parse_options(&params),
        };

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.embed(request).await })?;

        Ok(json!({
            "model": model,
            "embedding": response.embedding,
            "dimensions": response.embedding.len(),
        }))
    }
}

impl FgpService for OllamaService {
    fn name(&self) -> &str {
        "ollama"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" | "ollama.health" => self.health(),
            "list" | "ollama.list" => self.list_models(),
            "show" | "ollama.show" => self.show_model(params),
            "pull" | "ollama.pull" => self.pull_model(params),
            "generate" | "ollama.generate" => self.generate(params),
            "chat" | "ollama.chat" => self.chat(params),
            "embed" | "ollama.embed" => self.embed(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // ollama.health - Check if Ollama is running
            MethodInfo::new("ollama.health", "Check if Ollama is running and reachable")
                .schema(SchemaBuilder::object().build())
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "status",
                            SchemaBuilder::string()
                                .enum_values(&["healthy", "unhealthy"])
                                .description("Health status"),
                        )
                        .property(
                            "ollama_running",
                            SchemaBuilder::boolean().description("Whether Ollama server is running"),
                        )
                        .property(
                            "host",
                            SchemaBuilder::string().description("Ollama server host URL"),
                        )
                        .build(),
                )
                .example("Check health", json!({})),

            // ollama.list - List installed models
            MethodInfo::new("ollama.list", "List all installed Ollama models")
                .schema(SchemaBuilder::object().build())
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "models",
                            SchemaBuilder::array()
                                .items(
                                    SchemaBuilder::object()
                                        .property("name", SchemaBuilder::string())
                                        .property("modified_at", SchemaBuilder::string().format("date-time"))
                                        .property("size", SchemaBuilder::integer().description("Size in bytes"))
                                        .property("digest", SchemaBuilder::string()),
                                )
                                .description("List of installed models"),
                        )
                        .property("count", SchemaBuilder::integer())
                        .build(),
                )
                .example("List models", json!({})),

            // ollama.show - Show model details
            MethodInfo::new("ollama.show", "Show details about a specific model")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "name",
                            SchemaBuilder::string().description("Model name (e.g., 'llama2', 'mistral')"),
                        )
                        .required(&["name"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("modelfile", SchemaBuilder::string())
                        .property("parameters", SchemaBuilder::string())
                        .property("template", SchemaBuilder::string())
                        .property(
                            "details",
                            SchemaBuilder::object()
                                .property("format", SchemaBuilder::string())
                                .property("family", SchemaBuilder::string())
                                .property("parameter_size", SchemaBuilder::string())
                                .property("quantization_level", SchemaBuilder::string()),
                        )
                        .build(),
                )
                .example("Show llama2 details", json!({"name": "llama2"}))
                .errors(&["MODEL_NOT_FOUND"]),

            // ollama.pull - Pull a model from the registry
            MethodInfo::new("ollama.pull", "Pull/download a model from the Ollama registry")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "name",
                            SchemaBuilder::string()
                                .description("Model name to pull (e.g., 'llama2', 'mistral', 'codellama')"),
                        )
                        .required(&["name"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("success", SchemaBuilder::boolean())
                        .property("status", SchemaBuilder::string())
                        .property("digest", SchemaBuilder::string())
                        .build(),
                )
                .example("Pull mistral", json!({"name": "mistral"}))
                .errors(&["DOWNLOAD_FAILED", "INVALID_MODEL"]),

            // ollama.generate - Text generation
            MethodInfo::new("ollama.generate", "Generate text from a prompt")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "model",
                            SchemaBuilder::string()
                                .description("Model name (e.g., 'llama2', 'mistral', 'codellama')"),
                        )
                        .property(
                            "prompt",
                            SchemaBuilder::string().description("The prompt to generate from"),
                        )
                        .property(
                            "system",
                            SchemaBuilder::string().description("System prompt (optional)"),
                        )
                        .property(
                            "options",
                            SchemaBuilder::object()
                                .property(
                                    "temperature",
                                    SchemaBuilder::number()
                                        .description("Sampling temperature (0.0-2.0)"),
                                )
                                .property(
                                    "top_p",
                                    SchemaBuilder::number()
                                        .description("Top-p sampling (0.0-1.0)"),
                                )
                                .property(
                                    "top_k",
                                    SchemaBuilder::integer().description("Top-k sampling"),
                                )
                                .property(
                                    "num_predict",
                                    SchemaBuilder::integer().description("Max tokens to generate"),
                                )
                                .property(
                                    "seed",
                                    SchemaBuilder::integer().description("Random seed for reproducibility"),
                                )
                                .property(
                                    "stop",
                                    SchemaBuilder::array()
                                        .items(SchemaBuilder::string())
                                        .description("Stop sequences"),
                                )
                                .description("Generation options"),
                        )
                        .required(&["model", "prompt"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("model", SchemaBuilder::string())
                        .property("response", SchemaBuilder::string().description("Generated text"))
                        .property("done", SchemaBuilder::boolean())
                        .property(
                            "context",
                            SchemaBuilder::array()
                                .items(SchemaBuilder::integer())
                                .description("Context for follow-up generations"),
                        )
                        .property("total_duration_ns", SchemaBuilder::integer())
                        .property("eval_count", SchemaBuilder::integer())
                        .build(),
                )
                .example(
                    "Generate with llama2",
                    json!({"model": "llama2", "prompt": "Explain quantum computing in simple terms"}),
                )
                .example(
                    "Generate with options",
                    json!({
                        "model": "llama2",
                        "prompt": "Write a haiku about coding",
                        "options": {"temperature": 0.8, "num_predict": 100}
                    }),
                )
                .errors(&["MODEL_NOT_FOUND", "GENERATION_FAILED"]),

            // ollama.chat - Chat completion
            MethodInfo::new("ollama.chat", "Chat completion with message history")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "model",
                            SchemaBuilder::string()
                                .description("Model name (e.g., 'llama2', 'mistral')"),
                        )
                        .property(
                            "messages",
                            SchemaBuilder::array()
                                .items(
                                    SchemaBuilder::object()
                                        .property(
                                            "role",
                                            SchemaBuilder::string()
                                                .enum_values(&["system", "user", "assistant"])
                                                .description("Message role"),
                                        )
                                        .property(
                                            "content",
                                            SchemaBuilder::string().description("Message content"),
                                        )
                                        .property(
                                            "images",
                                            SchemaBuilder::array()
                                                .items(SchemaBuilder::string())
                                                .description("Base64-encoded images (for vision models)"),
                                        )
                                        .required(&["role", "content"]),
                                )
                                .description("Array of chat messages"),
                        )
                        .property(
                            "options",
                            SchemaBuilder::object()
                                .property("temperature", SchemaBuilder::number())
                                .property("top_p", SchemaBuilder::number())
                                .property("num_predict", SchemaBuilder::integer())
                                .description("Generation options"),
                        )
                        .required(&["model", "messages"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("model", SchemaBuilder::string())
                        .property(
                            "message",
                            SchemaBuilder::object()
                                .property("role", SchemaBuilder::string())
                                .property("content", SchemaBuilder::string()),
                        )
                        .property("done", SchemaBuilder::boolean())
                        .property("total_duration_ns", SchemaBuilder::integer())
                        .property("eval_count", SchemaBuilder::integer())
                        .build(),
                )
                .example(
                    "Simple chat",
                    json!({
                        "model": "llama2",
                        "messages": [
                            {"role": "user", "content": "Hello!"}
                        ]
                    }),
                )
                .example(
                    "Multi-turn chat",
                    json!({
                        "model": "llama2",
                        "messages": [
                            {"role": "system", "content": "You are a helpful assistant."},
                            {"role": "user", "content": "What is Rust?"},
                            {"role": "assistant", "content": "Rust is a systems programming language..."},
                            {"role": "user", "content": "Why is it fast?"}
                        ]
                    }),
                )
                .errors(&["MODEL_NOT_FOUND", "CHAT_FAILED"]),

            // ollama.embed - Generate embeddings
            MethodInfo::new("ollama.embed", "Generate embeddings for text")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "model",
                            SchemaBuilder::string()
                                .description("Model name (e.g., 'nomic-embed-text', 'all-minilm')"),
                        )
                        .property(
                            "prompt",
                            SchemaBuilder::string().description("Text to generate embeddings for"),
                        )
                        .required(&["model", "prompt"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("model", SchemaBuilder::string())
                        .property(
                            "embedding",
                            SchemaBuilder::array()
                                .items(SchemaBuilder::number())
                                .description("Embedding vector"),
                        )
                        .property(
                            "dimensions",
                            SchemaBuilder::integer().description("Number of dimensions"),
                        )
                        .build(),
                )
                .example(
                    "Generate embeddings",
                    json!({
                        "model": "nomic-embed-text",
                        "prompt": "The quick brown fox jumps over the lazy dog"
                    }),
                )
                .errors(&["MODEL_NOT_FOUND", "EMBEDDING_FAILED"]),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("OllamaService starting, checking Ollama connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Ollama connection verified at {}", client.base_url());
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!(
                        "Ollama not responding at {}. Some methods may fail.",
                        client.base_url()
                    );
                    Ok(())
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to Ollama: {}. Daemon will start but methods may fail.", e);
                    Ok(())
                }
            }
        })
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let client = self.client.clone();
        let start = std::time::Instant::now();
        let result = self.runtime.block_on(async move { client.ping().await });

        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(true) => {
                checks.insert("ollama".into(), HealthStatus::healthy_with_latency(latency));
            }
            Ok(false) => {
                checks.insert(
                    "ollama".into(),
                    HealthStatus::unhealthy("Ollama not responding"),
                );
            }
            Err(e) => {
                checks.insert("ollama".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_str() {
        let mut params = HashMap::new();
        params.insert("model".to_string(), json!("llama2"));
        assert_eq!(OllamaService::get_str(&params, "model"), Some("llama2"));
        assert_eq!(OllamaService::get_str(&params, "missing"), None);
    }

    #[test]
    fn test_get_f64() {
        let mut params = HashMap::new();
        params.insert("temp".to_string(), json!(0.8));
        assert_eq!(OllamaService::get_f64(&params, "temp"), Some(0.8));
        assert_eq!(OllamaService::get_f64(&params, "missing"), None);
    }

    #[test]
    fn test_get_i32() {
        let mut params = HashMap::new();
        params.insert("count".to_string(), json!(100));
        assert_eq!(OllamaService::get_i32(&params, "count"), Some(100));
        assert_eq!(OllamaService::get_i32(&params, "missing"), None);
    }

    #[test]
    fn test_parse_options() {
        let mut params = HashMap::new();
        params.insert(
            "options".to_string(),
            json!({
                "temperature": 0.8,
                "top_p": 0.9,
                "num_predict": 100
            }),
        );

        let opts = OllamaService::parse_options(&params).unwrap();
        assert_eq!(opts.temperature, Some(0.8));
        assert_eq!(opts.top_p, Some(0.9));
        assert_eq!(opts.num_predict, Some(100));
    }

    #[test]
    fn test_dispatch_requires_model_for_generate() {
        let service = OllamaService::new(None).unwrap();
        let mut params = HashMap::new();
        params.insert("prompt".to_string(), json!("test"));

        let err = service
            .dispatch("ollama.generate", params)
            .expect_err("expected missing model error");
        assert!(err.to_string().contains("model"));
    }

    #[test]
    fn test_dispatch_requires_prompt_for_generate() {
        let service = OllamaService::new(None).unwrap();
        let mut params = HashMap::new();
        params.insert("model".to_string(), json!("llama2"));

        let err = service
            .dispatch("ollama.generate", params)
            .expect_err("expected missing prompt error");
        assert!(err.to_string().contains("prompt"));
    }

    #[test]
    fn test_dispatch_rejects_unknown_method() {
        let service = OllamaService::new(None).unwrap();
        let err = service
            .dispatch("ollama.unknown", HashMap::new())
            .expect_err("expected unknown method error");
        assert!(err.to_string().contains("Unknown method"));
    }

    #[test]
    fn test_method_list_has_expected_methods() {
        let service = OllamaService::new(None).unwrap();
        let methods = service.method_list();

        let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"ollama.health"));
        assert!(names.contains(&"ollama.list"));
        assert!(names.contains(&"ollama.show"));
        assert!(names.contains(&"ollama.pull"));
        assert!(names.contains(&"ollama.generate"));
        assert!(names.contains(&"ollama.chat"));
        assert!(names.contains(&"ollama.embed"));
    }
}
