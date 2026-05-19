use serde::{Deserialize, Serialize};

/// Petición OpenAI-compatible (entrada del proxy y de los adaptadores).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Métricas de uso opcionales que el proveedor puede reportar al final del stream.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Resultado final después de consumir el stream (para logging).
#[derive(Debug, Default)]
pub struct StreamStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub error: Option<String>,
}
