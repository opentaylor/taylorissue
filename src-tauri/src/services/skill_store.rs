use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::base::make_llm;
use crate::kernel::middleware::logging::LoggingMiddleware;
use crate::kernel::tool::bash::BashTool;
use crate::services::shell_env;
use crate::prompts::render;
use crate::services::step_runner::extract_json;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Skill not found: {0}")]
    NotFound(String),
    #[error("Cannot uninstall bundled skill: {0}")]
    CannotUninstall(String),
    #[error("Skill already eligible: {0}")]
    AlreadyEligible(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub emoji: Option<String>,
    pub eligible: bool,
    pub disabled: bool,
    pub source: String,
    pub bundled: bool,
    pub homepage: Option<String>,
    pub missing: Value,
    pub install: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub ok: bool,
    pub outputs: Vec<String>,
}

fn managed_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".openclaw")
        .join("skills")
}

fn workspace_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".openclaw")
        .join("workspace")
        .join("skills")
}

fn run_cli(args: &[&str], openclaw_bin: &str) -> Value {
    let bin = if openclaw_bin.is_empty() {
        crate::services::setup_detection::detect_openclaw_bin_path()
            .unwrap_or_else(|| "openclaw".to_string())
    } else {
        openclaw_bin.to_string()
    };

    let mut cli_args: Vec<&str> = args.to_vec();
    cli_args.push("--json");

    let mut cmd = shell_env::build_command(&bin);
    cmd.args(&cli_args);
    shell_env::apply_env(&mut cmd);
    let result = cmd.output();

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str(&stdout).unwrap_or(Value::Object(serde_json::Map::new()))
        }
        _ => Value::Object(serde_json::Map::new()),
    }
}

pub async fn list_skills(openclaw_bin: &str) -> Result<Vec<SkillInfo>, SkillError> {
    shell_env::refresh_path();
    let bin = openclaw_bin.to_string();
    let list_handle = tokio::task::spawn_blocking({
        let bin = bin.clone();
        move || run_cli(&["skills", "list"], &bin)
    });
    let check_handle = tokio::task::spawn_blocking({
        let bin = bin.clone();
        move || run_cli(&["skills", "check"], &bin)
    });

    let (list_data, check_data) = tokio::join!(list_handle, check_handle);
    let list_data = list_data.unwrap_or(Value::Null);
    let check_data = check_data.unwrap_or(Value::Null);

    let skills_raw = list_data
        .get("skills")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut missing_map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    if let Some(arr) = check_data.get("missingRequirements").and_then(|v| v.as_array()) {
        for entry in arr {
            if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                missing_map.insert(name.to_string(), entry.clone());
            }
        }
    }

    let result: Vec<SkillInfo> = skills_raw
        .iter()
        .map(|s| {
            let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let check_entry = missing_map.get(&name);
            SkillInfo {
                name: name.clone(),
                description: s.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                emoji: s.get("emoji").and_then(|v| v.as_str()).map(String::from),
                eligible: s.get("eligible").and_then(|v| v.as_bool()).unwrap_or(false),
                disabled: s.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false),
                source: s.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                bundled: s.get("bundled").and_then(|v| v.as_bool()).unwrap_or(false),
                homepage: s.get("homepage").and_then(|v| v.as_str()).map(String::from),
                missing: s.get("missing").cloned().unwrap_or(serde_json::json!({"bins":[],"env":[],"config":[],"os":[]})),
                install: check_entry
                    .and_then(|e| e.get("install"))
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default(),
            }
        })
        .collect();

    Ok(result)
}

pub const SYSTEM_PROMPT: &str = include_str!("../prompts/skill/system.md");
pub const DEPS_TEMPLATE: &str = include_str!("../prompts/skill/deps.md");
pub const CLAWHUB_INSTALL_TEMPLATE: &str = include_str!("../prompts/skill/clawhub_install.md");

pub const MAX_INSTALL_RETRIES: usize = 16;

