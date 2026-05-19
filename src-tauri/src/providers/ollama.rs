use async_trait::async_trait;
use crate::error::AppResult;
use super::{ChunkStream, LlmProvider};
use super::openai::OpenAiProvider;
use super::types::ChatRequest;

/// Ollama expone el endpoint /v1/chat/completions compatible con OpenAI.
/// Reusa el adaptador OpenAI con la URL local.
pub struct OllamaProvider {
    inner: OpenAiProvider,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            inner: OpenAiProvider::new("http://localhost:11434/v1"),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn stream_chat(&self, req: &ChatRequest, _api_key: &str) -> AppResult<ChunkStream> {
        // Ollama no requiere API key en local
        self.inner.stream_chat(req, "ollama").await
    }
}
