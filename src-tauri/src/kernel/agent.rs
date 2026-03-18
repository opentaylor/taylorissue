// Phase 5: Full implementation of Session, Suspension, Agent ReAct loop
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::kernel::llm::BaseLlm;
use crate::kernel::middleware::Middleware;
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
    pub session: Option<Session>,
    pub prefix_messages: Vec<Value>,
    pub llm_messages: Option<Vec<Value>>,
    active_response_format: Option<Value>,
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
            session: None,
            prefix_messages: Vec::new(),
            llm_messages: None,
            active_response_format: None,
        }
    }

    pub async fn run(
        &mut self,
        mut session: Session,
        response_format: Option<Value>,
        kwargs: HashMap<String, Value>,
    ) -> Result<Session, Suspensions> {
        self.session = Some(session.clone());
        self.metadata = kwargs;
        self.prefix_messages = Vec::new();
        self.active_response_format = response_format.or(self.response_format.clone());

        {
            let mws = std::mem::take(&mut self.middlewares);
            for mw in &mws { let _ = mw.wrap_start(self).await; }
            self.middlewares = mws;
        }

        let mut step = 0;
        loop {
            if step >= self.max_steps {
                break;
            }

            if has_pending_tool_calls(&session) {
                self.session = Some(session.clone());
                {
                    let mws = std::mem::take(&mut self.middlewares);
                    for mw in &mws { let _ = mw.wrap_tool(self).await; }
                    self.middlewares = mws;
                }
                self.execute_tool_calls(&mut session).await?;
            } else {
                self.session = Some(session.clone());
                {
                    let mws = std::mem::take(&mut self.middlewares);
                    for mw in &mws { let _ = mw.wrap_llm(self).await; }
                    self.middlewares = mws;
                }

                self.prefix_messages = Vec::new();
                self.call_llm(&mut session).await?;
                step += 1;

                if !has_tool_calls(&session) {
                    break;
                }

                self.session = Some(session.clone());
                {
                    let mws = std::mem::take(&mut self.middlewares);
                    for mw in &mws { let _ = mw.wrap_tool(self).await; }
                    self.middlewares = mws;
                }
                self.execute_tool_calls(&mut session).await?;
            }
        }

        self.session = Some(session.clone());
        {
            let mws = std::mem::take(&mut self.middlewares);
            for mw in &mws { let _ = mw.wrap_end(self).await; }
            self.middlewares = mws;
        }
        Ok(session)
    }

    async fn call_llm(&mut self, session: &mut Session) -> Result<(), Suspensions> {
        let messages = build_llm_messages(
            &self.prefix_messages,
            session,
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
                    session.messages.extend(new_msgs);
                }
                Err(e) => {
                    log::error!("LLM call failed: {}", e);
                    session.messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": format!("LLM call failed: {}", e),
                    }));
                }
            }
        } else {
            log::error!("No LLM configured for agent '{}'", self.name);
            session.messages.push(serde_json::json!({
                "role": "assistant",
                "content": "Error: No LLM configured for this agent.",
            }));
        }
        Ok(())
    }

    async fn execute_tool_calls(&self, session: &mut Session) -> Result<(), Suspensions> {
        let active = &session.messages[session.cursor..];

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
                        session.messages.push(serde_json::json!({
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
                            session.messages.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tc_id,
                                "content": format!("Error: {}", e),
                            }));
                        }
                    }
                }
            } else {
                let available: Vec<&str> = tool_map.keys().map(|k| k.as_str()).collect();
                session.messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tc_id,
                    "content": format!("Error: Tool '{}' not found. Available: {:?}", name, available),
                }));
            }
        }

        if !suspensions.is_empty() {
            return Err(Suspensions::new("suspensions", suspensions));
        }

        Ok(())
    }
}
