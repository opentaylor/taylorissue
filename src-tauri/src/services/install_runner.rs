use serde_json::Value;
use tauri::ipc::Channel;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::base::make_llm;
use crate::kernel::middleware::logging::LoggingMiddleware;
use crate::kernel::tool::bash::BashTool;
use crate::prompts::render;
use crate::services::setup_detection;
use crate::services::shell_env;
use crate::services::step_runner::{run_step, run_step_dynamic, StepDef};
use std::process::Stdio;

pub const SYSTEM_PROMPT: &str = include_str!("../prompts/install/system.md");

pub static STEP_DETECT_ENV: StepDef = StepDef {
    id: "detectEnv",
    prompt: include_str!("../prompts/install/detect_env.md"),
};

pub const SCRIPT_TEMPLATE: &str = include_str!("../prompts/install/script.md");
pub const CONFIGURE_TEMPLATE: &str = include_str!("../prompts/install/configure.md");
pub const START_GATEWAY_TEMPLATE: &str = include_str!("../prompts/install/start_gateway.md");
pub const VERIFY_TEMPLATE: &str = include_str!("../prompts/install/verify.md");

pub fn build_script_prompt(script_path: &str, label: &str, verify_cmd: &str, json_tpl: &str) -> String {
    render(SCRIPT_TEMPLATE, &[
        ("label", label),
        ("script_path", script_path),
        ("verify_cmd", verify_cmd),
        ("json_tpl", json_tpl),
    ])
}

pub fn build_configure_prompt(config: &AppConfig, openclaw_bin: &str) -> String {
    render(CONFIGURE_TEMPLATE, &[
        ("base_url", &config.base_url),
        ("model", &config.model),
        ("api_key", &config.api_key),
        ("port", &config.gateway_port.to_string()),
        ("openclaw_bin", openclaw_bin),
    ])
}

pub fn build_start_gateway_prompt(config: &AppConfig, openclaw_bin: &str) -> String {
    render(START_GATEWAY_TEMPLATE, &[
        ("port", &config.gateway_port.to_string()),
        ("openclaw_bin", openclaw_bin),
    ])
}

pub fn build_verify_prompt(config: &AppConfig, openclaw_bin: &str) -> String {
    render(VERIFY_TEMPLATE, &[
        ("port", &config.gateway_port.to_string()),
        ("openclaw_bin", openclaw_bin),
    ])
}

struct ResolvedScripts {
    git: String,
    node: String,
    openclaw: String,
}

fn safe_script_path(source: &std::path::Path, filename: &str) -> String {
    let s = source.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    if s.is_ascii() {
        return s.to_string();
    }
    let scripts_dir = taylorissue_dir().join("scripts");
    if std::fs::create_dir_all(&scripts_dir).is_err() {
        log::error!("Failed to create script dir: {}", scripts_dir.display());
        return s.to_string();
    }
    let dest = scripts_dir.join(filename);
    match std::fs::copy(source, &dest) {
        Ok(_) => {
            log::info!("Copied script to ASCII-safe path: {}", dest.display());
            dest.to_string_lossy().to_string()
        }
        Err(e) => {
            log::error!("Failed to copy script to temp: {e}");
            s.to_string()
        }
    }
}

fn taylorissue_dir() -> std::path::PathBuf {
    #[cfg(windows)]
    {
        std::env::var("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_default()
                    .join("AppData")
                    .join("Local")
            })
            .join("taylorissue")
    }
    #[cfg(not(windows))]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".taylorissue")
    }
}

fn resolve_scripts(app: &AppHandle) -> ResolvedScripts {
    let ext = if cfg!(windows) { "ps1" } else { "sh" };
    let resolve = |name: &str| -> String {
        let relative = format!("scripts/{name}.{ext}");
        let filename = format!("{name}.{ext}");

        let found = app
            .path()
            .resolve(&relative, BaseDirectory::Resource)
            .ok()
            .filter(|p| p.exists())
            .or_else(|| {
                std::env::current_exe().ok().and_then(|exe| {
                    let p = exe.parent()?.join(&relative);
                    p.exists().then_some(p)
                })
            });

        match found {
            Some(path) => safe_script_path(&path, &filename),
            None => {
                log::error!("Script not found anywhere: {}", relative);
                String::new()
            }
        }
    };
    let scripts = ResolvedScripts {
        git: resolve("install-git"),
        node: resolve("install-node"),
        openclaw: resolve("install-openclaw"),
    };
    log::info!("[install] script paths: git={}, node={}, openclaw={}", scripts.git, scripts.node, scripts.openclaw);
    scripts
}

