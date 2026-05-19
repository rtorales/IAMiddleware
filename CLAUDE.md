# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Stack

- **Backend**: Rust (stable) + Tauri v2 — lógica, red externa, criptografía
- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS — UI pasiva únicamente
- **DB**: SQLite vía `rusqlite` (bundled) — datos en `{data_dir}/ia-middleware/middleware.db`
- **Vault fallback**: `{data_dir}/ia-middleware/vault.db` (AES-256-GCM + Argon2id)
- **Orquestador local**: Ollama — qwen2.5:3b (router), qwen2.5:1.5b (clasificador)

## Prerrequisitos

```
rustup (stable)              → winget install Rustlang.Rustup
cargo install tauri-cli --version "^2"
Ollama                       → winget install Ollama.Ollama
Node.js ≥ 18 (ya disponible)
```

## Comandos

```bash
npm install                  # deps frontend
npm run tauri dev            # dev con hot-reload (inicia Vite en :1420 + Tauri)
npm run tauri build          # release multiplataforma

cd src-tauri
cargo check                  # verificar sin compilar
cargo test                   # tests unitarios
cargo clippy -- -D warnings  # linting — sin warnings tolerados
```

## REGLA ABSOLUTA: Aislamiento Criptográfico

Las llaves API, tokens Bearer y material criptográfico **nunca** deben aparecer en `src/`, stores Zustand ni payloads IPC de Tauri.

- Todo HTTP externo (OpenAI, Anthropic, etc.) se construye y ejecuta en `src-tauri/`
- El frontend recibe únicamente: métricas agregadas, estados booleanos, texto de respuesta
- Bóveda primaria: crate `keyring` → OS keychain (Windows Credential Manager / macOS Keychain)
- Bóveda fallback: SQLite + AES-256-GCM; clave derivada con Argon2id (m=64MB, t=3, p=4)
- Al agregar nuevos comandos IPC, verificar con `/auditoria-cripto` antes de hacer commit

## Arquitectura Rust (`src-tauri/src/`)

```
error.rs         → AppError (thiserror) + impl serde::Serialize para IPC
lib.rs           → AppState { vault: Arc<Vault> }, registro de comandos, entry point
vault/           → mod.rs selecciona backend (keyring si is_available(), sqlite_vault fallback)
  kdf.rs         → Argon2id::hash_password_into → [u8; 32] para AES key
  keyring_vault.rs   → Entry::new(service, account).set/get_password()
  sqlite_vault.rs    → AES-256-GCM: nonce único por secreto, cipher en Mutex<Option<Aes256Gcm>>
db/
  mod.rs         → open_db() abre WAL + foreign_keys y ejecuta migraciones
  schema.rs      → MIGRATION_V1: todas las tablas DDL + inserts iniciales de providers
```

**Módulos pendientes (Fases B–E — activar por feature flag):**
```
proxy/       → Axum server :12434 (feature "proxy"   → dep:axum)
providers/   → trait LlmProvider + adaptadores       (feature "providers" → dep:reqwest)
router/      → OllamaSemanticRouter, scoring τ
budget/      → tracker.rs + alertas 60/85/100%
telemetry/   → métricas tiempo real
```
Para activar Fase B: en `Cargo.toml` cambiar `[features] default = ["proxy", "providers"]`.

## Patrones de Código Rust

- **Errores**: propagar con `?` hacia `AppError`; nunca `unwrap()` en producción
- **Secretos en memoria**: `Zeroizing<T>` y `secrecy::SecretString` (se zeroizan en drop)
- **Streaming LLM**: `mpsc::Sender<StreamChunk>` — nunca bufferizar respuesta completa
- Prohibido: `#[allow(warnings)]`, `#[allow(dead_code)]`

## Patrones de Código Frontend

- **IPC Tauri v2**: `invoke` desde `@tauri-apps/api/core`; `listen` desde `@tauri-apps/api/event`
- **Estado**: solo Zustand (`appStore.ts`) — nunca secretos, nunca API keys
- **Hooks IPC**: cleanup obligatorio → `unlisten.then((fn: () => void) => fn())`
- **TypeScript**: `strict: true` activo — sin `any` implícito

## Comandos IPC Implementados (Fase A)

```
get_vault_status  → { unlocked: bool }
unlock_vault      ← { masterPassword: string }
lock_vault        ← {}
store_api_key     ← { provider: string, api_key: string }   // sin retorno del secret
```

Eventos emitidos al frontend: `vault_locked` (al bloquear por inactividad o cierre de sesión OS).
