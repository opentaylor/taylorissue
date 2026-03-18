use serde_json::Value;
use tauri::ipc::Channel;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::openai::OpenAiLlm;
use crate::kernel::tool::bash::BashTool;
use crate::services::step_runner::{run_step, run_step_dynamic, StepDef};

const SYSTEM_PROMPT: &str = "\
You are an automated installer for OpenClaw. \
You execute shell commands one at a time and report results as JSON. \
You MUST respond with ONLY a valid JSON object — no markdown, no explanation. \
If a step fails, set \"success\" to false with the reason in \"error\". \
IMPORTANT: Run scripts ONE AT A TIME, sequentially. Never combine or parallelize them.";

static STEP_DETECT_ENV: StepDef = StepDef {
    id: "detectEnv",
    prompt: "\
Detect the current system environment.\n\
Run: uname -s && uname -r && uname -m && df -h /\n\
On macOS also run: sw_vers\n\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"os\": \"<OS name and version>\", \
\"arch\": \"<CPU architecture>\", \
\"disk_free\": \"<free disk space with unit>\"}\n\
On failure: {\"success\": false, \"error\": \"<reason>\"}",
};

fn build_script_prompt(script_path: &str, label: &str, verify_cmd: &str, json_tpl: &str) -> String {
    format!(
        "{label}\n\
         Run this exact command:\n  bash {script_path}\n\n\
         Do NOT use sudo. Do NOT modify the script or download a different one.\n\
         Set timeout to 300 for this command.\n\n\
         After the script finishes, verify: {verify_cmd}\n\
         Respond with ONLY this JSON:\n{json_tpl}\n\
         On failure: {{\"success\": false, \"error\": \"<reason>\"}}"
    )
}

fn build_configure_prompt(config: &AppConfig) -> String {
    format!(
        "Configure OpenClaw with the user's model provider.\n\n\
         Step 1 — Run onboard (copy verbatim, do NOT change any values):\n\n\
         openclaw onboard --non-interactive \\\n\
           --mode local \\\n\
           --auth-choice custom-api-key \\\n\
           --custom-base-url '{base_url}' \\\n\
           --custom-model-id '{model}' \\\n\
           --custom-api-key '{api_key}' \\\n\
           --custom-compatibility openai \\\n\
           --accept-risk \\\n\
           --gateway-port {port} \\\n\
           --gateway-bind loopback \\\n\
           --install-daemon \\\n\
           --skip-skills \\\n\
           --skip-channels \\\n\
           --skip-search\n\n\
         Step 2 — Patch the model limits (onboard defaults are too low for most models):\n\n\
         python3 -c \"\nimport json, pathlib\n\
         p = pathlib.Path.home() / '.openclaw' / 'openclaw.json'\n\
         c = json.loads(p.read_text())\n\
         for prov in c.get('models', {{}}).get('providers', {{}}).values():\n\
             for m in prov.get('models', []):\n\
                 m['contextWindow'] = 1000000\n\
                 m['maxTokens'] = 32768\n\
         p.write_text(json.dumps(c, indent=2))\n\
         print('patched contextWindow=1000000 maxTokens=32768')\n\"\n\n\
         Step 3 — Restart the gateway so it picks up the patched config:\n\n\
           openclaw gateway restart\n\n\
         After all three steps finish, verify: ls ~/.openclaw/openclaw.json\n\n\
         Respond with ONLY this JSON:\n\
         {{\"success\": true, \"config_path\": \"<path to config file>\", \
         \"details\": \"<brief summary>\"}}\n\
         On failure: {{\"success\": false, \"error\": \"<reason>\"}}",
        base_url = config.base_url,
        model = config.model,
        api_key = config.api_key,
        port = config.gateway_port,
    )
}

fn build_verify_prompt(config: &AppConfig) -> String {
    format!(
        "Verify the OpenClaw gateway is healthy.\n\
         Run these commands in order:\n\
           openclaw gateway status\n\
           openclaw health\n\n\
         If gateway status shows it is not running, start it:\n\
           openclaw gateway --port {port} &\n\
         Then re-check: openclaw gateway status\n\n\
         Respond with ONLY this JSON:\n\
         {{\"success\": true, \"status\": \"<running|stopped>\", \"port\": <port number>}}\n\
         On failure: {{\"success\": false, \"error\": \"<reason>\"}}",
        port = config.gateway_port,
    )
}

struct ResolvedScripts {
    git: String,
    node: String,
    openclaw: String,
}

fn resolve_scripts(app: &AppHandle) -> ResolvedScripts {
    let resolve = |name: &str| -> String {
        app.path()
            .resolve(&format!("scripts/{name}"), BaseDirectory::Resource)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };
    ResolvedScripts {
        git: resolve("install-git.sh"),
        node: resolve("install-node.sh"),
        openclaw: resolve("install-openclaw.sh"),
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
