use serde_json::Value;
use tauri::ipc::Channel;

use crate::kernel::agent::{Agent, Session};

pub fn extract_json(text: &str) -> Option<Value> {
    let brace = text.find('{')?;
    let mut depth = 0i32;
    let mut end = None;
    for (i, ch) in text[brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(brace + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    serde_json::from_str(&text[brace..end]).ok()
}

pub struct StepDef {
    pub id: &'static str,
    pub prompt: &'static str,
}

pub type DetailsFn = fn(&str, &Value) -> Vec<String>;
pub type HasIssueFn = fn(&str, &Value) -> bool;

pub async fn run_step(
    agent: &mut Agent,
    session: &mut Session,
    step: &StepDef,
    channel: &Channel<Value>,
    max_retries: usize,
    details_fn: Option<DetailsFn>,
    has_issue_fn: Option<HasIssueFn>,
) -> bool {
    run_step_dynamic(agent, session, step.id, step.prompt, channel, max_retries, details_fn, has_issue_fn).await
}

pub async fn run_step_dynamic(
    agent: &mut Agent,
    session: &mut Session,
    step_id: &str,
    prompt: &str,
    channel: &Channel<Value>,
    max_retries: usize,
    details_fn: Option<DetailsFn>,
    has_issue_fn: Option<HasIssueFn>,
) -> bool {
    let _ = channel.send(serde_json::json!({
        "event": "step",
        "data": {"step_id": step_id, "status": "active"}
    }));

    let mut last_error = String::new();
    let mut is_first_attempt = true;

    for attempt in 0..max_retries {
        if is_first_attempt {
            session.messages.push(serde_json::json!({
                "role": "user",
                "content": prompt
            }));
            is_first_attempt = false;
        }

        agent.session = session.clone();
        let before = session.messages.len();
        let result = agent.run().await;

        match result {
            Ok(()) => {
                *session = agent.session.clone();
            }
            Err(e) => {
                *session = agent.session.clone();
                last_error = e.to_string();
                session.messages.push(serde_json::json!({
                    "role": "user",
                    "content": format!(
                        "The previous attempt raised an exception:\n{}\n\n\
                         Analyse the error, fix the root cause, and try again. \
                         Respond with the same JSON schema.",
                        last_error
                    )
                }));
                continue;
            }
        }

        let text = session.messages[before..]
            .iter()
            .rev()
            .find(|m| {
                m.get("role").and_then(|v| v.as_str()) == Some("assistant")
                    && m.get("content")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false)
                    && m.get("tool_calls").is_none()
            })
            .and_then(|m| m.get("content").and_then(|v| v.as_str()))
            .unwrap_or("");

        if text.is_empty() || text.starts_with("LLM call failed:") || text.starts_with("Error:") {
            last_error = if text.is_empty() {
                "Agent produced no response (LLM call may have failed — check API key and endpoint)".to_string()
            } else {
                text.to_string()
            };
            log::error!("Step '{}': {}", step_id, last_error);
            break;
        }

        let parsed = extract_json(text);

        if parsed.is_none() {
            last_error = format!("Agent response was not valid JSON: {}", &text[..text.len().min(200)]);
            log::warn!("Step '{}': {}", step_id, last_error);
            if attempt < max_retries - 1 {
                session.messages.push(serde_json::json!({
                    "role": "user",
                    "content": format!(
                        "Your response was not valid JSON. Here is what you said:\n{}\n\n\
                         Please respond with ONLY a valid JSON object matching the required schema.",
                        text
                    )
                }));
                continue;
            }
            break;
        }

        if let Some(ref p) = parsed {
            if p.get("success").and_then(|v| v.as_bool()) == Some(false) {
                last_error = p
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Step failed")
                    .to_string();
                if attempt < max_retries - 1 {
                    session.messages.push(serde_json::json!({
                        "role": "user",
                        "content": format!(
                            "The previous attempt failed: {}\n\n\
                             Analyse the error, fix the root cause, and try again. \
                             Respond with the same JSON schema.",
                            last_error
                        )
                    }));
                    continue;
                }
                break;
            }
        }

        let details = if let Some(ref p) = parsed {
            details_fn.map(|f| f(step_id, p)).unwrap_or_else(|| vec!["OK".to_string()])
        } else {
            vec!["OK".to_string()]
        };

        let mut signal_data = serde_json::json!({
            "step_id": step_id,
            "status": "complete",
            "details": details,
        });

        if let (Some(ref p), Some(issue_fn)) = (&parsed, has_issue_fn) {
            signal_data["has_issue"] = Value::Bool(issue_fn(step_id, p));
        }

        let _ = channel.send(serde_json::json!({"event": "step", "data": signal_data}));
        return true;
    }

    let _ = channel.send(serde_json::json!({
        "event": "step",
        "data": {
            "step_id": step_id,
            "status": "error",
            "error": if last_error.is_empty() {
                "Step failed after retries".to_string()
            } else {
                last_error
            },
        }
    }));
    false
}

pub async fn run_step_standalone(
    agent: &mut Agent,
    session: &mut Session,
    step_id: &str,
    prompt: &str,
    max_retries: usize,
) -> Result<Value, String> {
    let mut last_error = String::new();
    let mut is_first_attempt = true;

    for attempt in 0..max_retries {
        if is_first_attempt {
            session.messages.push(serde_json::json!({
                "role": "user",
                "content": prompt
            }));
            is_first_attempt = false;
        }

        agent.session = session.clone();
        let before = session.messages.len();
        let result = agent.run().await;

        match result {
            Ok(()) => {
                *session = agent.session.clone();
            }
            Err(e) => {
                *session = agent.session.clone();
                last_error = e.to_string();
                session.messages.push(serde_json::json!({
                    "role": "user",
                    "content": format!(
                        "The previous attempt raised an exception:\n{}\n\n\
                         Analyse the error, fix the root cause, and try again. \
                         Respond with the same JSON schema.",
                        last_error
                    )
                }));
                continue;
            }
        }

        let text = session.messages[before..]
            .iter()
            .rev()
            .find(|m| {
                m.get("role").and_then(|v| v.as_str()) == Some("assistant")
                    && m.get("content")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.is_empty())
                        .unwrap_or(false)
                    && m.get("tool_calls").is_none()
            })
            .and_then(|m| m.get("content").and_then(|v| v.as_str()))
            .unwrap_or("");

        if text.is_empty() || text.starts_with("LLM call failed:") || text.starts_with("Error:") {
            last_error = if text.is_empty() {
                "Agent produced no response (LLM call may have failed — check API key and endpoint)".to_string()
            } else {
                text.to_string()
            };
            log::error!("Step '{}': {}", step_id, last_error);
            break;
        }

        let parsed = extract_json(text);

        if parsed.is_none() {
            last_error = format!("Agent response was not valid JSON: {}", &text[..text.len().min(200)]);
            log::warn!("Step '{}': {}", step_id, last_error);
            if attempt < max_retries - 1 {
                session.messages.push(serde_json::json!({
                    "role": "user",
                    "content": format!(
                        "Your response was not valid JSON. Here is what you said:\n{}\n\n\
                         Please respond with ONLY a valid JSON object matching the required schema.",
                        text
                    )
                }));
                continue;
            }
            break;
        }

        if let Some(ref p) = parsed {
            if p.get("success").and_then(|v| v.as_bool()) == Some(false) {
                last_error = p
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Step failed")
                    .to_string();
                if attempt < max_retries - 1 {
                    session.messages.push(serde_json::json!({
                        "role": "user",
                        "content": format!(
                            "The previous attempt failed: {}\n\n\
                             Analyse the error, fix the root cause, and try again. \
                             Respond with the same JSON schema.",
                            last_error
                        )
                    }));
                    continue;
                }
                break;
            }
        }

        return Ok(parsed.unwrap());
    }

    Err(if last_error.is_empty() {
        format!("Step '{}' failed after retries", step_id)
    } else {
        format!("Step '{}': {}", step_id, last_error)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_valid() {
        let result = extract_json(r#"{"key": "value"}"#);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["key"], "value");
    }

    #[test]
    fn test_extract_json_embedded_in_text() {
        let result = extract_json(r#"Here is the result: {"success": true} and more text"#);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["success"], true);
    }

    #[test]
    fn test_extract_json_nested_braces() {
        let result = extract_json(r#"{"outer": {"inner": 42}}"#);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["outer"]["inner"], 42);
    }

    #[test]
    fn test_extract_json_invalid() {
        let result = extract_json(r#"{not valid json}"#);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_json_no_braces() {
        let result = extract_json("no json here");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_json_empty_string() {
        let result = extract_json("");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_json_first_of_multiple() {
        let result = extract_json(r#"{"a": 1} {"b": 2}"#);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["a"], 1);
    }

    #[test]
    fn test_extract_json_empty_object() {
        let result = extract_json("{}");
        assert!(result.is_some());
        assert!(result.unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn test_extract_json_with_array_value() {
        let result = extract_json(r#"result: {"items": [1, 2, 3]}"#);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_extract_json_braces_in_string() {
        let result = extract_json(r#"{"msg": "use {x} here"}"#);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["msg"], "use {x} here");
    }
}
