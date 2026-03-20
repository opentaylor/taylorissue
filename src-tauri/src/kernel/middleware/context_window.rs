use async_trait::async_trait;
use serde_json::Value;

use super::base::Middleware;

pub struct ContextWindowMiddleware {
    pub max_tokens: usize,
    pub model: String,
}

impl ContextWindowMiddleware {
    pub fn new(max_tokens: usize, model: &str) -> Self {
        Self {
            max_tokens,
            model: model.to_string(),
        }
    }
}

pub fn fix_tool_boundaries(messages: &[Value]) -> Vec<Value> {
    let mut result: Vec<Value> = Vec::new();
    let mut required_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for msg in messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
        if role == "assistant" {
            if let Some(tcs) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                for tc in tcs {
                    if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                        required_ids.insert(id.to_string());
                    }
                }
            }
        }
    }

    for msg in messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
        if role == "tool" {
            let tc_id = msg
                .get("tool_call_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !required_ids.contains(tc_id) {
                continue;
            }
        }
        result.push(msg.clone());
    }
    result
}

#[async_trait]
impl Middleware for ContextWindowMiddleware {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_tool_boundaries_keeps_matching() {
        let messages = vec![
            serde_json::json!({
                "role": "assistant",
                "content": "",
                "tool_calls": [{"id": "tc1", "function": {"name": "test"}}]
            }),
            serde_json::json!({"role": "tool", "tool_call_id": "tc1", "content": "result"}),
        ];
        let result = fix_tool_boundaries(&messages);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_fix_tool_boundaries_removes_orphan() {
        let messages = vec![
            serde_json::json!({"role": "tool", "tool_call_id": "tc_orphan", "content": "result"}),
            serde_json::json!({"role": "user", "content": "hi"}),
        ];
        let result = fix_tool_boundaries(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "user");
    }
}
