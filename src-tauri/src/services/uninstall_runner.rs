use serde_json::Value;
use tauri::ipc::Channel;

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::openai::OpenAiLlm;
use crate::kernel::tool::bash::BashTool;
use crate::services::step_runner::{run_step, StepDef};

const SYSTEM_PROMPT: &str = "\
You are an automated uninstaller for OpenClaw. \
You execute shell commands to remove installed components. \
You MUST respond with ONLY a valid JSON object — no markdown, no explanation. \
If a step fails, set \"success\" to false with the reason in \"error\". \
IMPORTANT: Before removing anything, CHECK whether it exists first. \
If it does not exist, report success and note it was already absent.";

static ALL_STEPS: &[StepDef] = &[
    StepDef {
        id: "stopServices",
        prompt: "\
Stop and uninstall the OpenClaw gateway service.\n\
Run these commands in order (skip any that fail gracefully):\n\
  1. openclaw gateway stop 2>/dev/null || true\n\
  2. openclaw gateway uninstall 2>/dev/null || true\n\n\
If the openclaw CLI is not installed, check for leftover service files:\n\
  macOS: ls ~/Library/LaunchAgents/ai.openclaw.gateway.plist 2>/dev/null && \
rm -f ~/Library/LaunchAgents/ai.openclaw.gateway.plist\n\
  Linux: systemctl --user stop openclaw-gateway 2>/dev/null; \
rm -f ~/.config/systemd/user/openclaw-gateway.service && \
systemctl --user daemon-reload 2>/dev/null\n\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"was_running\": true|false, \"details\": \"<summary>\"}\n\
On failure: {\"success\": false, \"error\": \"<reason>\"}",
    },
    StepDef {
        id: "removePackage",
        prompt: "\
Uninstall the OpenClaw npm package.\n\
  1. npm rm -g openclaw 2>/dev/null || true\n\
  2. Verify it is gone: which openclaw && echo 'still present' || echo 'removed'\n\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"version_removed\": \"<version or unknown>\", \"details\": \"<summary>\"}\n\
On failure: {\"success\": false, \"error\": \"<reason>\"}",
    },
    StepDef {
        id: "deleteWorkspace",
        prompt: "\
Delete the OpenClaw workspace directory.\n\
  ls -d ~/.openclaw/workspace 2>/dev/null\n\
  If it exists: rm -rf ~/.openclaw/workspace\n\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"existed\": true|false, \"details\": \"<summary>\"}\n\
On failure: {\"success\": false, \"error\": \"<reason>\"}",
    },
    StepDef {
        id: "deleteConfig",
        prompt: "\
Delete OpenClaw configuration files from ~/.openclaw.\n\
Target files: openclaw.json, openclaw.json.bak, packs.json, update-check.json\n\
  ls these files, delete any that exist.\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"files_deleted\": [\"<file1>\", ...], \"details\": \"<summary>\"}\n\
On failure: {\"success\": false, \"error\": \"<reason>\"}",
    },
    StepDef {
        id: "deleteData",
        prompt: "\
Delete all remaining data inside ~/.openclaw.\n\
This includes: credentials, agents, sessions, logs, secrets.\n\
  ls ~/.openclaw/ 2>/dev/null\n\
  If has contents: rm -rf ~/.openclaw\n\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"items_removed\": [\"<item1>\", ...], \"details\": \"<summary>\"}\n\
On failure: {\"success\": false, \"error\": \"<reason>\"}",
    },
];

fn build_uninstall_details(step_id: &str, parsed: &Value) -> Vec<String> {
    let mut details = Vec::new();
    match step_id {
        "stopServices" => {
            if parsed.get("was_running").and_then(|v| v.as_bool()) == Some(true) {
                details.push("Service was running".to_string());
            } else {
                details.push("Service was not running".to_string());
            }
        }
        "removePackage" => {
            if let Some(v) = parsed.get("version_removed").and_then(|v| v.as_str()) {
                details.push(format!("openclaw {} removed", v));
            }
        }
        "deleteWorkspace" => {
            if parsed.get("existed").and_then(|v| v.as_bool()) == Some(true) {
                details.push("Workspace deleted".to_string());
            } else {
                details.push("Workspace already absent".to_string());
            }
        }
        "deleteConfig" => {
            if let Some(arr) = parsed.get("files_deleted").and_then(|v| v.as_array()) {
                for f in arr {
                    if let Some(s) = f.as_str() { details.push(s.to_string()); }
                }
            }
        }
        "deleteData" => {
            if let Some(arr) = parsed.get("items_removed").and_then(|v| v.as_array()) {
                details.push(format!("{} items removed", arr.len()));
            }
        }
        _ => {}
    }
    if let Some(v) = parsed.get("details").and_then(|v| v.as_str()) {
        details.push(v.to_string());
    }
    if details.is_empty() { details.push("OK".to_string()); }
    details
}

pub async fn run_uninstall(
    config: AppConfig,
    selected_options: Vec<String>,
    channel: Channel<Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let llm = OpenAiLlm::new(&config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "UninstallRunner".to_string();
    agent.llm = Some(Box::new(llm));
    agent.tools = vec![Box::new(BashTool::new())];

    let mut session = Session::with_messages(vec![serde_json::json!({
        "role": "system", "content": SYSTEM_PROMPT
    })]);

    let steps: Vec<&StepDef> = ALL_STEPS
        .iter()
        .filter(|s| selected_options.iter().any(|o| o == s.id))
        .collect();

    for step in steps {
        let ok = run_step(&mut agent, &mut session, step, &channel, 16, Some(build_uninstall_details), None).await;
        if !ok { break; }
    }

    let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
    Ok(())
}
