use serde_json::Value;
use tauri::ipc::Channel;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::openai::OpenAiLlm;
use crate::kernel::middleware::logging::LoggingMiddleware;
use crate::kernel::tool::bash::BashTool;
use crate::prompts::render;
use crate::services::step_runner::{run_step, run_step_dynamic, StepDef};

pub const SYSTEM_PROMPT: &str = include_str!("../prompts/install/system.md");

pub static STEP_DETECT_ENV: StepDef = StepDef {
    id: "detectEnv",
    prompt: include_str!("../prompts/install/detect_env.md"),
};

pub const SCRIPT_TEMPLATE: &str = include_str!("../prompts/install/script.md");
pub const CONFIGURE_TEMPLATE: &str = include_str!("../prompts/install/configure.md");
pub const VERIFY_TEMPLATE: &str = include_str!("../prompts/install/verify.md");

pub fn build_script_prompt(script_path: &str, label: &str, verify_cmd: &str, json_tpl: &str) -> String {
    render(SCRIPT_TEMPLATE, &[
        ("label", label),
        ("script_path", script_path),
        ("verify_cmd", verify_cmd),
        ("json_tpl", json_tpl),
    ])
}

pub fn build_configure_prompt(config: &AppConfig) -> String {
    render(CONFIGURE_TEMPLATE, &[
        ("base_url", &config.base_url),
        ("model", &config.model),
        ("api_key", &config.api_key),
        ("port", &config.gateway_port.to_string()),
    ])
}

pub fn build_verify_prompt(config: &AppConfig) -> String {
    render(VERIFY_TEMPLATE, &[
        ("port", &config.gateway_port.to_string()),
    ])
}

struct ResolvedScripts {
    git: String,
    node: String,
    openclaw: String,
}

fn resolve_scripts(app: &AppHandle) -> ResolvedScripts {
    let ext = if cfg!(windows) { "ps1" } else { "sh" };
    let resolve = |name: &str| -> String {
        app.path()
            .resolve(&format!("scripts/{name}.{ext}"), BaseDirectory::Resource)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };
    ResolvedScripts {
        git: resolve("install-git"),
        node: resolve("install-node"),
        openclaw: resolve("install-openclaw"),
    }
}

fn build_details(step_id: &str, parsed: &Value) -> Vec<String> {
    let mut details = Vec::new();
    match step_id {
        "detectEnv" => {
            if let Some(v) = parsed.get("os").and_then(|v| v.as_str()) { details.push(v.to_string()); }
            if let Some(v) = parsed.get("arch").and_then(|v| v.as_str()) { details.push(v.to_string()); }
            if let Some(v) = parsed.get("disk_free").and_then(|v| v.as_str()) { details.push(format!("Disk: {} free", v)); }
        }
        "installGit" => {
            if let Some(v) = parsed.get("version").and_then(|v| v.as_str()) { details.push(format!("git {}", v)); }
        }
        "installNode" => {
            if let Some(v) = parsed.get("version").and_then(|v| v.as_str()) { details.push(format!("node {}", v)); }
        }
        "installOpenClaw" => {
            if let Some(v) = parsed.get("version").and_then(|v| v.as_str()) { details.push(format!("openclaw {}", v)); }
        }
        "configure" => {
            if let Some(v) = parsed.get("config_path").and_then(|v| v.as_str()) { details.push(v.to_string()); }
            if let Some(v) = parsed.get("details").and_then(|v| v.as_str()) { details.push(v.to_string()); }
        }
        "verify" => {
            if let Some(v) = parsed.get("status").and_then(|v| v.as_str()) { details.push(v.to_string()); }
            if let Some(v) = parsed.get("port").and_then(|v| v.as_u64()) { details.push(format!("Port: {}", v)); }
        }
        _ => {}
    }
    if details.is_empty() { details.push("OK".to_string()); }
    details
}

pub async fn run_install(
    app: AppHandle,
    config: AppConfig,
    channel: Channel<Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let scripts = resolve_scripts(&app);

    let llm = OpenAiLlm::new(&config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "InstallRunner".to_string();
    agent.llm = Some(Box::new(llm));
    agent.tools = vec![Box::new(BashTool::new())];
    agent.middlewares = vec![Box::new(LoggingMiddleware::new("install"))];

    let mut session = Session::with_messages(vec![serde_json::json!({
        "role": "system", "content": SYSTEM_PROMPT
    })]);

    macro_rules! step {
        ($id:expr, $prompt:expr) => {
            if !run_step_dynamic(
                &mut agent, &mut session, $id, &$prompt,
                &channel, 16, Some(build_details), None,
            ).await {
                let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
                return Ok(());
            }
        };
    }

    if !run_step(&mut agent, &mut session, &STEP_DETECT_ENV, &channel, 16, Some(build_details), None).await {
        let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
        return Ok(());
    }

    let git_prompt = build_script_prompt(
        &scripts.git,
        "Install Git using the bundled script.",
        "git --version",
        "{\"success\": true, \"version\": \"<git version>\"}",
    );
    step!("installGit", git_prompt);

    let node_prompt = build_script_prompt(
        &scripts.node,
        "Install Node.js using the bundled script.",
        "node --version && npm --version",
        "{\"success\": true, \"version\": \"<node version>\"}",
    );
    step!("installNode", node_prompt);

    let openclaw_prompt = build_script_prompt(
        &scripts.openclaw,
        "Install OpenClaw using the bundled script.",
        "openclaw --version",
        "{\"success\": true, \"version\": \"<openclaw version>\"}",
    );
    step!("installOpenClaw", openclaw_prompt);

    step!("configure", build_configure_prompt(&config));
    step!("verify", build_verify_prompt(&config));

    let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
    Ok(())
}
