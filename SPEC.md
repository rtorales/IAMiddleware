# SPEC.md — Arquitectura Maestra: IA Middleware (API Gateway)

> Versión: 0.1 — Estado: BORRADOR para aprobación  
> Stack: Tauri v2 + Rust + React 18 + TypeScript + SQLite + Ollama

---

## 1. Visión del Sistema

Un API Gateway de escritorio multiplataforma que:
1. Expone un proxy HTTP **compatible con OpenAI** en `localhost:12434`
2. Enruta peticiones hacia múltiples proveedores de IA con decisión semántica local
3. Gestiona credenciales sin jamás exponerlas al WebView (aislamiento total)
4. Audita costos, latencia y tokens con dashboards en tiempo real
5. Aplica políticas de presupuesto multiteniente con alertas escalonadas

---

## 2. Prerrequisitos de Instalación

| Herramienta | Versión mínima | Estado actual | Cómo instalar |
|---|---|---|---|
| Rust (stable) | 1.77+ | ❌ No instalado | `winget install Rustlang.Rustup` |
| cargo-tauri | 2.x | ❌ No instalado | `cargo install tauri-cli --version "^2"` |
| Ollama | latest | ❌ No instalado | https://ollama.ai/download |
| Node.js | 18+ | ✅ v24.15.0 | — |
| Git | any | ✅ v2.54.0 | — |
| WebView2 Runtime | any | ✅ Windows 11 nativo | — |

**Modelos Ollama requeridos:**
```bash
ollama pull qwen2.5:3b      # Router principal (~2GB, VRAM ~3.5GB Q4_K_M)
ollama pull qwen2.5:1.5b    # Clasificador rápido (~1GB, VRAM ~2GB Q4_K_M)
```

---

## 3. Estructura de Carpetas

