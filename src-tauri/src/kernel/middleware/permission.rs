use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::kernel::agent::{Agent, Suspension, Suspensions};
use super::base::Middleware;

pub struct PermissionMiddleware {
    pub key: String,
}

impl PermissionMiddleware {
    pub fn new() -> Self {
        Self {
            key: "permission:decisions".to_string(),
        }
    }
}

impl Default for PermissionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

pub fn apply_decision(
    messages: &mut Vec<Value>,
    tool_call: &Value,
    resume_item: &Value,
) -> Result<(), String> {
    let dtype = resume_item
        .get("decision")
        .and_then(|v| v.as_str())
        .unwrap_or("approve");

    let tc_id = tool_call
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match dtype {
        "approve" => Ok(()),
        "reject" => {
            let message = resume_item
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Tool call rejected by user.");
            messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tc_id,
                "content": message,
            }));
            Ok(())
        }
        "edit" => {
            if let Some(func) = tool_call.get("function") {
                let args_raw = func
                    .get("arguments")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let mut args: Value =
                    serde_json::from_str(args_raw).unwrap_or(serde_json::json!({}));
                if let Some(new_args) = resume_item.get("args") {
                    if let (Some(base), Some(patch)) = (args.as_object_mut(), new_args.as_object()) {
                        for (k, v) in patch {
                            base.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            Ok(())
        }
        other => Err(format!(
            "Unknown permission decision '{}' for tool call '{}'. Expected one of [approve, edit, reject].",
            other, tc_id
        )),
    }
}

#[async_trait]
impl Middleware for PermissionMiddleware {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_decision_reject() {
        let mut messages = Vec::new();
        let tc = serde_json::json!({"id": "tc1", "function": {"name": "bash", "arguments": "{}"}});
        let decision = serde_json::json!({"decision": "reject", "message": "No way"});
        apply_decision(&mut messages, &tc, &decision).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["content"], "No way");
    }

    #[test]
    fn test_apply_decision_approve() {
        let mut messages = Vec::new();
        let tc = serde_json::json!({"id": "tc1", "function": {"name": "bash", "arguments": "{}"}});
        let decision = serde_json::json!({"decision": "approve"});
        apply_decision(&mut messages, &tc, &decision).unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_apply_decision_invalid() {
        let mut messages = Vec::new();
        let tc = serde_json::json!({"id": "tc1"});
        let decision = serde_json::json!({"decision": "invalid"});
        let result = apply_decision(&mut messages, &tc, &decision);
        assert!(result.is_err());
    }
}
