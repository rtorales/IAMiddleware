use clap::Parser;
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

// ── ANSI ──────────────────────────────────────────────────────────────────────
const R: &str = "\x1b[0m";
const B: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";

// ── Args ──────────────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(
    name = "ia-cli",
    version = "0.1",
    about = "IA Middleware CLI — chat desde la terminal"
)]
struct Args {
    /// Mensaje directo (modo one-shot). Sin argumento → modo interactivo.
    message: Option<String>,

    /// Modelo: auto, gpt-4o, claude-sonnet-4-6, qwen2.5:3b, etc.
    #[arg(long, short, default_value = "auto")]
    model: String,

    /// System prompt inicial
    #[arg(long, short)]
    system: Option<String>,

    /// URL base del proxy
    #[arg(long, default_value = "http://localhost:12434/v1")]
    url: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    enable_ansi_windows();

    let args = Args::parse();
    let client = Client::new();

    if let Some(msg) = args.message {
        // One-shot
        let mut msgs: Vec<Value> = Vec::new();
        if let Some(sys) = &args.system {
            msgs.push(json!({"role": "system", "content": sys}));
        }
        msgs.push(json!({"role": "user", "content": msg}));

        match stream_chat(&client, &args.url, &args.model, &msgs).await {
            Ok(_) => println!(),
            Err(e) => eprintln!("\n{RED}Error: {e}{R}"),
        }
    } else {
        interactive(&client, &args.url, args.model, args.system).await;
    }
}

// ── Modo interactivo ──────────────────────────────────────────────────────────
async fn interactive(client: &Client, url: &str, initial_model: String, system: Option<String>) {
    let mut model = initial_model;
    let mut messages: Vec<Value> = Vec::new();

    if let Some(sys) = &system {
        messages.push(json!({"role": "system", "content": sys}));
    }

    print_header(&model, url);

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        print!("{GREEN}{B}>{R} ");
        io::stdout().flush().ok();

        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => {
                println!("{DIM}\n  Hasta luego.{R}");
                break;
            }
            Ok(_) => {}
        }

        let input = line.trim().to_string();
        if input.is_empty() {
            continue;
        }

        // ── Comandos ─────────────────────────────────────────────────────────
        match input.as_str() {
            "/exit" | "/quit" => {
                println!("{DIM}  Hasta luego.{R}");
                break;
            }
            "/clear" => {
                messages.retain(|m| m["role"] == "system");
                println!("{DIM}  Historial limpiado.{R}");
                continue;
            }
            "/help" => {
                print_help();
                continue;
            }
            "/models" => {
                list_models(client, url).await;
                continue;
            }
            _ => {}
        }

        if let Some(m) = input.strip_prefix("/model ") {
            model = m.trim().to_string();
            println!("{DIM}  Modelo: {CYAN}{model}{R}");
            continue;
        }

        if let Some(s) = input.strip_prefix("/system ") {
            messages.retain(|m| m["role"] != "system");
            messages.insert(0, json!({"role": "system", "content": s.trim()}));
            println!("{DIM}  System prompt actualizado.{R}");
            continue;
        }

        if input.starts_with('/') {
            println!("{DIM}  Comando desconocido. Escribe /help.{R}");
            continue;
        }

        // ── Petición al proxy ─────────────────────────────────────────────────
        messages.push(json!({"role": "user", "content": input}));
        print!("\n{CYAN}");
        io::stdout().flush().ok();

        match stream_chat(client, url, &model, &messages).await {
            Ok(response) => {
                print!("{R}");
                println!();
                messages.push(json!({"role": "assistant", "content": response}));
            }
            Err(e) => {
                print!("{R}");
                println!("{RED}  Error: {e}{R}");
                println!("{DIM}  ¿Está corriendo la app IA Middleware?{R}");
                messages.pop();
            }
        }
    }
}

// ── Streaming SSE ─────────────────────────────────────────────────────────────
async fn stream_chat(
    client: &Client,
    url: &str,
    model: &str,
    messages: &[Value],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let body = json!({
        "model": model,
        "messages": messages,
        "stream": true
    });

    let resp = client
        .post(format!("{url}/chat/completions"))
        .header("Authorization", "Bearer no-key")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("No se pudo conectar al proxy: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        // Try to extract a clean error message from JSON
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["error"]["message"].as_str().map(String::from))
            .unwrap_or(text);
        return Err(format!("HTTP {status}: {msg}").into());
    }

    let mut accumulated = String::new();
    let mut buf = String::new();
    let mut byte_stream = resp.bytes_stream();

    while let Some(chunk) = byte_stream.next().await {
        let bytes = chunk?;
        buf.push_str(&String::from_utf8_lossy(&bytes));

        loop {
            match buf.find('\n') {
                None => break,
                Some(pos) => {
                    let line = buf[..pos].trim_end_matches('\r').to_string();
                    buf = buf[pos + 1..].to_string();

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data.trim() == "[DONE]" {
                            break;
                        }
                        if let Ok(v) = serde_json::from_str::<Value>(data) {
                            if let Some(content) = v["choices"][0]["delta"]["content"].as_str() {
                                print!("{content}");
                                io::stdout().flush().ok();
                                accumulated.push_str(content);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(accumulated)
}

// ── Listar modelos ────────────────────────────────────────────────────────────
async fn list_models(client: &Client, url: &str) {
    match client.get(format!("{url}/models")).send().await {
        Err(e) => println!("{RED}  Error: {e}{R}"),
        Ok(resp) => match resp.json::<Value>().await {
            Err(_) => println!("{RED}  No se pudo leer la respuesta.{R}"),
            Ok(v) => {
                println!("{DIM}  Modelos disponibles:{R}");
                if let Some(data) = v["data"].as_array() {
                    for m in data {
                        let id = m["id"].as_str().unwrap_or("?");
                        let owner = m["owned_by"].as_str().unwrap_or("?");
                        println!("    {CYAN}{id:<30}{R}  {DIM}{owner}{R}");
                    }
                }
            }
        },
    }
}

// ── UI helpers ────────────────────────────────────────────────────────────────
fn print_header(model: &str, url: &str) {
    println!();
    println!("  {B}IA Middleware CLI{R}  {DIM}v0.1{R}");
    println!("  {DIM}modelo: {CYAN}{model}{R}  {DIM}│ proxy: {url}{R}");
    println!("  {DIM}{}{R}", "─".repeat(48));
    println!("  {DIM}Escribe tu mensaje. {YELLOW}/help{R}{DIM} para comandos. Ctrl+D para salir.{R}");
    println!();
}

fn print_help() {
    println!("{DIM}");
    println!("  {B}Comandos:{R}{DIM}");
    println!("    /model <nombre>    Cambia el modelo activo");
    println!("    /system <prompt>   Define el system prompt");
    println!("    /models            Lista los modelos disponibles");
    println!("    /clear             Limpia el historial de conversación");
    println!("    /help              Muestra esta ayuda");
    println!("    /exit              Sale del programa");
    println!("{R}");
}

// ── Windows: habilitar ANSI en consola ────────────────────────────────────────
fn enable_ansi_windows() {
    #[cfg(windows)]
    {
        // Ignoramos el resultado — si falla, los colores simplemente no aparecen
        let _ = std::process::Command::new("cmd")
            .args(["/c", ""])
            .status();
    }
}