```
C:\DESARROLLO\IAMiddleware\
├── CLAUDE.md                        ← Reglas del proyecto para Claude Code
├── SPEC.md                          ← Este documento
├── .claude/
│   ├── agents/
│   │   ├── crypto-auditor.md        ← Auditor solo lectura (frontend leaks)
│   │   ├── rust-backend.md          ← Especialista src-tauri/
│   │   └── frontend-ui.md           ← Especialista src/
│   └── skills/
│       └── auditoria-cripto/
│           └── SKILL.md             ← Skill /auditoria-cripto
│
├── src/                             ← Frontend React/TypeScript (UI PASIVA)
│   ├── main.tsx                     ← Entry point React
│   ├── App.tsx                      ← Routing principal
│   ├── components/
│   │   ├── dashboard/
│   │   │   ├── MetricsPanel.tsx     ← Panel principal de métricas
│   │   │   ├── CostTracker.tsx      ← Gráfica costo por proyecto/tag
│   │   │   ├── LatencyChart.tsx     ← Histograma de latencia
│   │   │   ├── TokenCounter.tsx     ← Conteo tokens entrada/salida
│   │   │   └── RequestFeed.tsx      ← Feed en tiempo real de peticiones
│   │   ├── settings/
│   │   │   ├── ProviderList.tsx     ← Habilitar/deshabilitar proveedores
│   │   │   ├── BudgetConfig.tsx     ← Configurar límites por proyecto
│   │   │   ├── VirtualKeys.tsx      ← Gestión de llaves virtuales
│   │   │   └── RouterConfig.tsx     ← Parámetro τ y pesos del router
│   │   ├── vault/
│   │   │   └── VaultUnlock.tsx      ← UI de desbloqueo de bóveda
│   │   └── common/
│   │       ├── StatusBadge.tsx
│   │       ├── AlertBanner.tsx
│   │       └── ProviderHealthDot.tsx
│   ├── hooks/
│   │   ├── useMetrics.ts            ← Suscripción a eventos IPC de métricas
│   │   ├── useBudget.ts             ← Estado de presupuestos
│   │   ├── useProviders.ts          ← Estado salud proveedores
│   │   └── useVault.ts              ← Estado de la bóveda (locked/unlocked)
│   ├── stores/
│   │   └── appStore.ts              ← Zustand store (solo UI state, sin secretos)
│   └── types/
│       └── index.ts                 ← Tipos compartidos frontend
│
├── src-tauri/                       ← Backend Rust (ZONA SEGURA — secretos aquí)
│   ├── src/
│   │   ├── main.rs                  ← Entry Tauri, registro de comandos IPC
│   │   ├── lib.rs                   ← Módulo raíz, re-exports
│   │   ├── error.rs                 ← AppError unificado (thiserror)
│   │   │
│   │   ├── vault/                   ← Gestión de secretos
│   │   │   ├── mod.rs
│   │   │   ├── keyring.rs           ← Capa primaria: OS keychain
│   │   │   ├── sqlite_vault.rs      ← Capa fallback: SQLite + AES-256-GCM
│   │   │   └── kdf.rs               ← Argon2id para derivación de llaves
│   │   │
│   │   ├── proxy/                   ← Servidor HTTP proxy
│   │   │   ├── mod.rs
│   │   │   ├── server.rs            ← Axum server (port 12434)
│   │   │   ├── handler.rs           ← Validación vkey + dispatch al router
│   │   │   ├── streaming.rs         ← SSE streaming chunks al cliente
│   │   │   └── openai_schema.rs     ← Tipos compatibles OpenAI API
│   │   │
│   │   ├── router/                  ← Motor de enrutamiento semántico
│   │   │   ├── mod.rs
│   │   │   ├── semantic.rs          ← Llamada a Ollama para clasificación
│   │   │   ├── scoring.rs           ← Optimizador multi-objetivo (τ-param)
│   │   │   └── policy.rs            ← Validación de presupuesto y salud
│   │   │
│   │   ├── providers/               ← Adaptadores por proveedor
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs            ← Trait LlmProvider (ver §6)
│   │   │   ├── openai.rs
│   │   │   ├── anthropic.rs
│   │   │   ├── gemini.rs
│   │   │   ├── deepseek.rs
│   │   │   └── ollama.rs            ← Proveedor local + instancia router
│   │   │
│   │   ├── db/                      ← Persistencia SQLite
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs            ← Migraciones DDL (ver §5)
│   │   │   ├── requests.rs          ← DAO logs de peticiones
│   │   │   ├── budgets.rs           ← DAO presupuestos
│   │   │   └── virtual_keys.rs      ← DAO llaves virtuales
│   │   │
│   │   ├── budget/                  ← Motor de presupuestos
│   │   │   ├── mod.rs
│   │   │   ├── tracker.rs           ← Acumulación de costos en tiempo real
│   │   │   └── alerts.rs            ← Motor de alertas 60%/85%/100%
│   │   │
│   │   └── telemetry/               ← Métricas y observabilidad
│   │       ├── mod.rs
│   │       └── metrics.rs           ← Contadores: tokens, latencia, errores
│   │
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
│
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

---

## 4. Esquemas SQLite

Base de datos ubicada en: `{app_data_dir}/ia-middleware/middleware.db`
La bóveda de secretos es una base separada: `{app_data_dir}/ia-middleware/vault.db` (AES-256-GCM)

```sql
-- === TABLA: requests (log inmutable de peticiones) ===
CREATE TABLE IF NOT EXISTS requests (
    id                TEXT PRIMARY KEY,       -- UUID v4
    created_at        INTEGER NOT NULL,       -- Unix timestamp ms
    virtual_key_id    TEXT NOT NULL,          -- FK a virtual_keys.id
    project_tag       TEXT NOT NULL,          -- Etiqueta para cost tracking
    provider          TEXT NOT NULL,          -- openai | anthropic | gemini | deepseek | ollama
    model             TEXT NOT NULL,          -- Ej: gpt-4o, claude-opus-4-7
    prompt_tokens     INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens      INTEGER NOT NULL,
    cost_usd          REAL NOT NULL DEFAULT 0.0,
    latency_ms        INTEGER NOT NULL,
    status            TEXT NOT NULL,          -- success | error | fallback | timeout
    router_decision   TEXT,                   -- JSON: {complexity, route_reason, tau_used}
    error_code        TEXT,
    FOREIGN KEY (virtual_key_id) REFERENCES virtual_keys(id)
);
CREATE INDEX idx_requests_tag_time ON requests(project_tag, created_at DESC);
CREATE INDEX idx_requests_provider ON requests(provider, created_at DESC);

