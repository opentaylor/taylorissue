use serde_json::Value;
use tauri::ipc::Channel;
use thiserror::Error;

use crate::config::AppConfig;
use crate::services::agent_registry::{self, AgentEntry};
use crate::services::setup_detection;
use crate::services::shell_env;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Agent not found: {0}")]
    NotFound(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

static ENDPOINT_ENSURED: AtomicBool = AtomicBool::new(false);

async fn ensure_http_endpoint(config: &AppConfig) {
    if ENDPOINT_ENSURED.load(Ordering::Relaxed) {
        return;
    }
    if setup_detection::check_http_endpoint_enabled() == Some(true) {
        ENDPOINT_ENSURED.store(true, Ordering::Relaxed);
        return;
    }
    log::info!("[ensure_http_endpoint] enabling chatCompletions HTTP endpoint");
    if setup_detection::enable_http_endpoint() {
        let bin = &config.openclaw_bin;
        let bin = if bin.is_empty() { "openclaw" } else { bin.as_str() };
        log::info!("[ensure_http_endpoint] restarting gateway via {}", bin);
        let result = tokio::task::spawn_blocking({
            let bin = bin.to_string();
            move || {
                let mut cmd = std::process::Command::new(&bin);
                cmd.args(["gateway", "restart"]);
                shell_env::apply_env(&mut cmd);
                cmd.output()
            }
        })
        .await;
        match result {
            Ok(Ok(output)) => {
                log::info!(
                    "[ensure_http_endpoint] gateway restart exit={}",
                    output.status
                );
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Ok(Err(e)) => log::warn!("[ensure_http_endpoint] failed to run restart: {}", e),
            Err(e) => log::warn!("[ensure_http_endpoint] spawn error: {}", e),
        }
    }
    ENDPOINT_ENSURED.store(true, Ordering::Relaxed);
}

pub fn list_registered_agents(config: &AppConfig) -> Result<Vec<AgentEntry>, MessageError> {
    let workspace = config.resolve_workspace_path();
    Ok(agent_registry::load_registry(
        Some(&workspace),
        Some(&config.openclaw_bin),
    ))
}

fn get_agent(agent_id: &str, config: &AppConfig) -> Option<AgentEntry> {
    let workspace = config.resolve_workspace_path();
    let agents = agent_registry::load_registry(
        Some(&workspace),
        Some(&config.openclaw_bin),
    );
    agents.into_iter().find(|a| a.id == agent_id)
}

pub async fn stream_chat(
    agent_id: &str,
    messages: Vec<(String, String)>,
    operator_name: &str,
    config: &AppConfig,
    channel: Channel<Value>,
) -> Result<(), MessageError> {
    let workspace = config.resolve_workspace_path();
    let agent = get_agent(agent_id, config).ok_or_else(|| MessageError::NotFound(agent_id.to_string()))?;

    let gateway_token = &config.gateway_token;
    let gateway_url = config.gateway_url();

    let soul: Option<String> = agent.soul_path.as_ref().and_then(|sp| {
        let soul_file = std::path::PathBuf::from(&workspace).join(sp);
        std::fs::read_to_string(&soul_file).ok()
    });

    let system_prompt = if let Some(ref soul) = soul {
        format!(
            "{}\n\nYou are speaking directly with {}, your operator. \
             Stay fully in character. Be concise — this is a live chat. \
             2-4 sentences unless detail is asked for. No em dashes.",
            soul, operator_name
        )
    } else {
        format!(
            "You are {}, {}. Respond in character. Be concise. No em dashes.",
            agent.name, agent.title
        )
    };

    let mut openai_messages = vec![serde_json::json!({"role": "system", "content": system_prompt})];
    for (role, content) in &messages {
        if role != "system" {
            openai_messages.push(serde_json::json!({"role": role, "content": content}));
        }
    }

    let model = agent.model.unwrap_or_else(|| config.model.clone());

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "model": model,
        "stream": true,
        "messages": openai_messages,
    });

    ensure_http_endpoint(config).await;

    let url = format!("{}/chat/completions", gateway_url);
    log::info!("[stream_chat] POST {} model={} token_len={}", url, model, gateway_token.len());

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", gateway_token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            log::info!("[stream_chat] response status={}", status);
            if status.as_u16() == 404 || status.as_u16() == 405 {
                let _ = channel.send(serde_json::json!({
                    "error": format!(
                        "Gateway returned {}. The Chat Completions HTTP endpoint is disabled by default. \
                         Enable it: set gateway.http.endpoints.chatCompletions.enabled = true \
                         in ~/.openclaw/openclaw.json, then run: openclaw gateway restart",
                        status.as_u16()
                    )
                }));
                let _ = channel.send(serde_json::json!({"done": true}));
                return Ok(());
            }

            use futures::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        log::warn!("Stream chunk error: {}", e);
                        break;
                    }
                };
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() { continue; }
                    if !line.starts_with("data: ") { continue; }

                    let data_str = &line[6..];
                    if data_str == "[DONE]" {
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<Value>(data_str) {
                        let content = parsed
                            .get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c| c.get("delta"))
                            .and_then(|d| d.get("content"))
                            .and_then(|c| c.as_str());
                        if let Some(text) = content {
                            let _ = channel.send(serde_json::json!({"content": text}));
                        }
                    }
                }
            }
            let _ = channel.send(serde_json::json!({"done": true}));
        }
        Err(e) => {
            log::error!("[stream_chat] request failed: {}", e);
            let _ = channel.send(serde_json::json!({"error": e.to_string()}));
            let _ = channel.send(serde_json::json!({"done": true}));
        }
    }

    Ok(())
}
