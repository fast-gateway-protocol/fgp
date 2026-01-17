//! Data models for Ollama API requests and responses.

use serde::{Deserialize, Serialize};

/// Request for text generation.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
    /// Set to false to get the full response at once (not streaming)
    #[serde(default = "default_false")]
    pub stream: bool,
}

fn default_false() -> bool {
    false
}

/// Response from text generation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenerateResponse {
    pub model: String,
    pub response: String,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub context: Option<Vec<i64>>,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

/// Request for chat completion.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
    /// Set to false to get the full response at once (not streaming)
    #[serde(default = "default_false")]
    pub stream: bool,
}

/// A message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

/// Response from chat completion.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatResponse {
    pub model: String,
    pub message: ChatMessage,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

/// Request for embeddings.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

/// Response from embeddings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f64>,
}

/// Options for generation (temperature, top_p, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f64>,
}

/// Request to pull a model.
#[derive(Debug, Clone, Serialize)]
pub struct PullRequest {
    pub name: String,
    #[serde(default = "default_false")]
    pub stream: bool,
}

/// Response from pulling a model.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PullResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub completed: Option<u64>,
}

/// Request to show model info.
#[derive(Debug, Clone, Serialize)]
pub struct ShowRequest {
    pub name: String,
}

/// Response from showing model info.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShowResponse {
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
    #[serde(default)]
    pub details: Option<ModelDetails>,
}

/// Model details.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    #[serde(default)]
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

/// Response from listing models.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListResponse {
    pub models: Vec<ModelInfo>,
}

/// Information about a model.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    #[serde(default)]
    pub details: Option<ModelDetails>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_request_serialization() {
        let req = GenerateRequest {
            model: "llama2".to_string(),
            prompt: "Hello world".to_string(),
            system: None,
            template: None,
            context: None,
            options: None,
            stream: false,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("llama2"));
        assert!(json.contains("Hello world"));
        assert!(json.contains("\"stream\":false"));
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hi there".to_string(),
            images: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hi there"));
    }

    #[test]
    fn test_generate_options_serialization() {
        let opts = GenerateOptions {
            temperature: Some(0.8),
            top_p: Some(0.9),
            top_k: None,
            num_predict: Some(100),
            num_ctx: None,
            seed: None,
            stop: Some(vec!["END".to_string()]),
            repeat_penalty: None,
        };

        let json = serde_json::to_string(&opts).unwrap();
        assert!(json.contains("0.8"));
        assert!(json.contains("0.9"));
        assert!(json.contains("100"));
        assert!(json.contains("END"));
    }

    #[test]
    fn test_generate_response_deserialization() {
        let json = r#"{
            "model": "llama2",
            "response": "Hello!",
            "done": true,
            "total_duration": 1000000
        }"#;

        let resp: GenerateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.model, "llama2");
        assert_eq!(resp.response, "Hello!");
        assert!(resp.done);
        assert_eq!(resp.total_duration, Some(1000000));
    }

    #[test]
    fn test_model_info_deserialization() {
        let json = r#"{
            "name": "llama2:latest",
            "modified_at": "2024-01-14T00:00:00Z",
            "size": 3826793472,
            "digest": "sha256:abc123"
        }"#;

        let info: ModelInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "llama2:latest");
        assert_eq!(info.size, 3826793472);
    }
}