-- === TABLA: budgets (políticas de presupuesto por proyecto) ===
CREATE TABLE IF NOT EXISTS budgets (
    id                TEXT PRIMARY KEY,
    project_tag       TEXT NOT NULL UNIQUE,
    limit_usd         REAL NOT NULL,
    period            TEXT NOT NULL CHECK(period IN ('daily','weekly','monthly')),
    period_start_at   INTEGER NOT NULL,       -- Inicio del período actual
    spent_usd         REAL NOT NULL DEFAULT 0.0,
    alert_60_sent_at  INTEGER,               -- Null = no enviada
    alert_85_sent_at  INTEGER,
    hard_locked_at    INTEGER,               -- Null = no bloqueado
    notify_email      TEXT,
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL
);

-- === TABLA: virtual_keys (tokens internos para clientes) ===
CREATE TABLE IF NOT EXISTS virtual_keys (
    id                TEXT PRIMARY KEY,
    key_hash          TEXT NOT NULL UNIQUE,  -- Argon2id hash del token — NUNCA plaintext
    name              TEXT NOT NULL,
    project_tag       TEXT NOT NULL,
    allowed_models    TEXT,                  -- JSON array o NULL (= todos)
    allowed_providers TEXT,                  -- JSON array o NULL (= todos)
    rate_limit_rpm    INTEGER,               -- Requests por minuto, NULL = sin límite
    expires_at        INTEGER,               -- NULL = no expira
    created_at        INTEGER NOT NULL,
    revoked_at        INTEGER                -- NULL = activa
);

-- === TABLA: providers (configuración de proveedores) ===
-- NOTA: Las API keys NO se almacenan aquí — van a la bóveda (vault)
CREATE TABLE IF NOT EXISTS providers (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL UNIQUE,  -- Ej: openai, anthropic
    display_name      TEXT NOT NULL,
    enabled           INTEGER NOT NULL DEFAULT 1,
    priority          INTEGER NOT NULL DEFAULT 0,  -- Mayor = preferido
    base_url          TEXT NOT NULL,         -- URL del endpoint
    health_status     TEXT DEFAULT 'unknown' CHECK(health_status IN ('healthy','degraded','down','unknown')),
    last_health_check INTEGER,
    model_pricing     TEXT NOT NULL,         -- JSON: {"gpt-4o": {"input": 0.005, "output": 0.015}}
    created_at        INTEGER NOT NULL
);

