use rusqlite::Connection;

/// Precios de fallback (USD por 1K tokens) cuando no hay datos en la tabla providers.
fn fallback(provider: &str, is_output: bool) -> f64 {
    match (provider, is_output) {
        ("openai",    false) => 0.005,
        ("openai",    true)  => 0.015,
        ("anthropic", false) => 0.003,
        ("anthropic", true)  => 0.015,
        ("deepseek",  false) => 0.00027,
        ("deepseek",  true)  => 0.0011,
        ("gemini",    false) => 0.0001,
        ("gemini",    true)  => 0.0004,
        _                    => 0.0,   // ollama y desconocidos: gratis
    }
}

/// Lee (input_per_k, output_per_k) de la tabla providers para el modelo dado.
/// Si el modelo exacto no existe en el JSON, usa el primer precio disponible del proveedor.
fn read_from_db(provider: &str, model: &str, conn: &Connection) -> Option<(f64, f64)> {
    let json_str: String = conn
        .query_row(
            "SELECT model_pricing FROM providers WHERE name = ?1",
            rusqlite::params![provider],
            |row| row.get(0),
        )
        .ok()?;

    let map: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    let entry = map.get(model)
        .or_else(|| map.as_object().and_then(|m| m.values().next()))?;

    Some((
        entry.get("input")?.as_f64()?,
        entry.get("output")?.as_f64()?,
    ))
}

/// Calcula el costo estimado en USD para una petición.
pub fn estimate_cost(
    provider: &str,
    model: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
    conn: &Connection,
) -> f64 {
    let (input_k, output_k) = read_from_db(provider, model, conn)
        .unwrap_or_else(|| (fallback(provider, false), fallback(provider, true)));

    (prompt_tokens as f64 / 1000.0) * input_k
        + (completion_tokens as f64 / 1000.0) * output_k
}
