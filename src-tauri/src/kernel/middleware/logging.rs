use async_trait::async_trait;
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::kernel::agent::Agent;
use super::base::Middleware;

pub struct LoggingMiddleware {
    pub label: String,
    checkpoint: AtomicUsize,
}

impl LoggingMiddleware {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            checkpoint: AtomicUsize::new(0),
        }
    }

    fn get_checkpoint(&self) -> usize {
        self.checkpoint.load(Ordering::Relaxed)
    }

    fn set_checkpoint(&self, val: usize) {
        self.checkpoint.store(val, Ordering::Relaxed);
    }
}

pub fn extract_text(content: &Value) -> String {
    match content {
        Value::String(s) => {
            let truncated: String = s.chars().take(200).collect();
            if truncated.len() < s.len() {
                format!("{}...", truncated)
            } else {
                s.clone()
            }
        }
        Value::Array(arr) => {
            arr.iter()
                .filter_map(|p| {
                    if p.get("type").and_then(|v| v.as_str()) == Some("text") {
                        p.get("text").and_then(|v| v.as_str()).map(String::from)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        _ => String::new(),
    }
}

struct Node {
    label: String,
    children: Vec<Node>,
}

impl Node {
    fn new(label: &str) -> Self {
        Self { label: label.to_string(), children: Vec::new() }
    }

    fn with_children(label: &str, children: Vec<Node>) -> Self {
        Self { label: label.to_string(), children }
    }

    fn render(&self, prefix: &str, is_last: bool, is_root: bool) -> Vec<String> {
        let label_lines: Vec<&str> = self.label.split('\n').collect();
        let mut lines = Vec::new();

        if is_root {
            lines.extend(label_lines.iter().map(|l| l.to_string()));
        } else {
            let connector = if is_last { "└── " } else { "├── " };
            let cont = if is_last { "    " } else { "│   " };
            lines.push(format!("{prefix}{connector}{}", label_lines[0]));
            for extra in &label_lines[1..] {
                lines.push(format!("{prefix}{cont}{extra}"));
            }
        }

        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        for (i, child) in self.children.iter().enumerate() {
            lines.extend(child.render(&child_prefix, i == self.children.len() - 1, false));
        }
        lines
    }

    fn to_string_tree(&self) -> String {
        self.render("", true, true).join("\n")
    }
}

fn build_tool_call_map(messages: &[Value]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for msg in messages {
        if msg.get("role").and_then(|v| v.as_str()) == Some("assistant") {
            if let Some(tcs) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                for tc in tcs {
                    if let (Some(id), Some(name)) = (
                        tc.get("id").and_then(|v| v.as_str()),
                        tc.get("function").and_then(|f| f.get("name").and_then(|n| n.as_str())),
                    ) {
                        map.insert(id.to_string(), name.to_string());
                    }
                }
            }
        }
    }
    map
}

fn log_user_message(label: &str, msg: &Value) {
    let text = extract_text(msg.get("content").unwrap_or(&Value::Null));
    let tree = Node::with_children("User", vec![Node::new(&text)]);
    log::info!("[{}]\n{}", label, tree.to_string_tree());
}

fn log_tool_result(label: &str, msg: &Value, tool_name: Option<&str>) {
    let title = match tool_name {
        Some(name) => format!("Tool Result ({name})"),
        None => "Tool Result".to_string(),
    };
    let text = extract_text(msg.get("content").unwrap_or(&Value::Null));
    let tc_id = msg.get("tool_call_id").and_then(|v| v.as_str()).unwrap_or("");
    let mut children = vec![Node::new(&format!("ID: {tc_id}"))];
    if !text.is_empty() {
        let truncated = { let t: String = text.chars().take(500).collect(); if t.len() < text.len() { format!("{}...", t) } else { text } };
        children.push(Node::with_children("Output:", vec![Node::new(&truncated)]));
    }
    let tree = Node::with_children(&title, children);
    log::info!("[{}]\n{}", label, tree.to_string_tree());
}

fn log_assistant_message(label: &str, agent_name: &str, model: &str, msg: &Value) {
    let has_tool_calls = msg.get("tool_calls")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    let stop_reason = if has_tool_calls { "tool_use" } else { "end_turn" };

    let mut block_nodes = Vec::new();

    let text = extract_text(msg.get("content").unwrap_or(&Value::Null));
    if !text.is_empty() {
        block_nodes.push(Node::with_children(
            &format!("Block {}", block_nodes.len() + 1),
            vec![Node::with_children("Text", vec![Node::new(&text)])],
        ));
    }

    if let Some(tcs) = msg.get("tool_calls").and_then(|v| v.as_array()) {
        for tc in tcs {
            let func = tc.get("function").cloned().unwrap_or(Value::Null);
            let name = func.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let tc_id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let args_raw = func.get("arguments").cloned().unwrap_or(Value::String("{}".to_string()));
            let args: Value = if let Some(s) = args_raw.as_str() {
                serde_json::from_str(s).unwrap_or(Value::Object(serde_json::Map::new()))
            } else {
                args_raw
            };
            let input_json = serde_json::to_string_pretty(&args).unwrap_or_default();

            block_nodes.push(Node::with_children(
                &format!("Block {}", block_nodes.len() + 1),
                vec![Node::with_children(
                    &format!("Tool Use: {name}"),
                    vec![
                        Node::new(&format!("ID: {tc_id}")),
                        Node::with_children("Input:", vec![Node::new(&input_json)]),
                    ],
                )],
            ));
        }
    }

    let n = block_nodes.len();
    let content_label = format!("Content ({n} block{})", if n != 1 { "s" } else { "" });

    let tree = Node::with_children(
        &format!("Agent ({agent_name})"),
        vec![
            Node::new(&format!("Model: {model}")),
            Node::new(&format!("Stop Reason: {stop_reason}")),
            Node::with_children(&content_label, block_nodes),
        ],
    );
    log::info!("[{}]\n{}", label, tree.to_string_tree());
}

/// Log all new messages in the range [start..end) dispatching by role.
fn log_new_messages(label: &str, agent_name: &str, model: &str, messages: &[Value], start: usize, end: usize) {
    let tool_call_map = build_tool_call_map(&messages[..start]);

    for i in start..end {
        let entry = &messages[i];
        let role = entry.get("role").and_then(|v| v.as_str()).unwrap_or("");
        match role {
            "user" => log_user_message(label, entry),
            "tool" => {
                let tc_id = entry.get("tool_call_id").and_then(|v| v.as_str()).unwrap_or("");
                let tool_name = tool_call_map.get(tc_id).map(|s| s.as_str());
                log_tool_result(label, entry, tool_name);
            }
            "assistant" => log_assistant_message(label, agent_name, model, entry),
            _ => {}
        }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    /// Before LLM call: log new user messages and tool results since last checkpoint.
    async fn wrap_llm(&self, agent: &mut Agent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session = match &agent.session {
            Some(s) => s,
            None => return Ok(()),
        };
        let start = self.get_checkpoint();
        let end = session.messages.len();
        let model = agent.llm.as_ref().map(|l| l.model()).unwrap_or("unknown");

        log_new_messages(&self.label, &agent.name, model, &session.messages, start, end);
        self.set_checkpoint(end);
        Ok(())
    }

    /// Before tool execution: log the assistant message (with tool_calls) that was
    /// just produced by call_llm.
    async fn wrap_tool(&self, agent: &mut Agent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session = match &agent.session {
            Some(s) => s,
            None => return Ok(()),
        };
        let start = self.get_checkpoint();
        let end = session.messages.len();
        let model = agent.llm.as_ref().map(|l| l.model()).unwrap_or("unknown");

        log_new_messages(&self.label, &agent.name, model, &session.messages, start, end);
        self.set_checkpoint(end);
        Ok(())
    }

    /// After agent run completes: log any remaining messages (final assistant
    /// response and tool results from the last iteration).
    async fn wrap_end(&self, agent: &mut Agent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session = match &agent.session {
            Some(s) => s,
            None => return Ok(()),
        };
        let start = self.get_checkpoint();
        let end = session.messages.len();
        let model = agent.llm.as_ref().map(|l| l.model()).unwrap_or("unknown");

        log_new_messages(&self.label, &agent.name, model, &session.messages, start, end);
        self.set_checkpoint(end);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_string() {
        let content = serde_json::json!("hello world");
        assert_eq!(extract_text(&content), "hello world");
    }

    #[test]
    fn test_extract_text_truncates() {
        let long = "a".repeat(300);
        let content = Value::String(long);
        let result = extract_text(&content);
        assert!(result.len() < 210);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_extract_text_array() {
        let content = serde_json::json!([
            {"type": "text", "text": "hello"},
            {"type": "image_url", "url": "..."},
            {"type": "text", "text": "world"}
        ]);
        assert_eq!(extract_text(&content), "hello world");
    }

    #[test]
    fn test_node_render() {
        let tree = Node::with_children("Root", vec![
            Node::new("Child 1"),
            Node::with_children("Child 2", vec![Node::new("Grandchild")]),
        ]);
        let output = tree.to_string_tree();
        assert!(output.contains("Root"));
        assert!(output.contains("Child 1"));
        assert!(output.contains("Grandchild"));
    }
}
