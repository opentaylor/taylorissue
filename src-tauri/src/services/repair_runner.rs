use serde_json::Value;
use tauri::ipc::Channel;

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::openai::OpenAiLlm;
use crate::kernel::middleware::logging::LoggingMiddleware;
use crate::kernel::tool::bash::BashTool;
use crate::prompts::render;
use crate::services::step_runner::{run_step_dynamic, StepDef};

pub const SYSTEM_PROMPT: &str = include_str!("../prompts/repair/system.md");
pub const FIX_SYSTEM_PROMPT: &str = include_str!("../prompts/repair/fix_system.md");

pub const CHECK_GATEWAY_TEMPLATE: &str = include_str!("../prompts/repair/check_gateway.md");
pub const CHECK_CONFIG_TEMPLATE: &str = include_str!("../prompts/repair/check_config.md");
pub const CHECK_MODEL_REQUEST_TEMPLATE: &str = include_str!("../prompts/repair/check_model_request.md");
pub const FIX_TEMPLATE: &str = include_str!("../prompts/repair/fix.md");
pub const CUSTOM_FIX_ANALYZE_TEMPLATE: &str = include_str!("../prompts/repair/custom_fix_analyze.md");
pub const CUSTOM_FIX_DIAGNOSE_TEMPLATE: &str = include_str!("../prompts/repair/custom_fix_diagnose.md");
pub const CUSTOM_FIX_FIX_TEMPLATE: &str = include_str!("../prompts/repair/custom_fix_fix.md");
pub const CUSTOM_FIX_VERIFY_TEMPLATE: &str = include_str!("../prompts/repair/custom_fix_verify.md");

pub static STEP_RUN_DOCTOR: StepDef = StepDef {
    id: "runDoctor",
    prompt: include_str!("../prompts/repair/run_doctor.md"),
};

pub fn build_check_gateway_prompt(config: &AppConfig) -> String {
    render(CHECK_GATEWAY_TEMPLATE, &[
        ("port", &config.gateway_port.to_string()),
    ])
}

pub fn build_check_config_prompt(config: &AppConfig) -> String {
    render(CHECK_CONFIG_TEMPLATE, &[
        ("model", &config.model),
        ("base_url", &config.base_url),
        ("port", &config.gateway_port.to_string()),
    ])
}

pub fn build_check_model_request_prompt(config: &AppConfig) -> String {
    let base = config.base_url.trim_end_matches('/');
    let completions_url = if base.ends_with("/v1") {
        format!("{}/chat/completions", base)
    } else {
        format!("{}/v1/chat/completions", base)
    };

    render(CHECK_MODEL_REQUEST_TEMPLATE, &[
        ("completions_url", &completions_url),
        ("api_key", &config.api_key),
        ("model", &config.model),
        ("port", &config.gateway_port.to_string()),
        ("gateway_token", &config.gateway_token),
    ])
}

fn build_repair_details(step_id: &str, parsed: &Value) -> Vec<String> {
    let mut details = Vec::new();
    match step_id {
        "checkGateway" => {
            if let Some(v) = parsed.get("status").and_then(|v| v.as_str()) { details.push(v.to_string()); }
            if let Some(v) = parsed.get("port").and_then(|v| v.as_u64()) { details.push(format!("Port: {}", v)); }
        }
        "checkConfig" => {
            if parsed.get("model_configured").and_then(|v| v.as_bool()) == Some(true) {
                details.push("Model configured".to_string());
            } else {
                details.push("Model not configured".to_string());
            }
        }
        "checkModelRequest" => {
            if let Some(v) = parsed.get("http_status").and_then(|v| v.as_u64()) {
                details.push(format!("Provider: HTTP {}", v));
            }
            if let Some(v) = parsed.get("gateway_status").and_then(|v| v.as_u64()) {
                details.push(format!("Gateway: HTTP {}", v));
            }
        }
        "runDoctor" => {
            let warnings = parsed.get("warnings").and_then(|v| v.as_u64()).unwrap_or(0);
            let issues = parsed.get("issues").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
            if warnings == 0 && issues == 0 {
                details.push("No issues".to_string());
            } else {
                if warnings > 0 { details.push(format!("{} warnings", warnings)); }
                if issues > 0 { details.push(format!("{} issues", issues)); }
            }
        }
        _ => {}
    }
    if let Some(v) = parsed.get("details").and_then(|v| v.as_str()) { details.push(v.to_string()); }
    if details.is_empty() { details.push("OK".to_string()); }
    details
}