fn run_version(bin_path: &str) -> Option<String> {
    let mut cmd = shell_env::build_command(bin_path);
    cmd.arg("--version");
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    shell_env::apply_env(&mut cmd);
    let output = cmd.output().ok().filter(|o| o.status.success())?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|l| l.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn detect_in_path(name: &str) -> Option<(String, String)> {
    let enriched = shell_env::full_path();
    let path = which::which_in(name, Some(&enriched), ".").ok()?;
    let path_str = path.to_string_lossy().to_string();
    let version = run_version(&path_str)?;
    Some((path_str, version))
}

fn emit_step(channel: &Channel<Value>, step_id: &str, details: Vec<String>) {
    let _ = channel.send(serde_json::json!({
        "event": "step",
        "data": {"step_id": step_id, "status": "active"}
    }));
    let _ = channel.send(serde_json::json!({
        "event": "step",
        "data": {"step_id": step_id, "status": "complete", "details": details}
    }));
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
        "startGateway" => {
            if let Some(v) = parsed.get("port").and_then(|v| v.as_u64()) { details.push(format!("Port: {}", v)); }
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

    let llm = make_llm(&config.provider, &config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "InstallRunner".to_string();
    agent.llm = Some(llm);
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

    shell_env::refresh_path();

    let openclaw_resolved = setup_detection::detect_openclaw_bin();
    let openclaw_version = openclaw_resolved.as_ref().and_then(|r| run_version(&r.bin_path));
    let openclaw_runnable = openclaw_resolved.is_some() && openclaw_version.is_some();

    emit_step(&channel, "checkOpenClaw", match (&openclaw_resolved, &openclaw_version) {
        (Some(r), Some(ver)) => vec![
            format!("Found: {}", ver),
            r.bin_path.clone(),
            format!("Type: {}", r.install_type.as_str()),
        ],
        (Some(r), None) => vec![
            format!("Found at {} but cannot run", r.bin_path),
        ],
        _ => vec!["Not installed".to_string()],
    });

    let git_info = detect_in_path("git");
    if let Some((ref path, ref version)) = git_info {
        log::info!("[install] git detected: {} at {}", version, path);
        emit_step(&channel, "installGit", vec![
            format!("Detected: {}", version),
            path.clone(),
        ]);
    } else {
        let git_prompt = build_script_prompt(
            &scripts.git,
            "Install Git using the bundled script.",
            "git --version",
            "{\"success\": true, \"version\": \"<git version>\"}",
        );
        step!("installGit", git_prompt);
        shell_env::refresh_path();
    }

    let node_info = detect_in_path("node");
    if let Some((ref path, ref version)) = node_info {
        log::info!("[install] node detected: {} at {}", version, path);
        emit_step(&channel, "installNode", vec![
            format!("Detected: {}", version),
            path.clone(),
        ]);
    } else {
        let node_prompt = build_script_prompt(
            &scripts.node,
            "Install Node.js using the bundled script.",
            "node --version; npm --version",
            "{\"success\": true, \"version\": \"<node version>\"}",
        );
        step!("installNode", node_prompt);
        shell_env::refresh_path();
    }

    if openclaw_runnable {
        let r = openclaw_resolved.as_ref().unwrap();
        let ver = openclaw_version.as_ref().unwrap();
        log::info!("[install] openclaw detected: {} at {}", ver, r.bin_path);
        emit_step(&channel, "installOpenClaw", vec![
            format!("Detected: {}", ver),
            r.bin_path.clone(),
        ]);
    } else {
        let openclaw_prompt = build_script_prompt(
            &scripts.openclaw,
            "Install OpenClaw using the bundled script.",
            "openclaw --version",
            "{\"success\": true, \"version\": \"<openclaw version>\"}",
        );
        step!("installOpenClaw", openclaw_prompt);
        shell_env::refresh_path();
    }

    let openclaw_bin = setup_detection::detect_openclaw_bin_path()
        .unwrap_or_else(|| "openclaw".to_string());

    step!("configure", build_configure_prompt(&config, &openclaw_bin));
    step!("startGateway", build_start_gateway_prompt(&config, &openclaw_bin));
    step!("verify", build_verify_prompt(&config, &openclaw_bin));

    let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
    Ok(())
}
