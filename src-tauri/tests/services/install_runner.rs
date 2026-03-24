use taylor_issue_lib::services::install_runner::*;
use taylor_issue_lib::services::step_runner::run_step_standalone;

use crate::common::*;

#[tokio::test]
#[ignore]
async fn test_detect_env() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_detect_env");
    let mut session = make_session(SYSTEM_PROMPT);

    let result = run_step_standalone(
        &mut agent, &mut session,
        STEP_DETECT_ENV.id, STEP_DETECT_ENV.prompt, 16,
    ).await.expect("detectEnv step failed");

    assert_has_key(&result, "success");
    assert_has_key(&result, "os");
    assert_has_key(&result, "arch");
    assert_has_key(&result, "disk_free");
    println!("[detectEnv] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_install_git() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_install_git");
    let mut session = make_session(SYSTEM_PROMPT);

    let prompt = build_script_prompt(
        "echo 'skip script in test'",
        "Check whether Git is installed; if not, report failure.",
        "git --version",
        r#"{"success": true, "version": "<git version>"}"#,
    );
    let result = run_step_standalone(
        &mut agent, &mut session, "installGit", &prompt, 16,
    ).await.expect("installGit step failed");

    assert_has_key(&result, "success");
    println!("[installGit] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_install_node() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_install_node");
    let mut session = make_session(SYSTEM_PROMPT);

    let prompt = build_script_prompt(
        "echo 'skip script in test'",
        "Check whether Node.js is installed; if not, report failure.",
        "node --version && npm --version",
        r#"{"success": true, "version": "<node version>"}"#,
    );
    let result = run_step_standalone(
        &mut agent, &mut session, "installNode", &prompt, 16,
    ).await.expect("installNode step failed");

    assert_has_key(&result, "success");
    println!("[installNode] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_configure() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_configure");
    let mut session = make_session(SYSTEM_PROMPT);

    let config = make_config(&api_key, &base_url, &model);
    let prompt = build_configure_prompt(&config, "openclaw");

    let result = run_step_standalone(
        &mut agent, &mut session, "configure", &prompt, 16,
    ).await.expect("configure step failed");

    assert_has_key(&result, "success");
    println!("[configure] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_verify() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_verify");
    let mut session = make_session(SYSTEM_PROMPT);

    let config = make_config(&api_key, &base_url, &model);
    let prompt = build_verify_prompt(&config, "openclaw");

    let result = run_step_standalone(
        &mut agent, &mut session, "verify", &prompt, 16,
    ).await.expect("verify step failed");

    assert_has_key(&result, "success");
    assert_has_key(&result, "status");
    assert_has_key(&result, "port");
    println!("[verify] {result:#}");
}
