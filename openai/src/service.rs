//! FGP service implementation for OpenAI.

use anyhow::Result;
use fgp_daemon::schema::SchemaBuilder;
use fgp_daemon::service::{HealthStatus, MethodInfo};
use fgp_daemon::FgpService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::OpenAIClient;
use crate::models::{
    ChatCompletionRequest, ChatMessage, CompletionRequest, EmbeddingInput, EmbeddingRequest, Role,
};

/// FGP service for OpenAI operations.
pub struct OpenAIService {
    client: Arc<OpenAIClient>,
    runtime: Runtime,
}

impl OpenAIService {
    /// Create a new OpenAIService.
    ///
    /// API key is resolved from OPENAI_API_KEY environment variable.
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let client = OpenAIClient::new(api_key)?;
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

    /// Helper to get a bool parameter.
    fn get_bool(params: &HashMap<String, Value>, key: &str) -> Option<bool> {
        params.get(key).and_then(|v| v.as_bool())
    }

    // ========================================================================
    // Method implementations
    // ========================================================================

    fn health(&self) -> Result<Value> {
        let client = self.client.clone();
        let ok = self.runtime.block_on(async move { client.ping().await })?;

        Ok(json!({
            "status": if ok { "healthy" } else { "unhealthy" },
            "api_connected": ok,
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    fn chat(&self, params: HashMap<String, Value>) -> Result<Value> {
        let model = Self::get_str(&params, "model")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: model"))?
            .to_string();

        let messages_value = params
            .get("messages")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: messages"))?;

        let messages_array = messages_value
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("messages must be an array"))?;

        let messages: Vec<ChatMessage> = messages_array
            .iter()
            .map(|m| {
                let role_str = m
                    .get("role")
                    .and_then(|r| r.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Each message must have a role"))?;

                let role = match role_str {
                    "system" => Role::System,
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    "tool" => Role::Tool,
                    "function" => Role::Function,
                    _ => anyhow::bail!("Invalid role: {}", role_str),
                };

                let content = m
                    .get("content")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Each message must have content"))?
                    .to_string();

                let name = m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());

                Ok(ChatMessage {
                    role,
                    content,
                    name,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut request = ChatCompletionRequest::new(model, messages);
        request.temperature = Self::get_f64(&params, "temperature");
        request.max_tokens = Self::get_i32(&params, "max_tokens");
        request.top_p = Self::get_f64(&params, "top_p");
        request.frequency_penalty = Self::get_f64(&params, "frequency_penalty");
        request.presence_penalty = Self::get_f64(&params, "presence_penalty");
        request.stream = Self::get_bool(&params, "stream");

        if let Some(stop) = params.get("stop") {
            if let Some(stop_str) = stop.as_str() {
                request.stop = Some(vec![stop_str.to_string()]);
            } else if let Some(stop_arr) = stop.as_array() {
                request.stop = Some(
                    stop_arr
                        .iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect(),
                );
            }
        }

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.chat_completion(request).await })?;

        Ok(serde_json::to_value(response)?)
    }

    fn complete(&self, params: HashMap<String, Value>) -> Result<Value> {
        let model = Self::get_str(&params, "model")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: model"))?
            .to_string();

        let prompt = Self::get_str(&params, "prompt")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: prompt"))?
            .to_string();

        let mut request = CompletionRequest::new(model, prompt);
        request.temperature = Self::get_f64(&params, "temperature");
        request.max_tokens = Self::get_i32(&params, "max_tokens");
        request.top_p = Self::get_f64(&params, "top_p");
        request.frequency_penalty = Self::get_f64(&params, "frequency_penalty");
        request.presence_penalty = Self::get_f64(&params, "presence_penalty");
        request.echo = Self::get_bool(&params, "echo");

        if let Some(stop) = params.get("stop") {
            if let Some(stop_str) = stop.as_str() {
                request.stop = Some(vec![stop_str.to_string()]);
            } else if let Some(stop_arr) = stop.as_array() {
                request.stop = Some(
                    stop_arr
                        .iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect(),
                );
            }
        }

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.completion(request).await })?;

        Ok(serde_json::to_value(response)?)
    }

    fn embed(&self, params: HashMap<String, Value>) -> Result<Value> {
        let model = Self::get_str(&params, "model")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: model"))?
            .to_string();

        let input_value = params
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;

        let input = if let Some(s) = input_value.as_str() {
            EmbeddingInput::Single(s.to_string())
        } else if let Some(arr) = input_value.as_array() {
            let strings: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if strings.is_empty() {
                anyhow::bail!("input array must contain strings");
            }
            EmbeddingInput::Multiple(strings)
        } else {
            anyhow::bail!("input must be a string or array of strings");
        };

        let mut request = EmbeddingRequest::new(model, input);
        request.encoding_format = Self::get_str(&params, "encoding_format").map(|s| s.to_string());
        request.dimensions = Self::get_i32(&params, "dimensions");

        let client = self.client.clone();
        let response = self
            .runtime
            .block_on(async move { client.embedding(request).await })?;

        Ok(serde_json::to_value(response)?)
    }

    fn models(&self, params: HashMap<String, Value>) -> Result<Value> {
        let client = self.client.clone();

        // Check if a specific model was requested
        if let Some(model_id) = Self::get_str(&params, "model") {
            let model_id = model_id.to_string();
            let model = self
                .runtime
                .block_on(async move { client.get_model(&model_id).await })?;
            return Ok(serde_json::to_value(model)?);
        }

        // List all models
        let models = self
            .runtime
            .block_on(async move { client.list_models().await })?;

        // Filter and sort by ID
        let mut models = models;
        models.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(json!({
            "models": models,
            "count": models.len(),
        }))
    }
}

impl FgpService for OpenAIService {
    fn name(&self) -> &str {
        "openai"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.health(),
            "chat" | "openai.chat" => self.chat(params),
            "complete" | "openai.complete" => self.complete(params),
            "embed" | "openai.embed" => self.embed(params),
            "models" | "openai.models" => self.models(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            // openai.chat - Chat completions
            MethodInfo::new("openai.chat", "Create a chat completion with the specified model")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "model",
                            SchemaBuilder::string()
                                .description("Model ID (e.g., gpt-4, gpt-3.5-turbo)"),
                        )
                        .property(
                            "messages",
                            SchemaBuilder::array()
                                .items(
                                    SchemaBuilder::object()
                                        .property(
                                            "role",
                                            SchemaBuilder::string()
                                                .enum_values(&["system", "user", "assistant", "tool", "function"])
                                                .description("Role of the message author"),
                                        )
                                        .property(
                                            "content",
                                            SchemaBuilder::string().description("Message content"),
                                        )
                                        .property(
                                            "name",
                                            SchemaBuilder::string().description("Optional name of the author"),
                                        )
                                        .required(&["role", "content"]),
                                )
                                .description("Array of messages in the conversation"),
                        )
                        .property(
                            "temperature",
                            SchemaBuilder::number()
                                .description("Sampling temperature (0.0 to 2.0)"),
                        )
                        .property(
                            "max_tokens",
                            SchemaBuilder::integer()
                                .minimum(1)
                                .description("Maximum tokens to generate"),
                        )
                        .property(
                            "top_p",
                            SchemaBuilder::number()
                                .description("Nucleus sampling parameter (0.0 to 1.0)"),
                        )
                        .property(
                            "frequency_penalty",
                            SchemaBuilder::number()
                                .description("Frequency penalty (-2.0 to 2.0)"),
                        )
                        .property(
                            "presence_penalty",
                            SchemaBuilder::number()
                                .description("Presence penalty (-2.0 to 2.0)"),
                        )
                        .property(
                            "stop",
                            SchemaBuilder::string().description("Stop sequence(s)"),
                        )
                        .required(&["model", "messages"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("id", SchemaBuilder::string())
                        .property("model", SchemaBuilder::string())
                        .property(
                            "choices",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("index", SchemaBuilder::integer())
                                    .property(
                                        "message",
                                        SchemaBuilder::object()
                                            .property("role", SchemaBuilder::string())
                                            .property("content", SchemaBuilder::string()),
                                    )
                                    .property("finish_reason", SchemaBuilder::string()),
                            ),
                        )
                        .property(
                            "usage",
                            SchemaBuilder::object()
                                .property("prompt_tokens", SchemaBuilder::integer())
                                .property("completion_tokens", SchemaBuilder::integer())
                                .property("total_tokens", SchemaBuilder::integer()),
                        )
                        .build(),
                )
                .example(
                    "Simple chat",
                    json!({
                        "model": "gpt-4",
                        "messages": [
                            {"role": "user", "content": "Hello!"}
                        ]
                    }),
                )
                .example(
                    "With system prompt",
                    json!({
                        "model": "gpt-4",
                        "messages": [
                            {"role": "system", "content": "You are a helpful assistant."},
                            {"role": "user", "content": "What is the capital of France?"}
                        ],
                        "temperature": 0.7,
                        "max_tokens": 100
                    }),
                )
                .errors(&["INVALID_MODEL", "RATE_LIMIT", "CONTEXT_LENGTH_EXCEEDED"]),

            // openai.complete - Legacy completions
            MethodInfo::new("openai.complete", "Create a legacy text completion")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "model",
                            SchemaBuilder::string()
                                .description("Model ID (e.g., gpt-3.5-turbo-instruct)"),
                        )
                        .property(
                            "prompt",
                            SchemaBuilder::string().description("The prompt to complete"),
                        )
                        .property(
                            "max_tokens",
                            SchemaBuilder::integer()
                                .minimum(1)
                                .description("Maximum tokens to generate"),
                        )
                        .property(
                            "temperature",
                            SchemaBuilder::number()
                                .description("Sampling temperature (0.0 to 2.0)"),
                        )
                        .property(
                            "echo",
                            SchemaBuilder::boolean()
                                .description("Echo back the prompt in addition to completion"),
                        )
                        .required(&["model", "prompt"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property("id", SchemaBuilder::string())
                        .property("model", SchemaBuilder::string())
                        .property(
                            "choices",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("text", SchemaBuilder::string())
                                    .property("index", SchemaBuilder::integer())
                                    .property("finish_reason", SchemaBuilder::string()),
                            ),
                        )
                        .property(
                            "usage",
                            SchemaBuilder::object()
                                .property("prompt_tokens", SchemaBuilder::integer())
                                .property("completion_tokens", SchemaBuilder::integer())
                                .property("total_tokens", SchemaBuilder::integer()),
                        )
                        .build(),
                )
                .example(
                    "Basic completion",
                    json!({
                        "model": "gpt-3.5-turbo-instruct",
                        "prompt": "Say this is a test",
                        "max_tokens": 7
                    }),
                ),

            // openai.embed - Embeddings
            MethodInfo::new("openai.embed", "Generate embeddings for text input")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "model",
                            SchemaBuilder::string()
                                .description("Model ID (e.g., text-embedding-3-small, text-embedding-3-large)"),
                        )
                        .property(
                            "input",
                            SchemaBuilder::string()
                                .description("Text to generate embeddings for (string or array of strings)"),
                        )
                        .property(
                            "dimensions",
                            SchemaBuilder::integer()
                                .minimum(1)
                                .description("Number of dimensions for the output embedding"),
                        )
                        .property(
                            "encoding_format",
                            SchemaBuilder::string()
                                .enum_values(&["float", "base64"])
                                .description("Format for the embedding values"),
                        )
                        .required(&["model", "input"])
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "data",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("index", SchemaBuilder::integer())
                                    .property(
                                        "embedding",
                                        SchemaBuilder::array()
                                            .items(SchemaBuilder::number())
                                            .description("Embedding vector"),
                                    ),
                            ),
                        )
                        .property("model", SchemaBuilder::string())
                        .property(
                            "usage",
                            SchemaBuilder::object()
                                .property("prompt_tokens", SchemaBuilder::integer())
                                .property("total_tokens", SchemaBuilder::integer()),
                        )
                        .build(),
                )
                .example(
                    "Single text embedding",
                    json!({
                        "model": "text-embedding-3-small",
                        "input": "Hello world"
                    }),
                )
                .example(
                    "Multiple embeddings",
                    json!({
                        "model": "text-embedding-3-small",
                        "input": ["Hello", "World"],
                        "dimensions": 256
                    }),
                ),

            // openai.models - List models
            MethodInfo::new("openai.models", "List available models or get a specific model")
                .schema(
                    SchemaBuilder::object()
                        .property(
                            "model",
                            SchemaBuilder::string()
                                .description("Optional: specific model ID to retrieve"),
                        )
                        .build(),
                )
                .returns(
                    SchemaBuilder::object()
                        .property(
                            "models",
                            SchemaBuilder::array().items(
                                SchemaBuilder::object()
                                    .property("id", SchemaBuilder::string())
                                    .property("created", SchemaBuilder::integer())
                                    .property("owned_by", SchemaBuilder::string()),
                            ),
                        )
                        .property("count", SchemaBuilder::integer())
                        .build(),
                )
                .example("List all models", json!({}))
                .example("Get specific model", json!({"model": "gpt-4"})),
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("OpenAIService starting, verifying API connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("OpenAI API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("OpenAI API returned empty models list");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to OpenAI API: {}", e);
                    Err(e)
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
                checks.insert(
                    "openai_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "openai_api".into(),
                    HealthStatus::unhealthy("Empty models list"),
                );
            }
            Err(e) => {
                checks.insert("openai_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_str_returns_none_for_non_string() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), json!(42));
        assert!(OpenAIService::get_str(&params, "key").is_none());
    }

    #[test]
    fn test_get_str_returns_value() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), json!("value"));
        assert_eq!(OpenAIService::get_str(&params, "key"), Some("value"));
    }

    #[test]
    fn test_get_f64_returns_none_for_non_number() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), json!("not a number"));
        assert!(OpenAIService::get_f64(&params, "key").is_none());
    }

    #[test]
    fn test_get_f64_returns_value() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), json!(0.7));
        assert_eq!(OpenAIService::get_f64(&params, "key"), Some(0.7));
    }

    #[test]
    fn test_get_i32_returns_value() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), json!(100));
        assert_eq!(OpenAIService::get_i32(&params, "key"), Some(100));
    }

    #[test]
    fn test_dispatch_rejects_unknown_method() {
        let service = OpenAIService::new(Some("sk-test".to_string())).unwrap();
        let err = service
            .dispatch("openai.unknown", HashMap::new())
            .expect_err("expected unknown method error");
        assert!(err.to_string().contains("Unknown method"));
    }

    #[test]
    fn test_dispatch_chat_requires_model() {
        let service = OpenAIService::new(Some("sk-test".to_string())).unwrap();
        let err = service
            .dispatch("openai.chat", HashMap::new())
            .expect_err("expected missing model error");
        assert!(err.to_string().contains("model"));
    }

    #[test]
    fn test_dispatch_chat_requires_messages() {
        let service = OpenAIService::new(Some("sk-test".to_string())).unwrap();
        let mut params = HashMap::new();
        params.insert("model".to_string(), json!("gpt-4"));
        let err = service
            .dispatch("openai.chat", params)
            .expect_err("expected missing messages error");
        assert!(err.to_string().contains("messages"));
    }

    #[test]
    fn test_dispatch_embed_requires_model() {
        let service = OpenAIService::new(Some("sk-test".to_string())).unwrap();
        let err = service
            .dispatch("openai.embed", HashMap::new())
            .expect_err("expected missing model error");
        assert!(err.to_string().contains("model"));
    }

    #[test]
    fn test_dispatch_embed_requires_input() {
        let service = OpenAIService::new(Some("sk-test".to_string())).unwrap();
        let mut params = HashMap::new();
        params.insert("model".to_string(), json!("text-embedding-3-small"));
        let err = service
            .dispatch("openai.embed", params)
            .expect_err("expected missing input error");
        assert!(err.to_string().contains("input"));
    }

    #[test]
    fn test_method_list_contains_expected_methods() {
        let service = OpenAIService::new(Some("sk-test".to_string())).unwrap();
        let methods = service.method_list();

        let method_names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();

        assert!(method_names.contains(&"openai.chat"));
        assert!(method_names.contains(&"openai.complete"));
        assert!(method_names.contains(&"openai.embed"));
        assert!(method_names.contains(&"openai.models"));
    }

    #[test]
    fn test_chat_schema_requires_model_and_messages() {
        let service = OpenAIService::new(Some("sk-test".to_string())).unwrap();
        let methods = service.method_list();

        let chat_method = methods
            .iter()
            .find(|m| m.name == "openai.chat")
            .expect("expected openai.chat method");

        let schema = chat_method.schema.as_ref().expect("expected schema");

        let required = schema
            .get("required")
            .and_then(|v| v.as_array())
            .expect("expected required array");

        assert!(required.iter().any(|v| v == "model"));
        assert!(required.iter().any(|v| v == "messages"));
    }
}
