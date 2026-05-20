use reqwest::Client;
use rusqlite::Connection;
use serde_json::json;

use crate::vault::Vault;

// ── Configuración ─────────────────────────────────────────────────────────────

pub struct Config {
    pub tau: f64,
    pub ollama_url: String,
    pub classifier_model: String,
    pub router_model: String,
    pub threshold_low: f64,
    pub threshold_high: f64,
}

pub fn load_config(conn: &Connection) -> Config {
    conn.query_row(
        "SELECT tau, ollama_router_url, classifier_model, router_model,
                complexity_threshold_low, complexity_threshold_high
         FROM routing_config WHERE id = 'singleton'",
        [],
        |row| {
            Ok(Config {
                tau:              row.get(0)?,
                ollama_url:       row.get(1)?,
                classifier_model: row.get(2)?,
                router_model:     row.get(3)?,
                threshold_low:    row.get(4)?,
                threshold_high:   row.get(5)?,
            })
        },
    )
    .unwrap_or_else(|_| Config {
        tau:              0.5,
        ollama_url:       "http://localhost:11434".to_string(),
        classifier_model: "qwen2.5:1.5b".to_string(),
        router_model:     "qwen2.5:3b".to_string(),
        threshold_low:    0.3,
        threshold_high:   0.7,
    })
}

// ── Decisión de routing ───────────────────────────────────────────────────────

pub struct Decision {
    pub provider: &'static str,
    pub model: String,
    pub complexity: f64,
    pub reason: String,
}

pub async fn route(
    prompt: &str,
    config: &Config,
    vault: &Vault,
    client: &Client,
) -> Decision {
    let score = classify(prompt, config, client).await.unwrap_or_else(|| {
        tracing::warn!("[router] clasificación no disponible — usando tau={:.2}", config.tau);
        config.tau
    });

    tracing::info!(
        "[router] score={:.3} low={:.2} high={:.2} tau={:.2}",
        score, config.threshold_low, config.threshold_high, config.tau
    );

    if score <= config.threshold_low {
        Decision {
            provider: "ollama",
            model: config.router_model.clone(),
            complexity: score,
            reason: format!("local: {score:.2} ≤ low {:.2}", config.threshold_low),
        }
    } else if score >= config.threshold_high || score >= config.tau {
        let (provider, model) = pick_cloud(vault);
        Decision {
            provider,
            model,
            complexity: score,
            reason: format!("cloud: {score:.2} ≥ high {:.2}", config.threshold_high),
        }
    } else if score >= config.tau {
        let (provider, model) = pick_cloud(vault);
        Decision {
            provider,
            model,
            complexity: score,
            reason: format!("cloud: {score:.2} ≥ τ {:.2}", config.tau),
        }
    } else {
        Decision {
            provider: "ollama",
            model: config.router_model.clone(),
            complexity: score,
            reason: format!("local: {score:.2} < τ {:.2}", config.tau),
        }
    }
}

// ── Clasificación via Ollama ──────────────────────────────────────────────────

async fn classify(prompt: &str, config: &Config, client: &Client) -> Option<f64> {
    let body = json!({
        "model": config.classifier_model,
        "prompt": format!(
            "Rate the complexity of this task from 0.0 to 1.0.\n\
             0.0 = simple greeting, basic fact, yes/no question\n\
             1.0 = complex reasoning, code generation, multi-step analysis\n\
             Reply with ONLY a decimal number. No explanation.\n\n\
             Task: {prompt}"
        ),
        "stream": false,
        "options": { "temperature": 0.0, "num_predict": 8 }
    });

    let resp = client
        .post(format!("{}/api/generate", config.ollama_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let v: serde_json::Value = resp.json().await.ok()?;
    parse_score(v["response"].as_str()?)
}

fn parse_score(text: &str) -> Option<f64> {
    text.split_whitespace().find_map(|word| {
        let clean: String = word
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        clean
            .parse::<f64>()
            .ok()
            .filter(|&f| (0.0..=1.0).contains(&f))
    })
}

// ── Selección de proveedor cloud ──────────────────────────────────────────────

fn pick_cloud(vault: &Vault) -> (&'static str, String) {
    const PREFERENCE: &[(&str, &str)] = &[
        ("anthropic", "claude-haiku-4-5-20251001"),
        ("openai",    "gpt-4o-mini"),
        ("deepseek",  "deepseek-chat"),
        ("gemini",    "gemini-2.0-flash"),
    ];

    for &(provider, model) in PREFERENCE {
        if vault.has_api_key(provider) {
            return (provider, model.to_string());
        }
    }

    // Sin keys cloud configuradas → fallback local
    ("ollama", "qwen2.5:3b".to_string())
}
