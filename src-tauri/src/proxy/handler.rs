use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse, Json, Response,
    },
};
use futures::StreamExt;
use serde_json::json;
use std::{convert::Infallible, sync::{Arc, atomic::{AtomicU32, Ordering}}};
use tauri::Emitter;
use uuid::Uuid;

use crate::{
    db, router,
    providers::{get_provider, provider_for_model, types::ChatRequest},
};
use super::ProxyState;

pub async fn chat_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(mut req): Json<ChatRequest>,
) -> Response {
    // ── Router semántico ──────────────────────────────────────────────────────
    let mut router_decision: Option<String> = None;

    let provider_name: &'static str = if req.model.trim() == "auto" {
        let prompt = req.messages.iter().rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let config = {
            let conn = state.app_state.db.lock().unwrap();
            router::load_config(&conn)
        };

        let decision = router::route(prompt, &config, &state.app_state.vault, &state.http_client).await;

        router_decision = Some(serde_json::json!({
            "complexity":    decision.complexity,
            "taskType":      "auto",
            "tauUsed":       config.tau,
            "routeReason":   decision.reason,
            "estimatedCost": 0.0,
            "confidence":    1.0 - (decision.complexity - config.threshold_low).abs()
        }).to_string());

        req.model = decision.model;
        decision.provider
    } else {
        provider_for_model(&req.model)
    };

    // ── Despacho al proveedor ─────────────────────────────────────────────────
    let provider = match get_provider(provider_name) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":{"message": format!("proveedor '{}' no soportado", provider_name)}})),
            )
                .into_response();
        }
    };

    let api_key = resolve_api_key(&state, provider_name, &headers);

    if api_key.is_empty() && provider_name != "ollama" {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":{"message":"API key no encontrada. Usa store_api_key IPC o envía Authorization: Bearer <key>"}})),
        )
            .into_response();
    }

    let start_ms = chrono::Utc::now().timestamp_millis();
    let prompt_tokens: u32 = (req.messages.iter().map(|m| m.content.len()).sum::<usize>() / 4) as u32;

    match provider.stream_chat(&req, &api_key).await {
        Err(e) => {
            tracing::error!("[proxy] provider error: {e}");
            let db = state.app_state.db.clone();
            let handle = state.app_handle.clone();
            schedule_log(db, handle, provider_name, req.model, 0, true, 0, 0, router_decision);
            (StatusCode::BAD_GATEWAY, Json(json!({"error":{"message": e.to_string()}}))).into_response()
        }
        Ok(stream) if req.stream => {
            sse_response(state, req.model, provider_name, stream, start_ms, prompt_tokens, router_decision)
        }
        Ok(stream) => {
            json_response(state, req.model, provider_name, stream, start_ms, prompt_tokens, router_decision).await
        }
    }
}