fn has_issue(step_id: &str, parsed: &Value) -> bool {
    match step_id {
        "checkGateway" => {
            let status = parsed.get("status").and_then(|v| v.as_str()).unwrap_or("");
            !status.contains("running")
        }
        "checkConfig" => {
            parsed.get("model_configured").and_then(|v| v.as_bool()) != Some(true)
                || parsed.get("has_config").and_then(|v| v.as_bool()) != Some(true)
        }
        "checkModelRequest" => {
            let provider_ok = parsed
                .get("http_status")
                .and_then(|v| v.as_u64())
                .map(|s| s == 200)
                .unwrap_or(false);
            !provider_ok
        }
        "runDoctor" => {
            let warnings = parsed.get("warnings").and_then(|v| v.as_u64()).unwrap_or(0);
            let issues = parsed.get("issues").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
            warnings > 0 || issues > 0
        }
        _ => false,
    }
}

pub async fn run_repair(
    config: AppConfig,
    channel: Channel<Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let llm = OpenAiLlm::new(&config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "RepairRunner".to_string();
    agent.llm = Some(Box::new(llm));
    agent.tools = vec![Box::new(BashTool::new())];
    agent.middlewares = vec![Box::new(LoggingMiddleware::new("repair"))];

    let mut session = Session::with_messages(vec![serde_json::json!({
        "role": "system", "content": SYSTEM_PROMPT
    })]);

    let gateway_prompt = build_check_gateway_prompt(&config);
    run_step_dynamic(&mut agent, &mut session, "checkGateway", &gateway_prompt, &channel, 16, Some(build_repair_details), Some(has_issue)).await;

    let config_prompt = build_check_config_prompt(&config);
    run_step_dynamic(&mut agent, &mut session, "checkConfig", &config_prompt, &channel, 16, Some(build_repair_details), Some(has_issue)).await;

    let model_prompt = build_check_model_request_prompt(&config);
    run_step_dynamic(&mut agent, &mut session, "checkModelRequest", &model_prompt, &channel, 16, Some(build_repair_details), Some(has_issue)).await;

    run_step_dynamic(&mut agent, &mut session, "runDoctor", STEP_RUN_DOCTOR.prompt, &channel, 16, Some(build_repair_details), Some(has_issue)).await;

    let session_id = uuid::Uuid::new_v4().to_string();
    let _ = channel.send(serde_json::json!({"event": "done", "data": {"session_id": session_id}}));
    Ok(())
}

pub async fn run_fix(
    config: AppConfig,
    _session_id: &str,
    step_id: &str,
    issue_description: &str,
    channel: Channel<Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let llm = OpenAiLlm::new(&config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "FixRunner".to_string();
    agent.llm = Some(Box::new(llm));
    agent.tools = vec![Box::new(BashTool::new())];
    agent.middlewares = vec![Box::new(LoggingMiddleware::new("fix"))];

    let prompt = render(FIX_TEMPLATE, &[
        ("step_id", step_id),
        ("issue_description", issue_description),
    ]);

    let mut session = Session::with_messages(vec![serde_json::json!({
        "role": "system", "content": FIX_SYSTEM_PROMPT
    })]);

    run_step_dynamic(&mut agent, &mut session, "fix", &prompt, &channel, 16, None, None).await;

    let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
    Ok(())
}

pub async fn run_custom_fix(
    config: AppConfig,
    problem: &str,
    channel: Channel<Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let llm = OpenAiLlm::new(&config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "CustomFixRunner".to_string();
    agent.llm = Some(Box::new(llm));
    agent.tools = vec![Box::new(BashTool::new())];
    agent.middlewares = vec![Box::new(LoggingMiddleware::new("custom_fix"))];

    let mut session = Session::with_messages(vec![serde_json::json!({
        "role": "system", "content": FIX_SYSTEM_PROMPT
    })]);

    let analyze_prompt = render(CUSTOM_FIX_ANALYZE_TEMPLATE, &[("problem", problem)]);

    struct DynStep { id: &'static str, prompt: String }

    let steps = vec![
        DynStep { id: "analyze", prompt: analyze_prompt },
        DynStep { id: "diagnose", prompt: CUSTOM_FIX_DIAGNOSE_TEMPLATE.to_string() },
        DynStep { id: "fix", prompt: CUSTOM_FIX_FIX_TEMPLATE.to_string() },
        DynStep { id: "verify", prompt: CUSTOM_FIX_VERIFY_TEMPLATE.to_string() },
    ];

    for step in &steps {
        let ok = run_step_dynamic(&mut agent, &mut session, step.id, &step.prompt, &channel, 16, None, None).await;
        if !ok { break; }
    }

    let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
    Ok(())
}
