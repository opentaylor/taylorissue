use serde_json::Value;
use tauri::ipc::Channel;

use crate::config::AppConfig;
use crate::kernel::agent::{Agent, Session};
use crate::kernel::llm::openai::OpenAiLlm;
use crate::kernel::tool::bash::BashTool;
use crate::services::step_runner::{run_step_dynamic, StepDef};

const SYSTEM_PROMPT: &str = "\
You are an automated diagnostic scanner for OpenClaw. \
You execute shell commands to CHECK system health and REPORT findings. \
You MUST respond with ONLY a valid JSON object — no markdown, no explanation. \
CRITICAL: You are READ-ONLY. NEVER attempt to fix, install, start, stop, \
restart, or modify anything. Only run commands that read or check status. \
If you find a problem, describe it and what the user should do — \
but do NOT fix it yourself.";

fn build_check_gateway_prompt(config: &AppConfig) -> String {
    format!(
        "Check the OpenClaw gateway health. DO NOT fix anything.\n\
         Run these commands in order:\n\
           1. which openclaw || echo 'not installed'\n\
           2. openclaw gateway status 2>&1\n\
           3. openclaw health 2>&1\n\n\
         The gateway should be running on port {port}.\n\n\
         Respond with ONLY this JSON:\n\
         {{\"success\": true, \"status\": \"<running|stopped|not installed>\", \
         \"port\": <port number or null>, \
         \"details\": \"<description of gateway state>\"}}",
        port = config.gateway_port,
    )
}

fn build_check_config_prompt(config: &AppConfig) -> String {
    format!(
        "Check the OpenClaw configuration. DO NOT fix anything.\n\
         Run these commands:\n\
           1. cat ~/.openclaw/openclaw.json 2>/dev/null | head -80\n\
           2. Check that a model provider is configured with a baseUrl and apiKey\n\
           3. Check contextWindow and maxTokens values (should be large, e.g. 1000000 and 32768)\n\n\
         The expected model should be: {model}\n\
         The expected base URL should contain: {base_url}\n\
         The gateway port should be: {port}\n\n\
         Respond with ONLY this JSON:\n\
         {{\"success\": true, \"has_config\": true|false, \
         \"model_configured\": true|false, \
         \"details\": \"<description of config state>\"}}",
        model = config.model,
        base_url = config.base_url,
        port = config.gateway_port,
    )
}

fn build_check_model_request_prompt(config: &AppConfig) -> String {
    let base = config.base_url.trim_end_matches('/');
    let completions_url = if base.ends_with("/v1") {
        format!("{}/chat/completions", base)
    } else {
        format!("{}/v1/chat/completions", base)
    };

    format!(
        "Test whether the model provider responds to a chat completion request. \
         DO NOT fix anything.\n\n\
         First, try the provider directly:\n\
           curl -s -o /dev/null -w '%{{http_code}}' -m 30 -X POST \\\n\
             '{completions_url}' \\\n\
             -H 'Content-Type: application/json' \\\n\
             -H 'Authorization: Bearer {api_key}' \\\n\
             -d '{{\"model\": \"{model}\", \"max_tokens\": 16, \
         \"messages\": [{{\"role\": \"user\", \"content\": \"Say OK\"}}]}}'\n\n\
         Then, if the gateway is running, also test the gateway endpoint:\n\
           curl -s -o /dev/null -w '%{{http_code}}' -m 30 -X POST \\\n\
             'http://localhost:{port}/v1/chat/completions' \\\n\
             -H 'Content-Type: application/json' \\\n\
             -H 'Authorization: Bearer {gateway_token}' \\\n\
             -d '{{\"model\": \"{model}\", \"max_tokens\": 16, \
         \"messages\": [{{\"role\": \"user\", \"content\": \"Say OK\"}}]}}'\n\n\
         HTTP 200 = working. 404 from gateway = chatCompletions endpoint not enabled \
         (this is fine if the provider test passed). Other codes indicate a problem.\n\n\
         Respond with ONLY this JSON:\n\
         {{\"success\": true, \"http_status\": <provider status code or null>, \
         \"gateway_status\": <gateway status code or null>, \
         \"details\": \"<description>\"}}",
        completions_url = completions_url,
        api_key = config.api_key,
        model = config.model,
        port = config.gateway_port,
        gateway_token = config.gateway_token,
    )
}

