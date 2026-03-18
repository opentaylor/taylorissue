use serde_json::Value;
use tauri::ipc::Channel;

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::openai::OpenAiLlm;
use crate::kernel::middleware::logging::LoggingMiddleware;
use crate::kernel::tool::bash::BashTool;
use crate::services::step_runner::{run_step, StepDef};

pub const SYSTEM_PROMPT: &str = include_str!("../prompts/uninstall/system.md");

pub static ALL_STEPS: &[StepDef] = &[
    StepDef {
        id: "stopServices",
        prompt: include_str!("../prompts/uninstall/stop_services.md"),
    },
    StepDef {
        id: "removePackage",
        prompt: include_str!("../prompts/uninstall/remove_package.md"),
    },
    StepDef {
        id: "deleteWorkspace",
        prompt: include_str!("../prompts/uninstall/delete_workspace.md"),
    },
    StepDef {
        id: "deleteConfig",
        prompt: include_str!("../prompts/uninstall/delete_config.md"),
    },
    StepDef {
        id: "deleteData",
        prompt: include_str!("../prompts/uninstall/delete_data.md"),
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
    agent.middlewares = vec![Box::new(LoggingMiddleware::new("uninstall"))];

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
