# IA Middleware — Guía de Proyecto para Claude Code

## Stack Tecnológico
- **Backend**: Rust (stable toolchain) + Tauri v2
- **Frontend**: React 18 + TypeScript + Tailwind CSS + Vite
- **DB local**: SQLite vía `rusqlite` (encriptada AES-256-GCM en bóveda fallback)
- **Orquestador**: Ollama (qwen2.5:3b router principal, qwen2.5:1.5b clasificador)
- **HTTP externo**: `reqwest` con `rustls` (NO `openssl` nativo — mantiene binario ligero)

## Prerrequisitos de Entorno (instalar antes de compilar)
```
rustup (stable)    → https://rustup.rs
cargo-tauri v2     → cargo install tauri-cli --version "^2"
Ollama             → https://ollama.ai
Node.js ≥ 18       → ya instalado (v24.15.0)
```

## Comandos Clave
```bash
npm install                    # Instalar deps frontend
npm run tauri dev              # Dev con hot-reload
npm run tauri build            # Release multiplataforma
cd src-tauri && cargo check    # Verificar Rust sin compilar
cd src-tauri && cargo test     # Tests unitarios Rust
cd src-tauri && cargo clippy   # Linting estricto
```

## REGLA ABSOLUTA: Aislamiento Criptográfico del Frontend

> Las llaves API, tokens Bearer y material criptográfico JAMÁS deben
> aparecer en src/, componentes React, stores Zustand ni payloads IPC.

- Toda petición HTTP a proveedores externos se construye y ejecuta en `src-tauri/`
- El frontend recibe ÚNICAMENTE: métricas agregadas, estados, textos de respuesta
- IPC Tauri emite solo eventos de estado/métricas — nunca secretos en el payload
- Bóveda primaria: crate `keyring` (Windows Credential Manager / macOS Keychain)
- Bóveda fallback: SQLite + AES-256-GCM derivada con Argon2id desde contraseña maestra

## Convenciones de Código Rust
- PROHIBIDO: `#[allow(warnings)]`, `#[allow(dead_code)]`, `unwrap()` en prod
- Propagar errores con `?` y el tipo unificado `AppError` (definido en `error.rs`)
- Usar `Zeroizing<T>` y `SecretString` del crate `secrecy` para material sensible en memoria
- Streaming de respuestas LLM: `mpsc::Sender<StreamChunk>` — nunca bufferizar respuesta completa
- Operaciones DB y red: siempre `async` con `tokio`

## Módulos Rust (`src-tauri/src/`)
```
vault/        → Keyring OS + fallback SQLite AES (gestión de secretos)
proxy/        → Servidor Axum OpenAI-compatible (port 12434)
router/       → Motor de enrutamiento semántico multi-objetivo
providers/    → Adaptadores por proveedor (trait LlmProvider)
db/           → Esquemas SQLite, migraciones, DAOs
budget/       → Motor de presupuestos y alertas escalonadas (60/85/100%)
telemetry/    → Métricas tiempo real: tokens, latencia, tasas de error
```

## Política de Cambios
- NO modificar archivos sin diagramar el cambio primero
- NO generar bloques masivos de código sin aprobación explícita por módulo
- Resolver errores de compilación investigando la causa raíz — no silenciar
