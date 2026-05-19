use rusqlite::Connection;
use crate::error::AppResult;

/// Ejecuta todas las migraciones DDL en orden idempotente.
pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(MIGRATION_V1)?;
    Ok(())
}

const MIGRATION_V1: &str = "
CREATE TABLE IF NOT EXISTS requests (
    id                TEXT PRIMARY KEY,
    created_at        INTEGER NOT NULL,
    virtual_key_id    TEXT NOT NULL,
    project_tag       TEXT NOT NULL,
    provider          TEXT NOT NULL,
    model             TEXT NOT NULL,
    prompt_tokens     INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens      INTEGER NOT NULL DEFAULT 0,
    cost_usd          REAL    NOT NULL DEFAULT 0.0,
    latency_ms        INTEGER NOT NULL DEFAULT 0,
    status            TEXT    NOT NULL DEFAULT 'success',
    router_decision   TEXT,
    error_code        TEXT
);
CREATE INDEX IF NOT EXISTS idx_requests_tag_time
    ON requests(project_tag, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_requests_provider
    ON requests(provider, created_at DESC);

CREATE TABLE IF NOT EXISTS budgets (
    id                TEXT PRIMARY KEY,
    project_tag       TEXT NOT NULL UNIQUE,
    limit_usd         REAL NOT NULL,
    period            TEXT NOT NULL CHECK(period IN ('daily','weekly','monthly')),
    period_start_at   INTEGER NOT NULL,
    spent_usd         REAL NOT NULL DEFAULT 0.0,
    alert_60_sent_at  INTEGER,
    alert_85_sent_at  INTEGER,
    hard_locked_at    INTEGER,
    notify_email      TEXT,
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS virtual_keys (
    id                TEXT PRIMARY KEY,
    key_hash          TEXT NOT NULL UNIQUE,
    name              TEXT NOT NULL,
    project_tag       TEXT NOT NULL,
    allowed_models    TEXT,
    allowed_providers TEXT,
    rate_limit_rpm    INTEGER,
    expires_at        INTEGER,
    created_at        INTEGER NOT NULL,
    revoked_at        INTEGER
);

CREATE TABLE IF NOT EXISTS providers (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL UNIQUE,
    display_name      TEXT NOT NULL,
    enabled           INTEGER NOT NULL DEFAULT 1,
    priority          INTEGER NOT NULL DEFAULT 0,
    base_url          TEXT NOT NULL,
    health_status     TEXT DEFAULT 'unknown',
    last_health_check INTEGER,
    model_pricing     TEXT NOT NULL DEFAULT '{}',
    created_at        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS semantic_cache (
    id            TEXT PRIMARY KEY,
    prompt_hash   TEXT NOT NULL UNIQUE,
    response_blob BLOB NOT NULL,
    nonce         BLOB NOT NULL,
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    expires_at    INTEGER NOT NULL,
    hit_count     INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_cache_expiry ON semantic_cache(expires_at);

CREATE TABLE IF NOT EXISTS routing_config (
    id                         TEXT PRIMARY KEY DEFAULT 'singleton',
    tau                        REAL NOT NULL DEFAULT 0.5,
    ollama_router_url          TEXT NOT NULL DEFAULT 'http://localhost:11434',
    router_model               TEXT NOT NULL DEFAULT 'qwen2.5:3b',
    classifier_model           TEXT NOT NULL DEFAULT 'qwen2.5:1.5b',
    complexity_threshold_low   REAL NOT NULL DEFAULT 0.3,
    complexity_threshold_high  REAL NOT NULL DEFAULT 0.7,
    fallback_to_local          INTEGER NOT NULL DEFAULT 1,
    updated_at                 INTEGER NOT NULL DEFAULT (unixepoch() * 1000)
);
INSERT OR IGNORE INTO routing_config (id, updated_at) VALUES ('singleton', unixepoch() * 1000);

INSERT OR IGNORE INTO providers (id, name, display_name, base_url, model_pricing, created_at)
VALUES
  ('openai',    'openai',    'OpenAI',    'https://api.openai.com/v1',                          '{\"gpt-4o\":{\"input\":0.005,\"output\":0.015}}',     unixepoch() * 1000),
  ('anthropic', 'anthropic', 'Anthropic', 'https://api.anthropic.com',                          '{\"claude-sonnet-4-6\":{\"input\":0.003,\"output\":0.015}}', unixepoch() * 1000),
  ('gemini',    'gemini',    'Google Gemini', 'https://generativelanguage.googleapis.com/v1beta', '{\"gemini-2.0-flash\":{\"input\":0.0001,\"output\":0.0004}}', unixepoch() * 1000),
  ('deepseek',  'deepseek',  'DeepSeek',  'https://api.deepseek.com/v1',                        '{\"deepseek-chat\":{\"input\":0.00027,\"output\":0.0011}}', unixepoch() * 1000),
  ('ollama',    'ollama',    'Ollama (Local)', 'http://localhost:11434',                         '{\"qwen2.5:3b\":{\"input\":0.0,\"output\":0.0}}',   unixepoch() * 1000);
";