-- === TABLA: semantic_cache (respuestas cacheadas) ===
CREATE TABLE IF NOT EXISTS semantic_cache (
    id            TEXT PRIMARY KEY,
    prompt_hash   TEXT NOT NULL UNIQUE,  -- SHA-256(prompt_sanitizado + modelo + temp)
    response_blob BLOB NOT NULL,         -- Encriptado AES-256-GCM
    nonce         BLOB NOT NULL,         -- 12 bytes nonce GCM
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    expires_at    INTEGER NOT NULL,
    hit_count     INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_cache_expiry ON semantic_cache(expires_at);

-- === TABLA: routing_config (configuración del router) ===
CREATE TABLE IF NOT EXISTS routing_config (
    id                TEXT PRIMARY KEY DEFAULT 'singleton',
    tau               REAL NOT NULL DEFAULT 0.5,  -- 0=calidad, 1=costo
    ollama_router_url TEXT NOT NULL DEFAULT 'http://localhost:11434',
    router_model      TEXT NOT NULL DEFAULT 'qwen2.5:3b',
    classifier_model  TEXT NOT NULL DEFAULT 'qwen2.5:1.5b',
    complexity_threshold_low   REAL NOT NULL DEFAULT 0.3,
    complexity_threshold_high  REAL NOT NULL DEFAULT 0.7,
    fallback_to_local INTEGER NOT NULL DEFAULT 1,
    updated_at        INTEGER NOT NULL
);
```

---

## 5. Arquitectura de Traits Rust

```
┌──────────────────────────────────────────────────────────────┐
│                     TRAITS CORE                              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  trait LlmProvider: Send + Sync {                           │
│    fn name(&self) -> &str                                    │
│    fn supported_models(&self) -> Vec<String>                 │
│    async fn chat(&self, req, key) -> Result<Response>        │
│    async fn chat_stream(&self, req, key, tx) -> Result<()>   │
│    async fn health_check(&self) -> HealthStatus              │
│    fn calculate_cost(&self, model, tokens) -> f64            │
│  }                                                           │
│                                                              │
│  trait VaultBackend: Send + Sync {                          │
│    fn store(&self, svc, account, secret) -> Result<()>       │
│    fn retrieve(&self, svc, account) -> Result<Zeroizing>     │
│    fn delete(&self, svc, account) -> Result<()>              │
│    fn is_available(&self) -> bool                            │
│  }                                                           │
│                                                              │
│  trait RoutingEngine: Send + Sync {                         │
│    async fn decide(&self, req, ctx, constraints)             │
│              -> Result<RoutingDecision>                      │
│  }                                                           │
│                                                              │
│  trait BudgetEnforcer: Send + Sync {                        │
│    async fn check_and_reserve(&self, tag, est_cost)          │
│              -> Result<BudgetGrant>                          │
│    async fn commit(&self, grant, actual_cost) -> Result<()>  │
│    async fn rollback(&self, grant) -> Result<()>             │
│  }                                                           │
└──────────────────────────────────────────────────────────────┘

Implementaciones:
  LlmProvider:    OpenAiProvider, AnthropicProvider,
                  GeminiProvider, DeepSeekProvider, OllamaProvider

  VaultBackend:   KeyringVault (primaria, OS keychain)
                  SqliteAesVault (fallback, AES-256-GCM)

  RoutingEngine:  OllamaSemanticRouter (qwen2.5:3b via Ollama API)
                  StaticRuleRouter (fallback si Ollama no disponible)

  BudgetEnforcer: SqliteBudgetEnforcer (con SQLite WAL mode)
```

---

## 6. Diagrama del Router Semántico

```
  CLIENT (OpenAI-compatible) ──HTTP POST /v1/chat/completions──▶
                                                                 │
  ╔══════════════════════════════════════════════════════════════▼═╗
  ║              TAURI RUST BACKEND — localhost:12434              ║
  ║                                                                ║
  ║  [1] PROXY HANDLER (Axum)                                     ║
  ║      ├── Validar virtual-key (Argon2id hash lookup)            ║
  ║      ├── Verificar revocación y expiración                     ║
  ║      ├── Check presupuesto disponible (BudgetEnforcer)         ║
  ║      └── Extraer: messages[], model_hint, project_tag          ║
  ║                          │                                     ║
  ║  [2] ROUTING ENGINE      ▼                                     ║
  ║   ┌────────────────────────────────────────────────────────┐   ║
  ║   │  OllamaSemanticRouter → qwen2.5:3b (port 11434)        │   ║
  ║   │                                                         │   ║
  ║   │  Inputs:                                                │   ║
  ║   │   • Últimos N mensajes del historial                    │   ║
  ║   │   • Estimación de complejidad (token count, keywords)   │   ║
  ║   │   • RoutingConstraints {τ, budget_remaining, latency}   │   ║
  ║   │                                                         │   ║
  ║   │  Tool Call → classify_request():                        │   ║
  ║   │   {                                                      │   ║
  ║   │     complexity: "low" | "medium" | "high",              │   ║
  ║   │     task_type: "chat" | "code" | "reasoning" | "math",  │   ║
  ║   │     requires_context_window: bool,                       │   ║
  ║   │     suggested_tier: "local" | "mid" | "premium"         │   ║
  ║   │   }                                                      │   ║
  ║   └────────────────────────────────────────────────────────┘   ║
  ║                          │                                     ║
  ║  [3] MULTI-OBJECTIVE SCORER                                   ║
  ║   Score(route) = (1-τ) × quality_score(m)                     ║
  ║                - τ × normalized_cost(m)                        ║
  ║                - penalty_if_budget_warning                     ║
  ║                - penalty_if_provider_degraded                  ║
  ║                                                                ║
  ║   τ=0.0 → maximizar calidad (usa modelos premium)             ║
  ║   τ=1.0 → minimizar costo (usa local/barato)                  ║
  ║   τ=0.5 → balance óptimo (default)                            ║
  ║                                                                ║
  ║  [4] DISPATCH                                                  ║
  ║   ┌─────────────┐  ┌───────────────┐  ┌──────────────────┐   ║
  ║   │ OLLAMA LOCAL│  │ OPENAI/CLAUDE │  │ GEMINI/DEEPSEEK  │   ║
  ║   │  (τ alto)   │  │   (τ bajo)    │  │  (τ medio)       │   ║
  ║   │ qwen/llama  │  │ API + vault   │  │ API + vault      │   ║
  ║   │ port 11434  │  │   key segura  │  │   key segura     │   ║
  ║   └─────────────┘  └───────────────┘  └──────────────────┘   ║
  ║          │                  │                   │              ║
  ║  [5] TELEMETRY + BUDGET COMMIT                                 ║
  ║      ├── Registrar: tokens, costo, latencia, ruta elegida      ║
  ║      ├── Actualizar presupuesto (SqliteBudgetEnforcer)         ║
  ║      ├── Evaluar alertas 60%/85%/100%                          ║
  ║      └── Emitir IPC events → React Dashboard                   ║
  ╚══════════════════════════════════════════════════════════════╝
                          │
  ╔═════════════════════════▼══════════════════════════════════════╗
  ║         REACT WEBVIEW — UI PASIVA (sin secretos)              ║
  ║                                                               ║
  ║  IPC Events recibidos:    IPC Commands que puede invocar:     ║
  ║  • metrics_update         • get_metrics_summary               ║
  ║  • budget_alert           • get_request_log                   ║
  ║  • provider_health        • configure_budget                  ║
  ║  • vault_locked           • create_virtual_key                ║
  ║                           • store_api_key (→ vault, sin eco)  ║
  ║                           • set_routing_tau                   ║
  ╚═══════════════════════════════════════════════════════════════╝
```

---

## 7. Superficie IPC Tauri (Commands y Events)

### Comandos (Frontend → Backend)
```typescript
// Bóveda
invoke('unlock_vault', { master_password: string })
invoke('lock_vault')
invoke('store_api_key', { provider: string, api_key: string })  // sin retorno del key

// Proxy
invoke('start_proxy_server')
invoke('stop_proxy_server')
invoke('get_proxy_status') → { running: bool, port: number, requests_total: number }

// Métricas
invoke('get_metrics_summary', { period: 'hour'|'day'|'week'|'month' })
  → { total_cost: number, total_tokens: number, request_count: number, by_provider: [...] }
invoke('get_request_log', { limit: number, offset: number, filter?: RequestFilter })
  → RequestEntry[]

// Presupuestos
invoke('configure_budget', { project_tag, limit_usd, period })
invoke('get_budget_status', { project_tag })
  → { spent: number, limit: number, pct: number, locked: bool }

// Llaves virtuales
invoke('create_virtual_key', { name, project_tag, allowed_models?, expires_at? })
  → { id: string, key: string }  // key mostrada UNA vez — nunca almacenada en plaintext
invoke('list_virtual_keys') → VirtualKeyInfo[]  // sin key plaintext
invoke('revoke_virtual_key', { id: string })

// Router
invoke('set_routing_tau', { tau: number })  // 0.0 a 1.0
invoke('get_routing_config') → RoutingConfig

// Proveedores
invoke('list_providers') → ProviderInfo[]
invoke('set_provider_enabled', { provider: string, enabled: bool })
```

### Eventos (Backend → Frontend, push)
```typescript
listen('metrics_update', (e: MetricsUpdate) => ...)      // cada 5 segundos
listen('request_logged', (e: RequestEntry) => ...)        // en tiempo real
listen('budget_alert', (e: BudgetAlert) => ...)           // al 60%/85%/100%
listen('provider_health_changed', (e: HealthChange) => ...)
listen('vault_locked', () => ...)                         // bloqueo por inactividad
listen('proxy_started', (e: { port: number }) => ...)
listen('proxy_stopped', () => ...)
```

---

## 8. Modelo de Seguridad

### Capas de defensa
```
Capa 1: VAULT — secretos aislados del WebView
  • keyring OS (primario) — material protegido por hardware/kernel
  • SQLite AES-256-GCM (fallback) — encriptación a nivel de archivo
  • Argon2id KDF: m=65536KB, t=3, p=4 (parámetros mínimos OWASP)
  • Memoria: SecretString/Zeroizing<T> — limpieza en drop

Capa 2: IPC — nunca secretos en payloads
  • Los comandos store_api_key y unlock_vault no retornan el secreto
  • Frontend solo recibe estados booleanos y métricas agregadas

Capa 3: PROXY — validación en cada petición
  • Virtual key validada vía Argon2id hash lookup
  • Rate limiting por proyecto antes de tocar la bóveda

Capa 4: SESIÓN — bloqueo automático
  • Windows: WM_WTSSESSION_CHANGE (session lock/unlock)
  • Timeout de inactividad configurable (default: 15 min)
  • Al bloquear: vault se cierra, proxy rechaza nuevas peticiones

Capa 5: STREAMING — protección OOM
  • Respuestas LLM: SSE chunks, no buffer completo en memoria
  • Encriptación de caché: nonces rolling, chunks de 1MB máximo
```

---

## 9. Plan de Implementación por Fases

### Fase A — Scaffolding y Vault (Prerequisito: instalar Rust + Ollama)
1. `npm create tauri-app@latest` → React + TypeScript + Vite
2. Módulo `vault/` — keyring.rs + sqlite_vault.rs + kdf.rs
3. Comandos IPC: `unlock_vault`, `lock_vault`, `store_api_key`
4. UI mínima: VaultUnlock.tsx
5. Tests: `cargo test` en vault

### Fase B — Proxy HTTP y Proveedores
1. Módulo `proxy/` — servidor Axum en port 12434
2. Módulo `providers/` — trait + OpenAI + Anthropic
3. Endpoint `/v1/chat/completions` funcionando
4. Tests: curl manual contra el proxy

### Fase C — Router Semántico
1. Módulo `router/` — OllamaSemanticRouter
2. Módulo `router/scoring.rs` — optimizador τ
3. Integración con Fase B (dispatch por router)
4. Agregar proveedores Gemini, DeepSeek, Ollama-local

### Fase D — Presupuestos y Telemetría
1. Módulos `budget/` y `telemetry/`
2. Alertas 60%/85%/100% con IPC events
3. Virtual keys con Argon2id hash

### Fase E — Dashboard React
1. Componentes de métricas (Recharts)
2. Configuración de proveedores y presupuestos
3. Feed en tiempo real de peticiones
4. Integración completa de eventos IPC

---

## 10. Dependencias Rust Clave (Cargo.toml preliminar)

```toml
[dependencies]
# Tauri core
tauri = { version = "2", features = ["protocol-asset"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP server (proxy)
axum = { version = "0.7", features = ["tokio"] }

# HTTP client (providers) — rustls para no inflar con openssl
reqwest = { version = "0.12", default-features = false,
            features = ["rustls-tls", "json", "stream"] }

# Serialización
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Base de datos
rusqlite = { version = "0.31", features = ["bundled", "column_metadata"] }

# Criptografía
aes-gcm = "0.10"          # AES-256-GCM
argon2 = "0.5"             # Argon2id KDF
keyring = "2"              # OS keychain (Windows/macOS/Linux)
secrecy = "0.8"            # SecretString + Zeroizing
rand = "0.8"               # Nonce generation

# Errores
thiserror = "1"
anyhow = "1"

# Utilidades
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
```

---

*Este documento está sujeto a revisión y aprobación antes de iniciar implementación.*
