use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use crate::error::{AppError, AppResult};
use super::{ChunkStream, LlmProvider};
use super::types::ChatRequest;

/// Adaptador compatible con el protocolo OpenAI v1 (también sirve para DeepSeek).
pub struct OpenAiProvider {
    client: Client,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn stream_chat(&self, req: &ChatRequest, api_key: &str) -> AppResult<ChunkStream> {
        let mut body = serde_json::to_value(req)
            .map_err(|e| AppError::Provider(format!("serialize: {e}")))?;
        body["stream"] = serde_json::json!(true);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!("API {status}: {text}")));
        }

        let (tx, rx) = mpsc::channel::<AppResult<String>>(64);

        tokio::spawn(async move {
            let mut buf = String::new();
            let mut byte_stream = response.bytes_stream();

            while let Some(result) = byte_stream.next().await {
                match result {
                    Err(e) => {
                        let _ = tx
                            .send(Err(AppError::Provider(format!("stream read: {e}"))))
                            .await;
                        return;
                    }
                    Ok(bytes) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                        loop {
                            match buf.find('\n') {
                                None => break,
                                Some(pos) => {
                                    let line = buf[..pos]
                                        .trim_end_matches('\r')
                                        .to_string();
                                    buf = buf[pos + 1..].to_string();

                                    if let Some(data) = line.strip_prefix("data: ") {
                                        if data.trim() == "[DONE]" {
                                            return;
                                        }
                                        if let Ok(v) =
                                            serde_json::from_str::<serde_json::Value>(data)
                                        {
                                            if let Some(content) =
                                                v["choices"][0]["delta"]["content"].as_str()
                                            {
                                                if !content.is_empty()
                                                    && tx
                                                        .send(Ok(content.to_string()))
                                                        .await
                                                        .is_err()
                                                {
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
            }
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}
