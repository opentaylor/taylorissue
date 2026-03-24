use taylor_issue_lib::prompts::render;
use taylor_issue_lib::services::repair_runner::*;
use taylor_issue_lib::services::step_runner::run_step_standalone;

use crate::common::*;

#[tokio::test]
#[ignore]
async fn test_check_gateway() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_check_gateway");
    let mut session = make_session(SYSTEM_PROMPT);

    let config = make_config(&api_key, &base_url, &model);
    let prompt = build_check_gateway_prompt(&config);

    let result = run_step_standalone(
        &mut agent, &mut session, "checkGateway", &prompt, 16,
    ).await.expect("checkGateway step failed");

    assert_has_key(&result, "success");
    assert_has_key(&result, "status");
    assert_has_key(&result, "details");
    println!("[checkGateway] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_check_config() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_check_config");
    let mut session = make_session(SYSTEM_PROMPT);

    let config = make_config(&api_key, &base_url, &model);
    let prompt = build_check_config_prompt(&config);

    let result = run_step_standalone(
        &mut agent, &mut session, "checkConfig", &prompt, 16,
    ).await.expect("checkConfig step failed");

    assert_has_key(&result, "success");
    assert_has_key(&result, "details");
    println!("[checkConfig] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_check_model_request() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_check_model_request");
    let mut session = make_session(SYSTEM_PROMPT);

    let config = make_config(&api_key, &base_url, &model);
    let prompt = build_check_model_request_prompt(&config);

    let result = run_step_standalone(
        &mut agent, &mut session, "checkModelRequest", &prompt, 16,
    ).await.expect("checkModelRequest step failed");

    assert_has_key(&result, "success");
    println!("[checkModelRequest] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_run_doctor() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_run_doctor");
    let mut session = make_session(SYSTEM_PROMPT);

    let config = make_config(&api_key, &base_url, &model);
    let prompt = build_run_doctor_prompt(&config);

    let result = run_step_standalone(
        &mut agent, &mut session,
        "runDoctor", &prompt, 16,
    ).await.expect("runDoctor step failed");

    assert_has_key(&result, "success");
    println!("[runDoctor] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_fix() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_fix");
    let mut session = make_session(FIX_SYSTEM_PROMPT);

    let prompt = render(FIX_TEMPLATE, &[
        ("step_id", "checkGateway"),
        ("issue_description", "Gateway is not running on port 18789"),
    ]);
    let result = run_step_standalone(
        &mut agent, &mut session, "fix", &prompt, 16,
    ).await.expect("fix step failed");

    assert_has_key(&result, "success");
    println!("[fix] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_custom_fix_analyze() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_custom_fix_analyze");
    let mut session = make_session(FIX_SYSTEM_PROMPT);

    let prompt = render(CUSTOM_FIX_ANALYZE_TEMPLATE, &[
        ("problem", "OpenClaw gateway does not start"),
    ]);
    let result = run_step_standalone(
        &mut agent, &mut session, "analyze", &prompt, 16,
    ).await.expect("analyze step failed");

    assert_has_key(&result, "success");
    assert_has_key(&result, "summary");
    println!("[custom_fix_analyze] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_custom_fix_diagnose() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_custom_fix_diagnose");
    let mut session = make_session(FIX_SYSTEM_PROMPT);

    let result = run_step_standalone(
        &mut agent, &mut session, "diagnose", CUSTOM_FIX_DIAGNOSE_TEMPLATE, 16,
    ).await.expect("diagnose step failed");

    assert_has_key(&result, "success");
    println!("[custom_fix_diagnose] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_custom_fix_fix() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_custom_fix_fix");
    let mut session = make_session(FIX_SYSTEM_PROMPT);

    session.messages.push(serde_json::json!({
        "role": "assistant",
        "content": "{\"success\": true, \"summary\": \"OpenClaw config has model contextWindow set too low.\", \"system_info\": \"Windows 11 ARM64. OpenClaw installed via npm. Config at ~/.openclaw/openclaw.json.\"}"
    }));
    session.messages.push(serde_json::json!({
        "role": "assistant",
        "content": "{\"success\": true, \"root_cause\": \"Model contextWindow is 16000, should be 1000000\", \"details\": \"The contextWindow for models.providers.custom.models[0] is set to 16000 which is too low. Fix with: openclaw config set models.providers.custom.models[0].contextWindow 1000000 --strict-json\"}"
    }));

    let result = run_step_standalone(
        &mut agent, &mut session, "fix", CUSTOM_FIX_FIX_TEMPLATE, 16,
    ).await.expect("custom fix step failed");

    assert_has_key(&result, "success");
    println!("[custom_fix_fix] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_custom_fix_verify() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_custom_fix_verify");
    let mut session = make_session(FIX_SYSTEM_PROMPT);

    let result = run_step_standalone(
        &mut agent, &mut session, "verify", CUSTOM_FIX_VERIFY_TEMPLATE, 16,
    ).await.expect("custom fix verify step failed");

    assert_has_key(&result, "success");
    println!("[custom_fix_verify] {result:#}");
}