static STEP_RUN_DOCTOR: StepDef = StepDef {
    id: "runDoctor",
    prompt: "\
Run the OpenClaw built-in diagnostics. DO NOT fix anything.\n\
  1. openclaw doctor 2>&1\n\
  2. openclaw status 2>&1\n\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"warnings\": <number of warnings>, \
\"issues\": [\"<issue1>\", ...], \
\"details\": \"<doctor output summary>\"}",
};

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

const FIX_SYSTEM_PROMPT: &str = "\
You are an automated diagnostic and repair tool for OpenClaw. \
You execute shell commands to analyse, diagnose, fix, and verify issues. \
You MUST respond with ONLY a valid JSON object — no markdown, no explanation. \
IMPORTANT: After any config change, restart the gateway with: openclaw gateway restart";

pub async fn run_repair(
    config: AppConfig,
    channel: Channel<Value>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let llm = OpenAiLlm::new(&config.api_key, &config.base_url, &config.model);
    let mut agent = Agent::new();
    agent.name = "RepairRunner".to_string();
    agent.llm = Some(Box::new(llm));
    agent.tools = vec![Box::new(BashTool::new())];

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

    let prompt = format!(
        "The diagnostic scan for step \"{}\" found the following issue:\n\n\
         {}\n\n\
         Fix this issue now. You CAN and SHOULD run commands that modify system state.\n\
         IMPORTANT: If you modify ~/.openclaw/openclaw.json, you MUST run \
         'openclaw gateway restart' afterwards so changes take effect.\n\n\
         Respond with ONLY this JSON:\n\
         {{\"success\": true, \"details\": \"<what was done to fix it>\"}}\n\
         On failure: {{\"success\": false, \"error\": \"<why the fix did not work>\"}}",
        step_id, issue_description
    );

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

    let mut session = Session::with_messages(vec![serde_json::json!({
        "role": "system", "content": FIX_SYSTEM_PROMPT
    })]);

    let analyze_prompt = format!(
        "Analyse the user's problem. Gather system info with read-only commands.\n\n\
         User's problem:\n{}\n\n\
         Respond with ONLY this JSON:\n\
         {{\"success\": true, \"summary\": \"<concise summary>\", \"system_info\": \"<relevant info>\"}}",
        problem
    );

    struct DynStep { id: &'static str, prompt: String }

    let steps = vec![
        DynStep { id: "analyze", prompt: analyze_prompt },
        DynStep { id: "diagnose", prompt: "\
Based on your analysis, diagnose the root cause. Run additional diagnostic commands if needed.\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"root_cause\": \"<identified root cause>\", \"details\": \"<explanation>\"}".to_string() },
        DynStep { id: "fix", prompt: "\
Fix the diagnosed issue. You CAN and SHOULD modify system state.\n\
IMPORTANT: If you modify ~/.openclaw/openclaw.json, run 'openclaw gateway restart' afterwards.\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"actions\": [\"<action1>\", ...], \"details\": \"<summary>\"}\n\
On failure: {\"success\": false, \"error\": \"<why the fix did not work>\"}".to_string() },
        DynStep { id: "verify", prompt: "\
Verify the fix resolved the problem. Re-run checks that were failing.\n\
Respond with ONLY this JSON:\n\
{\"success\": true, \"verified\": true, \"details\": \"<verification results>\"}\n\
If still broken: {\"success\": true, \"verified\": false, \"details\": \"<what is still broken>\"}".to_string() },
    ];

    for step in &steps {
        let ok = run_step_dynamic(&mut agent, &mut session, step.id, &step.prompt, &channel, 16, None, None).await;
        if !ok { break; }
    }

    let _ = channel.send(serde_json::json!({"event": "done", "data": {}}));
    Ok(())
}
