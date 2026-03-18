use taylor_issue_lib::prompts::render;
use taylor_issue_lib::services::skill_store::*;

use crate::common::*;

// ---- agent_install with deps prompt ----

#[tokio::test]
#[ignore]
async fn test_install_deps() {
    let (api_key, base_url, model) = load_env();
    let config = make_config(&api_key, &base_url, &model);

    let cmds = if cfg!(windows) {
        "python --version\npip --version"
    } else {
        "python3 --version\npip3 --version"
    };
    let prompt = render(DEPS_TEMPLATE, &[
        ("name", "web-pilot"),
        ("commands", cmds),
    ]);
    let result = agent_install(&config, &prompt).await;

    println!("[install_deps] ok={}, outputs={:?}", result.ok, result.outputs);
    assert!(
        result.ok || !result.outputs.is_empty(),
        "agent_install returned no output"
    );
}

// ---- agent_install with clawhub prompt ----

#[tokio::test]
#[ignore]
async fn test_clawhub_install() {
    let (api_key, base_url, model) = load_env();
    let config = make_config(&api_key, &base_url, &model);

    let prompt = render(CLAWHUB_INSTALL_TEMPLATE, &[("slug", "web-pilot")]);
    let result = agent_install(&config, &prompt).await;

    println!("[clawhub_install] ok={}, outputs={:?}", result.ok, result.outputs);
    assert!(
        result.ok || !result.outputs.is_empty(),
        "agent_install returned no output"
    );
}
