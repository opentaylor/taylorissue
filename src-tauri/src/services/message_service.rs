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
    if !setup_detection::enable_http_endpoint() {
        log::warn!("[ensure_http_endpoint] failed to enable endpoint in config");
        return;
    }
    let bin = if config.openclaw_bin.is_empty() {
        setup_detection::detect_openclaw_bin()
            .unwrap_or_else(|| "openclaw".to_string())
    } else {
        config.openclaw_bin.clone()
    };
    log::info!("[ensure_http_endpoint] restarting gateway via {}", bin);
    let result = tokio::task::spawn_blocking({
        move || {
            let mut cmd = shell_env::build_command(&bin);
            cmd.args(["gateway", "restart"]);
            shell_env::apply_env(&mut cmd);
            cmd.output()
        }
    })
    .await;
    match result {
        Ok(Ok(output)) if output.status.success() => {
            log::info!("[ensure_http_endpoint] gateway restart exit={}", output.status);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            ENDPOINT_ENSURED.store(true, Ordering::Relaxed);
        }
        Ok(Ok(output)) => {
            log::warn!("[ensure_http_endpoint] gateway restart failed exit={}", output.status);
        }
        Ok(Err(e)) => log::warn!("[ensure_http_endpoint] failed to run restart: {}", e),
        Err(e) => log::warn!("[ensure_http_endpoint] spawn error: {}", e),
    }
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

fn extract_error(v: &Value) -> Option<String> {
    if let Some(err) = v.get("error") {
        if let Some(msg) = err.get("message").and_then(|m| m.as_str()) {
            return Some(msg.to_string());
        }
        if let Some(s) = err.as_str() {
            return Some(s.to_string());
        }
        return Some(err.to_string());
    }
    None
}

async fn probe_gateway_error(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    model: &str,
) -> Option<String> {
    let probe = serde_json::json!({
        "model": model,
        "stream": false,
        "max_tokens": 1,
        "messages": [{"role": "user", "content": "hi"}],
    });
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&probe)
        .send()
        .await
        .ok()?;
    let body: Value = resp.json().await.ok()?;
    let total_tokens = body
        .pointer("/usage/total_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    if total_tokens == 0 {
        let content = body
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let looks_like_error = content
            .trim()
            .chars()
            .take(3)
            .all(|c| c.is_ascii_digit())
            && content.trim().len() >= 3;
        if !content.is_empty() && looks_like_error {
            log::warn!("[stream_chat] probe detected gateway error: {}", content);
            return Some(content);
        }
    }
    None
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
            let mut got_content = false;
            let mut got_error = false;

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        log::warn!("[stream_chat] chunk error: {}", e);
                        if !got_content {
                            let _ = channel.send(serde_json::json!({"error": e.to_string()}));
                            got_error = true;
                        }
                        break;
                    }
                };
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() { continue; }
                    if !line.starts_with("data: ") {
                        if !got_content && !got_error {
                            if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
                                if let Some(err) = extract_error(&parsed) {
                                    log::warn!("[stream_chat] stream error: {}", err);
                                    let _ = channel.send(serde_json::json!({"error": err}));
                                    got_error = true;
                                }
                            }
                        }
                        continue;
                    }

                    let data_str = &line[6..];
                    if data_str == "[DONE]" {
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<Value>(data_str) {
                        if let Some(err) = extract_error(&parsed) {
                            log::warn!("[stream_chat] stream error: {}", err);
                            let _ = channel.send(serde_json::json!({"error": err}));
                            got_error = true;
                            continue;
                        }
                        let content = parsed
                            .get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c| c.get("delta"))
                            .and_then(|d| d.get("content"))
                            .and_then(|c| c.as_str());
                        if let Some(text) = content {
                            got_content = true;
                            let _ = channel.send(serde_json::json!({"content": text}));
                        }
                    }
                }
            }

            if !got_content && !got_error {
                let remaining = buffer.trim().to_string();
                if !remaining.is_empty() {
                    if let Ok(parsed) = serde_json::from_str::<Value>(&remaining) {
                        if let Some(err) = extract_error(&parsed) {
                            let _ = channel.send(serde_json::json!({"error": err}));
                            got_error = true;
                        }
                    }
                }
                if !got_error {
                    if let Some(err) = probe_gateway_error(&client, &url, gateway_token, &model).await {
                        let _ = channel.send(serde_json::json!({"error": err}));
                    } else {
                        let _ = channel.send(serde_json::json!({"error": "No response received from the model. Check your API key and quota."}));
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
