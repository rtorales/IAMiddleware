pub mod db;
pub mod error;
pub mod vault;

// Stubs para Fases B-E (se activarán módulo por módulo)
// pub mod proxy;
// pub mod router;
// pub mod providers;
// pub mod budget;
// pub mod telemetry;

use std::sync::Arc;
use tauri::{Manager, State};
use vault::Vault;
use error::AppResult;

/// Estado compartido de la aplicación Tauri.
pub struct AppState {
    pub vault: Arc<Vault>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let vault = Arc::new(Vault::init().expect("No se pudo inicializar la bóveda"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState { vault: vault.clone() })
        .invoke_handler(tauri::generate_handler![
            cmd_get_vault_status,
            cmd_unlock_vault,
            cmd_lock_vault,
            cmd_store_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("Error al iniciar la aplicación Tauri");
}

// === Comandos IPC — Fase A (Vault) ===

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn cmd_get_vault_status(state: State<'_, AppState>) -> serde_json::Value {
    serde_json::json!({ "unlocked": state.vault.is_unlocked() })
}

#[tauri::command]
fn cmd_unlock_vault(
    master_password: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.vault.unlock(&master_password)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn cmd_lock_vault(state: State<'_, AppState>) {
    state.vault.lock();
}

/// Almacena una API key en la bóveda. NO retorna el valor — el frontend solo recibe Ok/Err.
#[tauri::command]
fn cmd_store_api_key(
    provider: String,
    api_key: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.vault.store_api_key(&provider, &api_key)
}