pub async fn agent_install(config: &AppConfig, prompt: &str) -> InstallResult {
    let llm = make_llm(&config.provider, &config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "SkillInstaller".to_string();
    agent.llm = Some(llm);
    agent.tools = vec![Box::new(BashTool::new())];
    agent.middlewares = vec![Box::new(LoggingMiddleware::new("skill"))];

    let mut session = Session::with_messages(vec![
        serde_json::json!({"role": "system", "content": SYSTEM_PROMPT}),
        serde_json::json!({"role": "user", "content": prompt}),
    ]);

    let mut outputs = Vec::new();

    for attempt in 0..MAX_INSTALL_RETRIES {
        agent.session = session.clone();
        let before = session.messages.len();
        let result = agent.run().await;

        match result {
            Ok(()) => {
                session = agent.session.clone();

                let text = session.messages[before..]
                    .iter()
                    .rev()
                    .find(|m| {
                        m.get("role").and_then(|v| v.as_str()) == Some("assistant")
                            && m.get("content")
                                .and_then(|v| v.as_str())
                                .map(|s| !s.is_empty())
                                .unwrap_or(false)
                            && m.get("tool_calls").is_none()
                    })
                    .and_then(|m| m.get("content").and_then(|v| v.as_str()))
                    .unwrap_or("");

                if text.is_empty() || text.starts_with("LLM call failed:") || text.starts_with("Error:") {
                    let msg = if text.is_empty() {
                        "Agent produced no response (check API key and endpoint)".to_string()
                    } else {
                        text.to_string()
                    };
                    log::error!("[agent_install] {}", msg);
                    outputs.push(msg);
                    break;
                }

                if let Some(parsed) = extract_json(text) {
                    if parsed.get("success").and_then(|v| v.as_bool()) == Some(true) {
                        if let Some(d) = parsed.get("details").and_then(|v| v.as_str()) {
                            outputs.push(d.to_string());
                        }
                        return InstallResult { ok: true, outputs };
                    }

                    let error = parsed.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                    outputs.push(error.to_string());

                    if attempt < MAX_INSTALL_RETRIES - 1 {
                        session.messages.push(serde_json::json!({
                            "role": "user",
                            "content": format!(
                                "The previous attempt failed: {}\n\n\
                                 Analyse the error, fix the root cause, and try again. \
                                 Respond with the same JSON schema.",
                                error
                            )
                        }));
                        continue;
                    }
                } else {
                    outputs.push(format!("Invalid response: {}", &text[..text.len().min(200)]));
                    if attempt < MAX_INSTALL_RETRIES - 1 {
                        session.messages.push(serde_json::json!({
                            "role": "user",
                            "content": "Your response was not valid JSON. \
                                        Please respond with ONLY a valid JSON object."
                        }));
                        continue;
                    }
                }
            }
            Err(e) => {
                log::error!("[agent_install] attempt {} error: {}", attempt, e);
                outputs.push(e.to_string());
                if attempt < MAX_INSTALL_RETRIES - 1 {
                    session.messages.push(serde_json::json!({
                        "role": "user",
                        "content": format!(
                            "Exception: {}\n\nAnalyse the error and try again. \
                             Respond with the same JSON schema.",
                            e
                        )
                    }));
                    continue;
                }
            }
        }
        break;
    }

    InstallResult { ok: false, outputs }
}

pub fn build_skill_deps_prompt(name: &str, install_instructions: &[Value]) -> String {
    let kind_cmd: HashMap<&str, Vec<&str>> = [
        ("brew", vec!["brew", "install"]),
        ("npm", vec!["npm", "install", "-g"]),
        ("pip", vec!["pip", "install"]),
        ("go", vec!["go", "install"]),
        ("cargo", vec!["cargo", "install"]),
    ]
    .into_iter()
    .collect();

    let mut commands = Vec::new();
    for inst in install_instructions {
        let kind = inst.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        let bins: Vec<&str> = inst
            .get("bins")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        if let Some(base) = kind_cmd.get(kind) {
            for bin in &bins {
                let mut parts = base.clone();
                parts.push(bin);
                commands.push(format!("  {}", parts.join(" ")));
            }
        }
    }

    render(DEPS_TEMPLATE, &[
        ("name", name),
        ("commands", &commands.join("\n")),
    ])
}

pub async fn install_skill(config: &AppConfig, name: &str) -> Result<InstallResult, SkillError> {
    let skills = list_skills(&config.openclaw_bin).await?;
    let matched = skills.iter().find(|s| s.name == name);
    let matched = matched.ok_or_else(|| SkillError::NotFound(name.to_string()))?;
    if matched.eligible {
        return Err(SkillError::AlreadyEligible(name.to_string()));
    }

    let install_instructions = &matched.install;
    if install_instructions.is_empty() {
        return Ok(InstallResult {
            ok: false,
            outputs: vec!["No install instructions available for this skill".to_string()],
        });
    }

    let prompt = build_skill_deps_prompt(name, install_instructions);
    Ok(agent_install(config, &prompt).await)
}

pub async fn install_clawhub_skill(config: &AppConfig, slug: &str) -> InstallResult {
    let prompt = render(CLAWHUB_INSTALL_TEMPLATE, &[("slug", slug)]);
    agent_install(config, &prompt).await
}

pub async fn uninstall_skill(openclaw_bin: &str, name: &str) -> Result<(), SkillError> {
    let skills = list_skills(openclaw_bin).await?;
    if let Some(matched) = skills.iter().find(|s| s.name == name) {
        if matched.bundled {
            return Err(SkillError::CannotUninstall(name.to_string()));
        }
    }

    let skill_dir = managed_skills_dir().join(name);
    let workspace_dir = workspace_skills_dir().join(name);
    if !skill_dir.is_dir() && !workspace_dir.is_dir() {
        return Err(SkillError::NotFound(name.to_string()));
    }
    if skill_dir.is_dir() {
        std::fs::remove_dir_all(&skill_dir)?;
    }
    if workspace_dir.is_dir() {
        let _ = std::fs::remove_dir_all(&workspace_dir);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_managed_skills_dir() {
        let dir = managed_skills_dir();
        assert!(dir.to_string_lossy().contains("skills"));
    }
}
