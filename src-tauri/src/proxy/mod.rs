pub mod handler;

use axum::{routing, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use crate::AppState;

#[derive(Clone)]
pub struct ProxyState {
    pub app_state: AppState,
    pub app_handle: tauri::AppHandle,
}

pub async fn start_server(state: AppState, app_handle: tauri::AppHandle) {
    let proxy_state = ProxyState {
        app_state: state,
        app_handle,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/v1/chat/completions", routing::post(handler::chat_completions))
        .route("/v1/models", routing::get(handler::list_models))
        .layer(cors)
        .with_state(proxy_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 12434));
    tracing::info!("Proxy IA Middleware escuchando en http://{addr}");

    match tokio::net::TcpListener::bind(addr).await {
        Err(e) => tracing::error!("No se pudo iniciar el proxy en {addr}: {e}"),
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("Proxy terminó con error: {e}");
            }
        }
    }
}
