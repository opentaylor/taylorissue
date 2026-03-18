use taylor_issue_lib::config::AppConfig;
use taylor_issue_lib::kernel::agent::{Agent, Session};
use taylor_issue_lib::kernel::llm::openai::OpenAiLlm;
use taylor_issue_lib::kernel::middleware::logging::LoggingMiddleware;
use taylor_issue_lib::kernel::tool::bash::BashTool;

pub fn load_env() -> (String, String, String) {
    let _ = env_logger::try_init();
    dotenvy::from_filename("../.env").ok();
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set in .env");
    let base_url = std::env::var("OPENAI_BASE_URL").expect("OPENAI_BASE_URL must be set in .env");
    let model = std::env::var("MODEL").expect("MODEL must be set in .env");
    (api_key, base_url, model)
}

pub fn make_agent(api_key: &str, base_url: &str, model: &str, name: &str) -> Agent {
    let llm = OpenAiLlm::new(api_key, base_url, model);
    let mut agent = Agent::new();
    agent.name = name.to_string();
    agent.llm = Some(Box::new(llm));
    agent.tools = vec![Box::new(BashTool::new())];
    agent.middlewares = vec![Box::new(LoggingMiddleware::new("test"))];
    agent
}

pub fn make_session(system_prompt: &str) -> Session {
    Session::with_messages(vec![serde_json::json!({
        "role": "system", "content": system_prompt
    })])
}

pub fn make_config(api_key: &str, base_url: &str, model: &str) -> AppConfig {
    AppConfig {
        api_key: api_key.to_string(),
        base_url: base_url.to_string(),
        model: model.to_string(),
        workspace_path: String::new(),
        gateway_token: String::new(),
        gateway_port: 18789,
        openclaw_bin: String::new(),
    }
}

pub fn assert_has_key(json: &serde_json::Value, key: &str) {
    assert!(
        json.get(key).is_some(),
        "Response JSON missing key \"{key}\": {json}"
    );
}
