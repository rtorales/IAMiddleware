pub mod db;
pub mod error;
pub mod providers;
pub mod proxy;
pub mod router;
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
            check_api_key,
            // Fase B — Proxy, métricas, presupuestos
            get_metrics_summary,
            list_budget_statuses,
            get_budget_status,
            get_proxy_status,
            create_budget,
            // Configuración de router
            get_routing_config,
            save_routing_config,
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

/// Indica si ya existe una key para el proveedor. Solo retorna bool — nunca el secreto.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn check_api_key(provider: String, state: State<'_, AppState>) -> bool {
    state.vault.has_api_key(&provider)
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
async fn create_budget(
    project_tag: String,
    limit_usd: f64,
    period: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let conn = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = conn.lock().map_err(|_| error::AppError::Config("db lock poisoned".into()))?;
        let now = chrono::Utc::now().timestamp_millis();
        let id = uuid::Uuid::new_v4().to_string();
        guard.execute(
            "INSERT INTO budgets
             (id,project_tag,limit_usd,period,period_start_at,spent_usd,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,0.0,?5,?5)
             ON CONFLICT(project_tag) DO UPDATE
             SET limit_usd=?3, period=?4, updated_at=?5,
                 alert_60_sent_at=NULL, alert_85_sent_at=NULL, hard_locked_at=NULL",
            rusqlite::params![id, project_tag, limit_usd, period, now],
        )?;
        Ok(())
    })
    .await
    .map_err(|e| error::AppError::Config(format!("task: {e}")))?
}

#[tauri::command]
async fn get_routing_config(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let conn = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = conn.lock().map_err(|_| error::AppError::Config("db lock poisoned".into()))?;
        let row = guard.query_row(
            "SELECT tau, ollama_router_url, router_model, classifier_model,
                    complexity_threshold_low, complexity_threshold_high, fallback_to_local
             FROM routing_config WHERE id = 'singleton'",
            [],
            |row| Ok(serde_json::json!({
                "tau":                     row.get::<_,f64>(0)?,
                "ollamaRouterUrl":         row.get::<_,String>(1)?,
                "routerModel":             row.get::<_,String>(2)?,
                "classifierModel":         row.get::<_,String>(3)?,
                "complexityThresholdLow":  row.get::<_,f64>(4)?,
                "complexityThresholdHigh": row.get::<_,f64>(5)?,
                "fallbackToLocal":         row.get::<_,bool>(6)?,
            })),
        ).map_err(|e| error::AppError::Config(e.to_string()))?;
        Ok(row)
    })
    .await
    .map_err(|e| error::AppError::Config(format!("task: {e}")))?
}

#[tauri::command]
async fn save_routing_config(
    tau: f64,
    ollama_router_url: String,
    router_model: String,
    classifier_model: String,
    complexity_threshold_low: f64,
    complexity_threshold_high: f64,
    fallback_to_local: bool,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let conn = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let guard = conn.lock().map_err(|_| error::AppError::Config("db lock poisoned".into()))?;
        let now = chrono::Utc::now().timestamp_millis();
        guard.execute(
            "UPDATE routing_config SET
               tau=?1, ollama_router_url=?2, router_model=?3,
               classifier_model=?4, complexity_threshold_low=?5,
               complexity_threshold_high=?6, fallback_to_local=?7, updated_at=?8
             WHERE id='singleton'",
            rusqlite::params![
                tau, ollama_router_url, router_model, classifier_model,
                complexity_threshold_low, complexity_threshold_high,
                fallback_to_local, now
            ],
        )?;
        Ok(())
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
