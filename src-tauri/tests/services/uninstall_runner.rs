use taylor_issue_lib::prompts::render;
use taylor_issue_lib::services::step_runner::run_step_standalone;
use taylor_issue_lib::services::uninstall_runner::*;

use crate::common::*;

fn render_step(template: &str) -> String {
    render(template, &[
        ("openclaw_bin", "openclaw"),
        ("install_type", "official"),
    ])
}

#[tokio::test]
#[ignore]
async fn test_stop_services() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_stop_services");
    let mut session = make_session(SYSTEM_PROMPT);

    let prompt = render_step(STOP_SERVICES_TEMPLATE);
    let result = run_step_standalone(
        &mut agent, &mut session, "stopServices", &prompt, 16,
    ).await.expect("stopServices step failed");

    assert_has_key(&result, "success");
    println!("[stopServices] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_remove_package() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_remove_package");
    let mut session = make_session(SYSTEM_PROMPT);

    let prompt = render_step(REMOVE_PACKAGE_TEMPLATE);
    let result = run_step_standalone(
        &mut agent, &mut session, "removePackage", &prompt, 16,
    ).await.expect("removePackage step failed");

    assert_has_key(&result, "success");
    println!("[removePackage] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_delete_workspace() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_delete_workspace");
    let mut session = make_session(SYSTEM_PROMPT);

    let prompt = render_step(DELETE_WORKSPACE_TEMPLATE);
    let result = run_step_standalone(
        &mut agent, &mut session, "deleteWorkspace", &prompt, 16,
    ).await.expect("deleteWorkspace step failed");

    assert_has_key(&result, "success");
    println!("[deleteWorkspace] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_delete_config() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_delete_config");
    let mut session = make_session(SYSTEM_PROMPT);

    let prompt = render_step(DELETE_CONFIG_TEMPLATE);
    let result = run_step_standalone(
        &mut agent, &mut session, "deleteConfig", &prompt, 16,
    ).await.expect("deleteConfig step failed");

    assert_has_key(&result, "success");
    println!("[deleteConfig] {result:#}");
}

#[tokio::test]
#[ignore]
async fn test_delete_data() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_delete_data");
    let mut session = make_session(SYSTEM_PROMPT);

    let prompt = render_step(DELETE_DATA_TEMPLATE);
    let result = run_step_standalone(
        &mut agent, &mut session, "deleteData", &prompt, 16,
    ).await.expect("deleteData step failed");

    assert_has_key(&result, "success");
    println!("[deleteData] {result:#}");
}
