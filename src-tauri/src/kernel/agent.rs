use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::kernel::llm::BaseLlm;
use crate::kernel::middleware::base::{AgentError, Middleware, Next, Phase};
use crate::kernel::tool::BaseTool;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    pub messages: Vec<Value>,
    pub cursor: usize,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            cursor: 0,
        }
    }

    pub fn with_messages(messages: Vec<Value>) -> Self {
        Self {
            messages,
            cursor: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Suspension {
    pub reason: String,
    pub payload: HashMap<String, Value>,
}

impl Suspension {
    pub fn new(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
            payload: HashMap::new(),
        }
    }

    pub fn with_payload(reason: &str, payload: HashMap<String, Value>) -> Self {
        Self {
            reason: reason.to_string(),
            payload,
        }
    }

    pub fn to_dict(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("reason".to_string(), Value::String(self.reason.clone()));
        for (k, v) in &self.payload {
            map.insert(k.clone(), v.clone());
        }
        Value::Object(map)
    }
}

impl std::fmt::Display for Suspension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Suspension({})", self.reason)
    }
}

#[derive(Debug, Clone)]
pub struct Suspensions {
    pub message: String,
    pub exceptions: Vec<Suspension>,
}

impl Suspensions {
    pub fn new(message: &str, exceptions: Vec<Suspension>) -> Self {
        Self {
            message: message.to_string(),
            exceptions,
        }
    }
}

impl std::fmt::Display for Suspensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Suspensions({})", self.message)
    }
}

impl std::error::Error for Suspensions {}

pub fn suspension(reason: &str, payload: HashMap<String, Value>) -> Suspensions {
    Suspensions::new("suspensions", vec![Suspension::with_payload(reason, payload)])
}

pub fn build_llm_messages(
    prefix_messages: &[Value],
    session: &Session,
    llm_messages_override: Option<&[Value]>,
) -> Vec<Value> {
    if let Some(override_msgs) = llm_messages_override {
        return override_msgs.to_vec();
    }
    let mut result = prefix_messages.to_vec();
    result.extend_from_slice(&session.messages[session.cursor..]);
    result
}

pub fn has_tool_calls(session: &Session) -> bool {
    let active = &session.messages[session.cursor..];
    if active.is_empty() {
        return false;
    }
    let last = &active[active.len() - 1];
    last.get("role").and_then(|v| v.as_str()) == Some("assistant")
        && last
            .get("tool_calls")
            .map(|tc| tc.is_array() && !tc.as_array().unwrap().is_empty())
            .unwrap_or(false)
}

pub fn has_pending_tool_calls(session: &Session) -> bool {
    let active = &session.messages[session.cursor..];
    let done: std::collections::HashSet<String> = active
        .iter()
        .filter(|m| m.get("role").and_then(|v| v.as_str()) == Some("tool"))
        .filter_map(|m| m.get("tool_call_id").and_then(|v| v.as_str()).map(String::from))
        .collect();

    for msg in active.iter().rev() {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
        if role == "assistant" {
            if let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                return tool_calls.iter().any(|tc| {
                    tc.get("id")
                        .and_then(|v| v.as_str())
                        .map(|id| !done.contains(id))
                        .unwrap_or(false)
                });
            }
        }
        if role == "tool" {
            continue;
        }
        break;
    }
    false
}

pub struct Agent {
    pub name: String,
    pub agent_dir: String,
    pub llm: Option<Box<dyn BaseLlm>>,
    pub tools: Vec<Box<dyn BaseTool>>,
    pub middlewares: Vec<Box<dyn Middleware>>,
    pub max_steps: usize,
    pub response_format: Option<Value>,
    pub metadata: HashMap<String, Value>,
    pub session: Session,
    pub prefix_messages: Vec<Value>,
    pub llm_messages: Option<Vec<Value>>,
    pub active_response_format: Option<Value>,
}

