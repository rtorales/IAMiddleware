use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use crate::error::{AppError, AppResult};
use super::{ChunkStream, LlmProvider};
use super::types::ChatRequest;

pub struct AnthropicProvider {
    client: Client,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Convierte mensajes OpenAI al formato Anthropic (separa system del resto).
    fn build_body(req: &ChatRequest) -> serde_json::Value {
        let mut system = String::new();
        let mut messages = Vec::new();

        for msg in &req.messages {
            if msg.role == "system" {
                system = msg.content.clone();
            } else {
                messages.push(json!({"role": msg.role, "content": msg.content}));
            }
        }

        let mut body = json!({
            "model": req.model,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "messages": messages,
            "stream": true,
        });

        if !system.is_empty() {
            body["system"] = json!(system);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }

        body
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn stream_chat(&self, req: &ChatRequest, api_key: &str) -> AppResult<ChunkStream> {
        let body = Self::build_body(req);

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("anthropic request: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!("Anthropic {status}: {text}")));
        }

        let (tx, rx) = mpsc::channel::<AppResult<String>>(64);

        tokio::spawn(async move {
            let mut buf = String::new();
            let mut byte_stream = response.bytes_stream();

            while let Some(result) = byte_stream.next().await {
                match result {
                    Err(e) => {
                        let _ = tx
                            .send(Err(AppError::Provider(format!("anthropic stream: {e}"))))
                            .await;
                        return;
                    }
                    Ok(bytes) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                        loop {
                            match buf.find('\n') {
                                None => break,
                                Some(pos) => {
                                    let line = buf[..pos].trim_end_matches('\r').to_string();
                                    buf = buf[pos + 1..].to_string();

                                    // Solo procesamos líneas "data: ..."
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        if let Ok(v) =
                                            serde_json::from_str::<serde_json::Value>(data)
                                        {
                                            // content_block_delta con text_delta
                                            if v["type"] == "content_block_delta"
                                                && v["delta"]["type"] == "text_delta"
                                            {
                                                if let Some(text) =
                                                    v["delta"]["text"].as_str()
                                                {
                                                    if !text.is_empty()
                                                        && tx
                                                            .send(Ok(text.to_string()))
                                                            .await
                                                            .is_err()
                                                    {
                                                        return;
                                                    }
                                                }
                                            }
                                            // message_stop → fin del stream
                                            if v["type"] == "message_stop" {
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}