pub async fn list_models() -> Json<serde_json::Value> {
    Json(json!({
        "object": "list",
        "data": [
            {"id":"gpt-4o",                    "object":"model","owned_by":"openai"},
            {"id":"gpt-4o-mini",               "object":"model","owned_by":"openai"},
            {"id":"claude-sonnet-4-6",         "object":"model","owned_by":"anthropic"},
            {"id":"claude-haiku-4-5-20251001", "object":"model","owned_by":"anthropic"},
            {"id":"deepseek-chat",             "object":"model","owned_by":"deepseek"},
            {"id":"qwen2.5:3b",                "object":"model","owned_by":"ollama"},
            {"id":"qwen2.5:1.5b",              "object":"model","owned_by":"ollama"},
        ]
    }))
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn resolve_api_key(state: &ProxyState, provider: &str, headers: &HeaderMap) -> String {
    let vault = &state.app_state.vault;
    if vault.is_unlocked() {
        if let Ok(k) = vault.get_api_key(provider) {
            let s = String::from_utf8(k.to_vec()).unwrap_or_default();
            if !s.is_empty() {
                return s;
            }
        }
    }
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_string()
}

fn sse_response(
    state: ProxyState,
    model: String,
    provider_name: &'static str,
    chunk_stream: crate::providers::ChunkStream,
    start_ms: i64,
    prompt_tokens: u32,
    router_decision: Option<String>,
) -> Response {
    let req_id = format!("chatcmpl-{}", Uuid::new_v4().simple());
    let req_id_clone = req_id.clone();
    let db = state.app_state.db.clone();
    let handle = state.app_handle.clone();
    let model_clone = model.clone();
    let char_count = Arc::new(AtomicU32::new(0));
    let char_count_chain = char_count.clone();

    let sse_stream = chunk_stream
        .map(move |result| -> Result<Event, Infallible> {
            let data = match result {
                Ok(text) => {
                    char_count.fetch_add(text.len() as u32, Ordering::Relaxed);
                    json!({
                        "id": req_id,
                        "object": "chat.completion.chunk",
                        "choices": [{"index":0,"delta":{"content":text},"finish_reason":null}]
                    })
                    .to_string()
                }
                Err(e) => json!({"error":{"message":e.to_string()}}).to_string(),
            };
            Ok(Event::default().data(data))
        })
        .chain(futures::stream::once(async move {
            let latency = (chrono::Utc::now().timestamp_millis() - start_ms) as i64;
            let completion_tokens = char_count_chain.load(Ordering::Relaxed) / 4;
            schedule_log(db, handle, provider_name, model_clone, latency, false, prompt_tokens, completion_tokens, router_decision);
            Ok::<Event, Infallible>(Event::default().data("[DONE]").id(req_id_clone))
        }));

    Sse::new(sse_stream).into_response()
}

async fn json_response(
    state: ProxyState,
    model: String,
    provider_name: &'static str,
    chunk_stream: crate::providers::ChunkStream,
    start_ms: i64,
    prompt_tokens: u32,
    router_decision: Option<String>,
) -> Response {
    let mut full_text = String::new();
    let mut stream = chunk_stream;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(t) => full_text.push_str(&t),
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"error":{"message":e.to_string()}})),
                )
                    .into_response();
            }
        }
    }

    let latency = (chrono::Utc::now().timestamp_millis() - start_ms) as i64;
    let completion_tokens = (full_text.len() as u32).saturating_div(4);
    let db = state.app_state.db.clone();
    let handle = state.app_handle.clone();
    schedule_log(db, handle, provider_name, model.clone(), latency, false, prompt_tokens, completion_tokens, router_decision);

    Json(json!({
        "id": format!("chatcmpl-{}", Uuid::new_v4().simple()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{"index":0,"message":{"role":"assistant","content":full_text},"finish_reason":"stop"}],
        "usage": {"prompt_tokens":prompt_tokens,"completion_tokens":completion_tokens,"total_tokens":prompt_tokens + completion_tokens}
    }))
    .into_response()
}

/// Lanza una tarea que inserta la petición en la DB, actualiza budgets y emite eventos.
fn schedule_log(
    db_arc: Arc<std::sync::Mutex<rusqlite::Connection>>,
    handle: tauri::AppHandle,
    provider: &'static str,
    model: String,
    latency_ms: i64,
    is_error: bool,
    prompt_tokens: u32,
    completion_tokens: u32,
    router_decision: Option<String>,
) {
    let db_clone = db_arc.clone();

    tokio::spawn(async move {
        let handle_budget = handle.clone();
        let _ = tokio::task::spawn_blocking(move || {
            let conn = db_clone.lock().unwrap();
            let cost_usd = db::pricing::estimate_cost(provider, &model, prompt_tokens, completion_tokens, &conn);
            let id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp_millis();
            let status = if is_error { "error" } else { "success" };
            let total = prompt_tokens + completion_tokens;

            let _ = conn.execute(
                "INSERT INTO requests \
                 (id,created_at,virtual_key_id,project_tag,provider,model,\
                  prompt_tokens,completion_tokens,total_tokens,cost_usd,latency_ms,status,router_decision) \
                 VALUES (?1,?2,'default','default',?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                rusqlite::params![id, now, provider, model, prompt_tokens, completion_tokens, total, cost_usd, latency_ms, status, router_decision],
            );

            db::update_budget_and_alerts(&conn, &handle_budget, "default", cost_usd);
        })
        .await;

        if let Ok(payload) = db::metrics_update_payload(db_arc).await {
            let _ = handle.emit("metrics_update", payload);
        }
    });
}
