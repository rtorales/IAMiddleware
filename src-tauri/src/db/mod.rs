pub mod schema;

use rusqlite::Connection;
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
use crate::error::{AppError, AppResult};

pub fn open_db() -> AppResult<Connection> {
    let path = db_path()?;
    let conn = Connection::open(&path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    schema::run_migrations(&conn)?;
    Ok(conn)
}

fn db_path() -> AppResult<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| AppError::Config("No se pudo determinar data_dir".to_string()))?;
    let dir = base.join("ia-middleware");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("middleware.db"))
}

// ── Tipos de retorno para IPC ─────────────────────────────────────────────────

/// Ventana de tiempo en milisegundos para cada período de métricas.
fn period_ms(period: &str) -> i64 {
    match period {
        "hour"  => 3_600_000,
        "week"  => 7 * 86_400_000,
        "month" => 30 * 86_400_000,
        _       => 86_400_000, // "day" por defecto
    }
}

/// Consulta el resumen de métricas para el período indicado.
/// Retorna una estructura compatible con MetricsSummary del frontend.
pub fn query_metrics_summary(conn: &Connection, period: &str) -> AppResult<serde_json::Value> {
    let since = chrono::Utc::now().timestamp_millis() - period_ms(period);

    // Totales globales
    let (total_cost, total_tokens, req_count, success_count, avg_latency): (f64, i64, i64, i64, f64) =
        conn.query_row(
            "SELECT
               COALESCE(SUM(cost_usd),0),
               COALESCE(SUM(total_tokens),0),
               COUNT(*),
               SUM(CASE WHEN status='success' THEN 1 ELSE 0 END),
               COALESCE(AVG(latency_ms),0)
             FROM requests WHERE created_at > ?1",
            rusqlite::params![since],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .unwrap_or((0.0, 0, 0, 0, 0.0));

    let success_rate = if req_count > 0 {
        success_count as f64 / req_count as f64
    } else {
        1.0
    };

    // Desglose por proveedor
    let mut stmt = conn.prepare(
        "SELECT provider,
                COUNT(*) as reqs,
                COALESCE(SUM(cost_usd),0),
                COALESCE(SUM(total_tokens),0),
                COALESCE(AVG(latency_ms),0),
                SUM(CASE WHEN status!='success' THEN 1 ELSE 0 END)*1.0/COUNT(*) as err_rate
         FROM requests WHERE created_at > ?1
         GROUP BY provider",
    )?;

    let by_provider: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![since], |row| {
            Ok(json!({
                "provider":       row.get::<_,String>(0)?,
                "requestCount":   row.get::<_,i64>(1)?,
                "totalCostUsd":   row.get::<_,f64>(2)?,
                "totalTokens":    row.get::<_,i64>(3)?,
                "avgLatencyMs":   row.get::<_,f64>(4)?,
                "errorRate":      row.get::<_,f64>(5)?,
                "healthStatus":   "unknown",
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(json!({
        "totalCostUsd":  total_cost,
        "totalTokens":   total_tokens,
        "requestCount":  req_count,
        "successRate":   success_rate,
        "avgLatencyMs":  avg_latency,
        "byProvider":    by_provider,
        "period":        period,
    }))
}

/// Consulta las últimas N peticiones para el feed del frontend.
pub fn query_recent_requests(conn: &Connection, limit: u32) -> AppResult<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        "SELECT id, created_at, project_tag, provider, model,
                prompt_tokens, completion_tokens, cost_usd, latency_ms, status
         FROM requests
         ORDER BY created_at DESC
         LIMIT ?1",
    )?;

    let rows: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![limit], |row| {
            Ok(json!({
                "id":               row.get::<_,String>(0)?,
                "createdAt":        row.get::<_,i64>(1)?,
                "projectTag":       row.get::<_,String>(2)?,
                "provider":         row.get::<_,String>(3)?,
                "model":            row.get::<_,String>(4)?,
                "promptTokens":     row.get::<_,i64>(5)?,
                "completionTokens": row.get::<_,i64>(6)?,
                "costUsd":          row.get::<_,f64>(7)?,
                "latencyMs":        row.get::<_,i64>(8)?,
                "status":           row.get::<_,String>(9)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Consulta todos los budgets registrados.
pub fn query_budgets(conn: &Connection) -> AppResult<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        "SELECT project_tag, spent_usd, limit_usd, period,
                hard_locked_at, alert_60_sent_at, alert_85_sent_at
         FROM budgets",
    )?;

    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let locked: Option<i64> = row.get(4)?;
            let alert60: Option<i64> = row.get(5)?;
            let alert85: Option<i64> = row.get(6)?;
            let spent: f64 = row.get(1)?;
            let limit: f64 = row.get(2)?;
            let pct = if limit > 0.0 { spent / limit } else { 0.0 };

            Ok(json!({
                "projectTag":   row.get::<_,String>(0)?,
                "spentUsd":     spent,
                "limitUsd":     limit,
                "percentUsed":  pct,
                "locked":       locked.is_some(),
                "period":       row.get::<_,String>(3)?,
                "alert60Sent":  alert60.is_some(),
                "alert85Sent":  alert85.is_some(),
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Genera el payload completo para el evento `metrics_update` del frontend.
pub async fn metrics_update_payload(
    db: Arc<std::sync::Mutex<Connection>>,
) -> AppResult<serde_json::Value> {
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|_| AppError::Config("db lock poisoned".into()))?;
        let summary = query_metrics_summary(&conn, "day")?;
        let recent = query_recent_requests(&conn, 50)?;
        Ok(json!({"summary": summary, "recentRequests": recent}))
    })
    .await
    .map_err(|e| AppError::Config(format!("db task: {e}")))?
}
