pub mod db;
pub mod error;
pub mod providers;
pub mod proxy;
pub mod vault;

use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use vault::Vault;
use error::AppResult;

/// Estado compartido de la aplicación Tauri.
/// Clone barato — todos los campos son Arc.
#[derive(Clone)]
pub struct AppState {
    pub vault: Arc<Vault>,
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let vault = Arc::new(Vault::init().expect("No se pudo inicializar la bóveda"));
    let conn = db::open_db().expect("No se pudo abrir la base de datos");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            vault,
            db: Arc::new(Mutex::new(conn)),
        })
        .setup(|app| {
            let state = app.state::<AppState>().inner().clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(proxy::start_server(state, handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Fase A — Vault
            get_vault_status,
            unlock_vault,
            lock_vault,
            store_api_key,
            // Fase B — Proxy, métricas, presupuestos
            get_metrics_summary,
            list_budget_statuses,
            get_budget_status,
            get_proxy_status,
        ])
        .run(tauri::generate_context!())
        .expect("Error al iniciar la aplicación Tauri");
}

// ══ Fase A — Vault ════════════════════════════════════════════════════════════

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn get_vault_status(state: State<'_, AppState>) -> serde_json::Value {
    serde_json::json!({ "unlocked": state.vault.is_unlocked() })
}

#[tauri::command]
fn unlock_vault(master_password: String, state: State<'_, AppState>) -> AppResult<()> {
    state.vault.unlock(&master_password)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn lock_vault(state: State<'_, AppState>) {
    state.vault.lock();
}

/// Almacena una API key en la bóveda. El secreto nunca retorna al frontend.
#[tauri::command]
fn store_api_key(provider: String, api_key: String, state: State<'_, AppState>) -> AppResult<()> {
    state.vault.store_api_key(&provider, &api_key)
}

// ══ Fase B — Métricas y proxy ═════════════════════════════════════════════════

#[tauri::command]
async fn get_metrics_summary(
    period: String,
    state: State<'_, AppState>,
) -> AppResult<serde_json::Value> {
    let conn = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = conn
            .lock()
            .map_err(|_| error::AppError::Config("db lock poisoned".into()))?;
        db::query_metrics_summary(&guard, &period)
    })
    .await
    .map_err(|e| error::AppError::Config(format!("task: {e}")))?
}

#[tauri::command]
async fn list_budget_statuses(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let conn = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = conn
            .lock()
            .map_err(|_| error::AppError::Config("db lock poisoned".into()))?;
        let rows = db::query_budgets(&guard)?;
        Ok(serde_json::json!(rows))
    })
    .await
    .map_err(|e| error::AppError::Config(format!("task: {e}")))?
}

#[tauri::command]
async fn get_budget_status(
    project_tag: String,
    state: State<'_, AppState>,
) -> AppResult<serde_json::Value> {
    let conn = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = conn
            .lock()
            .map_err(|_| error::AppError::Config("db lock poisoned".into()))?;
        let all = db::query_budgets(&guard)?;
        let found = all.into_iter().find(|b| b["projectTag"] == project_tag);
        Ok(found.unwrap_or(serde_json::Value::Null))
    })
    .await
    .map_err(|e| error::AppError::Config(format!("task: {e}")))?
}

#[tauri::command]
fn get_proxy_status() -> serde_json::Value {
    serde_json::json!({
        "running": true,
        "port": 12434,
        "requestsTotal": 0,
        "startedAt": chrono::Utc::now().timestamp_millis()
    })
}
