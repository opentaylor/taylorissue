use taylor_issue_lib::services::step_runner::run_step_standalone;
use taylor_issue_lib::services::uninstall_runner::*;

use crate::common::*;

// ---- stopServices ----

#[tokio::test]
#[ignore]
async fn test_stop_services() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_stop_services");
    let mut session = make_session(SYSTEM_PROMPT);

    let step = &ALL_STEPS[0];
    assert_eq!(step.id, "stopServices");

    let result = run_step_standalone(
        &mut agent, &mut session, step.id, step.prompt, 16,
    ).await.expect("stopServices step failed");

    assert_has_key(&result, "success");
    println!("[stopServices] {result:#}");
}

// ---- removePackage ----

#[tokio::test]
#[ignore]
async fn test_remove_package() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_remove_package");
    let mut session = make_session(SYSTEM_PROMPT);

    let step = &ALL_STEPS[1];
    assert_eq!(step.id, "removePackage");

    let result = run_step_standalone(
        &mut agent, &mut session, step.id, step.prompt, 16,
    ).await.expect("removePackage step failed");

    assert_has_key(&result, "success");
    println!("[removePackage] {result:#}");
}

// ---- deleteWorkspace ----

#[tokio::test]
#[ignore]
async fn test_delete_workspace() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_delete_workspace");
    let mut session = make_session(SYSTEM_PROMPT);

    let step = &ALL_STEPS[2];
    assert_eq!(step.id, "deleteWorkspace");

    let result = run_step_standalone(
        &mut agent, &mut session, step.id, step.prompt, 16,
    ).await.expect("deleteWorkspace step failed");

    assert_has_key(&result, "success");
    println!("[deleteWorkspace] {result:#}");
}

// ---- deleteConfig ----

#[tokio::test]
#[ignore]
async fn test_delete_config() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_delete_config");
    let mut session = make_session(SYSTEM_PROMPT);

    let step = &ALL_STEPS[3];
    assert_eq!(step.id, "deleteConfig");

    let result = run_step_standalone(
        &mut agent, &mut session, step.id, step.prompt, 16,
    ).await.expect("deleteConfig step failed");

    assert_has_key(&result, "success");
    println!("[deleteConfig] {result:#}");
}

// ---- deleteData ----

#[tokio::test]
#[ignore]
async fn test_delete_data() {
    let (api_key, base_url, model) = load_env();
    let mut agent = make_agent(&api_key, &base_url, &model, "test_delete_data");
    let mut session = make_session(SYSTEM_PROMPT);

    let step = &ALL_STEPS[4];
    assert_eq!(step.id, "deleteData");

    let result = run_step_standalone(
        &mut agent, &mut session, step.id, step.prompt, 16,
    ).await.expect("deleteData step failed");

    assert_has_key(&result, "success");
    println!("[deleteData] {result:#}");
}