impl Agent {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            agent_dir: String::new(),
            llm: None,
            tools: Vec::new(),
            middlewares: Vec::new(),
            max_steps: 100,
            response_format: None,
            metadata: HashMap::new(),
            session: Session::new(),
            prefix_messages: Vec::new(),
            llm_messages: None,
            active_response_format: None,
        }
    }

    pub async fn run(&mut self) -> Result<(), AgentError> {
        self.active_response_format = self.response_format.clone();

        let mws = std::mem::take(&mut self.middlewares);

        Next { remaining: &mws, phase: Phase::Start }.call(self).await?;

        let mut step = 0;
        loop {
            if step >= self.max_steps {
                break;
            }

            if has_pending_tool_calls(&self.session) {
                Next { remaining: &mws, phase: Phase::Tool }.call(self).await?;
            } else {
                self.prefix_messages.clear();
                Next { remaining: &mws, phase: Phase::Llm }.call(self).await?;
                step += 1;

                if !has_tool_calls(&self.session) {
                    break;
                }
                Next { remaining: &mws, phase: Phase::Tool }.call(self).await?;
            }
        }

        Next { remaining: &mws, phase: Phase::End }.call(self).await?;
        self.middlewares = mws;
        Ok(())
    }

    pub async fn call_llm(&mut self) -> Result<(), AgentError> {
        let messages = build_llm_messages(
            &self.prefix_messages,
            &self.session,
            self.llm_messages.as_deref(),
        );

        if let Some(ref llm) = self.llm {
            let tool_defs: Vec<Value> = self
                .tools
                .iter()
                .map(|t| t.to_openai_schema())
                .collect();
            match llm
                .run(messages, Some(tool_defs), self.active_response_format.clone())
                .await
            {
                Ok(new_msgs) => {
                    self.session.messages.extend(new_msgs);
                }
                Err(e) => {
                    log::error!("LLM call failed: {}", e);
                    self.session.messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": format!("LLM call failed: {}", e),
                    }));
                }
            }
        } else {
            log::error!("No LLM configured for agent '{}'", self.name);
            self.session.messages.push(serde_json::json!({
                "role": "assistant",
                "content": "Error: No LLM configured for this agent.",
            }));
        }
        Ok(())
    }

    pub async fn execute_tool_calls(&mut self) -> Result<(), AgentError> {
        let active = &self.session.messages[self.session.cursor..];

        let last_assistant = active
            .iter()
            .rev()
            .find(|m| {
                m.get("role").and_then(|v| v.as_str()) == Some("assistant")
                    && m.get("tool_calls").is_some()
            })
            .cloned();

        let Some(last) = last_assistant else {
            return Ok(());
        };

        let done: std::collections::HashSet<String> = active
            .iter()
            .filter(|m| m.get("role").and_then(|v| v.as_str()) == Some("tool"))
            .filter_map(|m| m.get("tool_call_id").and_then(|v| v.as_str()).map(String::from))
            .collect();

        let tool_calls = last
            .get("tool_calls")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let pending: Vec<Value> = tool_calls
            .into_iter()
            .filter(|tc| {
                tc.get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| !done.contains(id))
                    .unwrap_or(false)
            })
            .collect();

        if pending.is_empty() {
            return Ok(());
        }

        let tool_map: HashMap<String, &dyn BaseTool> = self
            .tools
            .iter()
            .map(|t| (t.name().to_string(), t.as_ref()))
            .collect();

        let mut suspensions: Vec<Suspension> = Vec::new();

        for tc in &pending {
            let tc_id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let func = tc.get("function").cloned().unwrap_or(Value::Null);
            let name = func.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args_raw = func.get("arguments").cloned().unwrap_or(Value::String("{}".to_string()));
            let args: Value = if let Some(s) = args_raw.as_str() {
                serde_json::from_str(s).unwrap_or(Value::Object(serde_json::Map::new()))
            } else {
                args_raw
            };

            if let Some(tool) = tool_map.get(name) {
                match tool.run(args.clone()).await {
                    Ok(result) => {
                        self.session.messages.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": tc_id,
                            "content": result,
                        }));
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        if err_str.starts_with("Suspension:") {
                            let mut payload = HashMap::new();
                            payload.insert(
                                "tool_call_id".to_string(),
                                Value::String(tc_id.to_string()),
                            );
                            suspensions.push(Suspension::with_payload("permission", payload));
                        } else {
                            self.session.messages.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tc_id,
                                "content": format!("Error: {}", e),
                            }));
                        }
                    }
                }
            } else {
                let available: Vec<&str> = tool_map.keys().map(|k| k.as_str()).collect();
                self.session.messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tc_id,
                    "content": format!("Error: Tool '{}' not found. Available: {:?}", name, available),
                }));
            }
        }

        if !suspensions.is_empty() {
            return Err(Box::new(Suspensions::new("suspensions", suspensions)));
        }

        Ok(())
    }

    pub fn has_tool_calls(&self) -> bool {
        has_tool_calls(&self.session)
    }

    pub fn has_pending_tool_calls(&self) -> bool {
        has_pending_tool_calls(&self.session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockLlm {
        responses: Mutex<Vec<Vec<Value>>>,
    }

    impl MockLlm {
        fn new(responses: Vec<Vec<Value>>) -> Self {
            Self { responses: Mutex::new(responses) }
        }
    }

    #[async_trait]
    impl crate::kernel::llm::BaseLlm for MockLlm {
        fn id(&self) -> &str { "mock" }
        fn api_key(&self) -> &str { "" }
        fn base_url(&self) -> &str { "" }
        fn model(&self) -> &str { "mock-model" }

        async fn run(
            &self,
            _messages: Vec<Value>,
            _tools: Option<Vec<Value>>,
            _response_format: Option<Value>,
        ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                return Err("no more mock responses".into());
            }
            Ok(responses.remove(0))
        }
    }

    struct MockTool {
        tool_name: String,
        response: Mutex<Vec<Result<String, String>>>,
    }

    impl MockTool {
        fn ok(name: &str, result: &str) -> Self {
            Self {
                tool_name: name.to_string(),
                response: Mutex::new(vec![Ok(result.to_string())]),
            }
        }

        fn err(name: &str, error: &str) -> Self {
            Self {
                tool_name: name.to_string(),
                response: Mutex::new(vec![Err(error.to_string())]),
            }
        }

        fn multi(name: &str, responses: Vec<Result<String, String>>) -> Self {
            Self {
                tool_name: name.to_string(),
                response: Mutex::new(responses),
            }
        }
    }

    #[async_trait]
    impl crate::kernel::tool::BaseTool for MockTool {
        fn name(&self) -> &str { &self.tool_name }
        fn description(&self) -> &str { "mock" }
        fn params_schema(&self) -> Value { serde_json::json!({"type": "object"}) }

        async fn run(&self, _args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            let mut responses = self.response.lock().unwrap();
            if responses.is_empty() {
                return Ok("default".to_string());
            }
            match responses.remove(0) {
                Ok(s) => Ok(s),
                Err(e) => Err(e.into()),
            }
        }
    }

    fn tc(id: &str, name: &str, args: &str) -> Value {
        serde_json::json!({
            "id": id,
            "type": "function",
            "function": {"name": name, "arguments": args}
        })
    }

    fn assistant_with_tool_calls(tool_calls: Vec<Value>) -> Value {
        serde_json::json!({
            "role": "assistant",
            "content": "",
            "tool_calls": tool_calls,
        })
    }

    fn tool_result(tc_id: &str, content: &str) -> Value {
        serde_json::json!({
            "role": "tool",
            "tool_call_id": tc_id,
            "content": content,
        })
    }

    #[test]
    fn test_session_new() {
        let s = Session::new();
        assert!(s.messages.is_empty());
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_session_default() {
        let s = Session::default();
        assert!(s.messages.is_empty());
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_session_with_messages() {
        let msgs = vec![serde_json::json!({"role": "user", "content": "hi"})];
        let s = Session::with_messages(msgs.clone());
        assert_eq!(s.messages.len(), 1);
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_suspension_new() {
        let s = Suspension::new("test");
        assert_eq!(s.reason, "test");
        assert!(s.payload.is_empty());
    }

    #[test]
    fn test_suspension_with_payload() {
        let mut payload = HashMap::new();
        payload.insert("key".to_string(), Value::String("val".to_string()));
        let s = Suspension::with_payload("reason", payload);
        assert_eq!(s.reason, "reason");
        assert_eq!(s.payload["key"], "val");
    }

    #[test]
    fn test_suspension_to_dict() {
        let mut payload = HashMap::new();
        payload.insert("tool_call_id".to_string(), Value::String("tc1".to_string()));
        let s = Suspension::with_payload("permission", payload);
        let dict = s.to_dict();
        assert_eq!(dict["reason"], "permission");
        assert_eq!(dict["tool_call_id"], "tc1");
    }

    #[test]
    fn test_suspension_display() {
        let s = Suspension::new("test_reason");
        assert_eq!(format!("{}", s), "Suspension(test_reason)");
    }

    #[test]
    fn test_suspensions_new() {
        let exceptions = vec![Suspension::new("a"), Suspension::new("b")];
        let s = Suspensions::new("msg", exceptions);
        assert_eq!(s.message, "msg");
        assert_eq!(s.exceptions.len(), 2);
    }

    #[test]
    fn test_suspensions_display() {
        let s = Suspensions::new("halt", vec![]);
        assert_eq!(format!("{}", s), "Suspensions(halt)");
    }

    #[test]
    fn test_suspensions_is_error() {
        let s = Suspensions::new("err", vec![]);
        let _e: &dyn std::error::Error = &s;
    }

    #[test]
    fn test_suspension_factory() {
        let mut payload = HashMap::new();
        payload.insert("k".to_string(), Value::Bool(true));
        let s = suspension("reason", payload);
        assert_eq!(s.message, "suspensions");
        assert_eq!(s.exceptions.len(), 1);
        assert_eq!(s.exceptions[0].reason, "reason");
        assert_eq!(s.exceptions[0].payload["k"], true);
    }

    #[test]
    fn test_build_llm_messages_empty() {
        let session = Session::new();
        let result = build_llm_messages(&[], &session, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_build_llm_messages_prefix_and_session() {
        let prefix = vec![serde_json::json!({"role": "system", "content": "sys"})];
        let session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        let result = build_llm_messages(&prefix, &session, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[1]["role"], "user");
    }

    #[test]
    fn test_build_llm_messages_cursor_offset() {
        let mut session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "old"}),
            serde_json::json!({"role": "assistant", "content": "old reply"}),
            serde_json::json!({"role": "user", "content": "new"}),
        ]);
        session.cursor = 2;
        let result = build_llm_messages(&[], &session, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["content"], "new");
    }

    #[test]
    fn test_build_llm_messages_override() {
        let session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "ignored"}),
        ]);
        let override_msgs = vec![serde_json::json!({"role": "system", "content": "override"})];
        let result = build_llm_messages(&[], &session, Some(&override_msgs));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["content"], "override");
    }

    #[test]
    fn test_has_tool_calls_empty() {
        let session = Session::new();
        assert!(!has_tool_calls(&session));
    }

    #[test]
    fn test_has_tool_calls_no_tool_calls() {
        let session = Session::with_messages(vec![
            serde_json::json!({"role": "assistant", "content": "hello"}),
        ]);
        assert!(!has_tool_calls(&session));
    }

    #[test]
    fn test_has_tool_calls_with_tool_calls() {
        let session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "bash", "{}")]),
        ]);
        assert!(has_tool_calls(&session));
    }

    #[test]
    fn test_has_tool_calls_empty_tool_calls_array() {
        let session = Session::with_messages(vec![
            serde_json::json!({"role": "assistant", "content": "", "tool_calls": []}),
        ]);
        assert!(!has_tool_calls(&session));
    }

    #[test]
    fn test_has_tool_calls_with_cursor() {
        let mut session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "bash", "{}")]),
            serde_json::json!({"role": "assistant", "content": "done"}),
        ]);
        session.cursor = 1;
        assert!(!has_tool_calls(&session));
    }

    #[test]
    fn test_has_pending_tool_calls_none() {
        let session = Session::with_messages(vec![
            serde_json::json!({"role": "assistant", "content": "hi"}),
        ]);
        assert!(!has_pending_tool_calls(&session));
    }

    #[test]
    fn test_has_pending_tool_calls_all_done() {
        let session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "bash", "{}")]),
            tool_result("tc1", "ok"),
        ]);
        assert!(!has_pending_tool_calls(&session));
    }

    #[test]
    fn test_has_pending_tool_calls_partial() {
        let session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![
                tc("tc1", "bash", "{}"),
                tc("tc2", "read", "{}"),
            ]),
            tool_result("tc1", "ok"),
        ]);
        assert!(has_pending_tool_calls(&session));
    }

    #[test]
    fn test_has_pending_tool_calls_with_cursor() {
        let mut session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "bash", "{}")]),
            serde_json::json!({"role": "user", "content": "new turn"}),
        ]);
        session.cursor = 1;
        assert!(!has_pending_tool_calls(&session));
    }

    #[test]
    fn test_agent_new_defaults() {
        let agent = Agent::new();
        assert!(agent.name.is_empty());
        assert!(agent.llm.is_none());
        assert!(agent.tools.is_empty());
        assert!(agent.middlewares.is_empty());
        assert_eq!(agent.max_steps, 100);
        assert!(agent.session.messages.is_empty());
    }

    #[tokio::test]
    async fn test_call_llm_no_llm() {
        let mut agent = Agent::new();
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        agent.call_llm().await.unwrap();
        let last = agent.session.messages.last().unwrap();
        assert_eq!(last["role"], "assistant");
        assert!(last["content"].as_str().unwrap().contains("No LLM configured"));
    }

    #[tokio::test]
    async fn test_call_llm_success() {
        let mut agent = Agent::new();
        agent.llm = Some(Box::new(MockLlm::new(vec![
            vec![serde_json::json!({"role": "assistant", "content": "hello!"})],
        ])));
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        agent.call_llm().await.unwrap();
        assert_eq!(agent.session.messages.len(), 2);
        assert_eq!(agent.session.messages[1]["content"], "hello!");
    }

    #[tokio::test]
    async fn test_call_llm_error() {
        let mut agent = Agent::new();
        agent.llm = Some(Box::new(MockLlm::new(vec![])));
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        agent.call_llm().await.unwrap();
        let last = agent.session.messages.last().unwrap();
        assert!(last["content"].as_str().unwrap().contains("LLM call failed"));
    }

    #[tokio::test]
    async fn test_execute_tool_calls_success() {
        let mut agent = Agent::new();
        agent.tools = vec![Box::new(MockTool::ok("echo", "echoed"))];
        agent.session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "echo", "{}")]),
        ]);
        agent.execute_tool_calls().await.unwrap();
        assert_eq!(agent.session.messages.len(), 2);
        assert_eq!(agent.session.messages[1]["content"], "echoed");
        assert_eq!(agent.session.messages[1]["tool_call_id"], "tc1");
    }

    #[tokio::test]
    async fn test_execute_tool_calls_unknown_tool() {
        let mut agent = Agent::new();
        agent.session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "nonexistent", "{}")]),
        ]);
        agent.execute_tool_calls().await.unwrap();
        let last = agent.session.messages.last().unwrap();
        assert!(last["content"].as_str().unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_execute_tool_calls_error() {
        let mut agent = Agent::new();
        agent.tools = vec![Box::new(MockTool::err("fail_tool", "something broke"))];
        agent.session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "fail_tool", "{}")]),
        ]);
        agent.execute_tool_calls().await.unwrap();
        let last = agent.session.messages.last().unwrap();
        assert!(last["content"].as_str().unwrap().contains("Error: something broke"));
    }

    #[tokio::test]
    async fn test_execute_tool_calls_suspension() {
        let mut agent = Agent::new();
        agent.tools = vec![Box::new(MockTool::err("sus", "Suspension: needs approval"))];
        agent.session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "sus", "{}")]),
        ]);
        let result = agent.execute_tool_calls().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let suspensions = err.downcast_ref::<Suspensions>().unwrap();
        assert_eq!(suspensions.exceptions.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_tool_calls_no_pending() {
        let mut agent = Agent::new();
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "assistant", "content": "no tools"}),
        ]);
        agent.execute_tool_calls().await.unwrap();
        assert_eq!(agent.session.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_execute_tool_calls_already_completed() {
        let mut agent = Agent::new();
        agent.tools = vec![Box::new(MockTool::ok("echo", "echoed"))];
        agent.session = Session::with_messages(vec![
            assistant_with_tool_calls(vec![tc("tc1", "echo", "{}")]),
            tool_result("tc1", "already done"),
        ]);
        agent.execute_tool_calls().await.unwrap();
        assert_eq!(agent.session.messages.len(), 2);
    }

    #[tokio::test]
    async fn test_run_single_turn_no_tools() {
        let mut agent = Agent::new();
        agent.llm = Some(Box::new(MockLlm::new(vec![
            vec![serde_json::json!({"role": "assistant", "content": "done"})],
        ])));
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        agent.run().await.unwrap();
        assert_eq!(agent.session.messages.len(), 2);
        assert_eq!(agent.session.messages[1]["content"], "done");
    }

    #[tokio::test]
    async fn test_run_tool_loop() {
        let mut agent = Agent::new();
        agent.tools = vec![Box::new(MockTool::ok("echo", "echoed"))];
        agent.llm = Some(Box::new(MockLlm::new(vec![
            vec![assistant_with_tool_calls(vec![tc("tc1", "echo", "{}")])],
            vec![serde_json::json!({"role": "assistant", "content": "final"})],
        ])));
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        agent.run().await.unwrap();

        let roles: Vec<&str> = agent.session.messages.iter()
            .map(|m| m["role"].as_str().unwrap())
            .collect();
        assert_eq!(roles, vec!["user", "assistant", "tool", "assistant"]);
        assert_eq!(agent.session.messages[3]["content"], "final");
    }

    #[tokio::test]
    async fn test_run_max_steps() {
        let mut agent = Agent::new();
        agent.max_steps = 2;
        agent.llm = Some(Box::new(MockLlm::new(vec![
            vec![assistant_with_tool_calls(vec![tc("tc1", "echo", "{}")])],
            vec![assistant_with_tool_calls(vec![tc("tc2", "echo", "{}")])],
            vec![serde_json::json!({"role": "assistant", "content": "should not reach"})],
        ])));
        agent.tools = vec![Box::new(MockTool::multi("echo", vec![
            Ok("r1".into()), Ok("r2".into()), Ok("r3".into()),
        ]))];
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        agent.run().await.unwrap();

        let llm_calls = agent.session.messages.iter()
            .filter(|m| m["role"] == "assistant")
            .count();
        assert_eq!(llm_calls, 2);
    }

    #[tokio::test]
    async fn test_run_restores_middlewares() {
        use crate::kernel::middleware::base::Middleware;

        struct CounterMiddleware;

        #[async_trait]
        impl Middleware for CounterMiddleware {}

        let mut agent = Agent::new();
        agent.llm = Some(Box::new(MockLlm::new(vec![
            vec![serde_json::json!({"role": "assistant", "content": "ok"})],
        ])));
        agent.session = Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        agent.middlewares = vec![Box::new(CounterMiddleware)];
        agent.run().await.unwrap();
        assert_eq!(agent.middlewares.len(), 1);
    }
}
