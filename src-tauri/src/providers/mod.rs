pub mod types;
pub mod openai;
pub mod anthropic;
pub mod ollama;

use async_trait::async_trait;
use futures::stream::BoxStream;
use std::sync::Arc;
use crate::error::AppResult;
use types::ChatRequest;

/// Stream de deltas de texto proveniente del proveedor.
pub type ChunkStream = BoxStream<'static, AppResult<String>>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;

    /// Inicia una llamada en modo streaming y retorna el stream de deltas.
    async fn stream_chat(&self, req: &ChatRequest, api_key: &str) -> AppResult<ChunkStream>;
}

/// Selecciona el proveedor correcto a partir del nombre del modelo.
pub fn provider_for_model(model: &str) -> &'static str {
    if model.starts_with("gpt-")
        || model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
    {
        "openai"
    } else if model.starts_with("claude-") {
        "anthropic"
    } else if model.starts_with("deepseek-") {
        "deepseek"
    } else if model.starts_with("gemini-") {
        "gemini"
    } else {
        "ollama"
    }
}

/// Instancia el proveedor concreto por nombre.
pub fn get_provider(name: &str) -> Option<Arc<dyn LlmProvider>> {
    match name {
        "openai" => Some(Arc::new(openai::OpenAiProvider::new(
            "https://api.openai.com/v1",
        ))),
        "anthropic" => Some(Arc::new(anthropic::AnthropicProvider::new())),
        "deepseek" => Some(Arc::new(openai::OpenAiProvider::new(
            "https://api.deepseek.com/v1",
        ))),
        "ollama" => Some(Arc::new(ollama::OllamaProvider::new())),
        _ => None,
    }
}
