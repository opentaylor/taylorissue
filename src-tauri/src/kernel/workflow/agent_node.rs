use std::collections::HashMap;
use serde_json::Value;

use crate::kernel::agent::{Agent, Session};
use super::workflow::Node;

pub struct AgentNode {
    node_id: String,
    node_name: String,
    pub agent: Agent,
    pub response_format: Option<Value>,
    suspended_session: Option<Session>,
    last_prep_result: Option<String>,
    prep_fn: Box<dyn Fn(&HashMap<String, Value>) -> String + Send + Sync>,
    post_fn: Box<dyn Fn(&mut HashMap<String, Value>, &str, &str) -> Option<String> + Send + Sync>,
}

impl AgentNode {
    pub fn new(
        id: &str,
        name: &str,
        agent: Agent,
        prep_fn: Box<dyn Fn(&HashMap<String, Value>) -> String + Send + Sync>,
        post_fn: Box<dyn Fn(&mut HashMap<String, Value>, &str, &str) -> Option<String> + Send + Sync>,
    ) -> Self {
        Self {
            node_id: id.to_string(),
            node_name: name.to_string(),
            agent,
            response_format: None,
            suspended_session: None,
            last_prep_result: None,
            prep_fn,
            post_fn,
        }
    }

    async fn exec(
        &mut self,
        prep_result: &str,
        resume: Option<Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let session = if resume.is_some() {
            self.suspended_session.take().unwrap_or_else(|| {
                Session::with_messages(vec![serde_json::json!({
                    "role": "user",
                    "content": prep_result
                })])
            })
        } else {
            Session::with_messages(vec![serde_json::json!({
                "role": "user",
                "content": prep_result
            })])
        };

        self.suspended_session = Some(session.clone());

        let mut kwargs = HashMap::new();
        kwargs.insert(
            "thread_id".to_string(),
            Value::String(format!("wf:{}", self.node_id)),
        );
        if let Some(r) = resume {
            kwargs.insert("resume".to_string(), r);
        }

        let result_session = self
            .agent
            .run(session, self.response_format.clone(), kwargs)
            .await;

        match result_session {
            Ok(session) => {
                self.suspended_session = None;
                for m in session.messages.iter().rev() {
                    if m.get("role").and_then(|v| v.as_str()) == Some("assistant") {
                        if let Some(parsed) = m.get("parsed") {
                            return Ok(parsed.to_string());
                        }
                        if let Some(content) = m.get("content").and_then(|v| v.as_str()) {
                            if !content.is_empty() {
                                return Ok(content.to_string());
                            }
                        }
                    }
                }
                Ok(String::new())
            }
            Err(suspensions) => {
                Err(Box::new(suspensions))
            }
        }
    }
}

#[async_trait::async_trait]
impl Node for AgentNode {
    fn id(&self) -> &str {
        &self.node_id
    }
    fn name(&self) -> &str {
        &self.node_name
    }

    async fn run(
        &mut self,
        context: &mut HashMap<String, Value>,
        resume: Option<Value>,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let prep_result = if resume.is_some() {
            self.last_prep_result.clone().unwrap_or_default()
        } else {
            let result = (self.prep_fn)(context);
            self.last_prep_result = Some(result.clone());
            result
        };

        let exec_result = self.exec(&prep_result, resume).await?;
        Ok((self.post_fn)(context, &prep_result, &exec_result))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_agent_node_compiles() {
        // AgentNode requires a full Agent which needs LLM; compile-time check only
        assert!(true);
    }
}
