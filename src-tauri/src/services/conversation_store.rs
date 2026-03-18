use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

const MAX_MESSAGES: usize = 500;

#[derive(Error, Debug)]
pub enum ConversationError {
    #[error("Invalid agent ID: {0}")]
    InvalidAgentId(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: f64,
}

pub fn validate_agent_id(agent_id: &str) -> Result<(), ConversationError> {
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !re.is_match(agent_id) {
        return Err(ConversationError::InvalidAgentId(agent_id.to_string()));
    }
    Ok(())
}

fn conversations_dir(workspace_path: &str) -> PathBuf {
    let ws = PathBuf::from(workspace_path);
    let parent = ws.parent().unwrap_or(&ws);
    parent.join("conversations")
}

fn parse_line(line: &str) -> Option<StoredMessage> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let obj: Value = serde_json::from_str(line).ok()?;
    let id = obj.get("id")?.as_str()?.to_string();
    if id.is_empty() {
        return None;
    }
    let role = obj.get("role")?.as_str()?;
    if role != "user" && role != "assistant" {
        return None;
    }
    let content = obj.get("content")?.as_str()?.to_string();
    let timestamp = obj
        .get("timestamp")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    Some(StoredMessage {
        id,
        role: role.to_string(),
        content,
        timestamp,
    })
}

pub fn get_messages(
    workspace_path: &str,
    agent_id: &str,
) -> Result<Vec<StoredMessage>, ConversationError> {
    validate_agent_id(agent_id)?;
    let file_path = conversations_dir(workspace_path).join(format!("{}.jsonl", agent_id));
    if !file_path.is_file() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&file_path)?;
    let mut messages: Vec<StoredMessage> = content
        .lines()
        .filter_map(parse_line)
        .collect();
    messages.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap_or(std::cmp::Ordering::Equal));
    if messages.len() > MAX_MESSAGES {
        messages = messages.split_off(messages.len() - MAX_MESSAGES);
    }
    Ok(messages)
}

pub fn append_messages(
    workspace_path: &str,
    agent_id: &str,
    messages: &[StoredMessage],
) -> Result<(), ConversationError> {
    validate_agent_id(agent_id)?;
    let conv_dir = conversations_dir(workspace_path);
    std::fs::create_dir_all(&conv_dir)?;
    let file_path = conv_dir.join(format!("{}.jsonl", agent_id));

    let existing_ids: HashSet<String> = if file_path.is_file() {
        get_messages(workspace_path, agent_id)?
            .into_iter()
            .map(|m| m.id)
            .collect()
    } else {
        HashSet::new()
    };

    let new_messages: Vec<&StoredMessage> = messages
        .iter()
        .filter(|m| !existing_ids.contains(&m.id))
        .collect();

    if new_messages.is_empty() {
        return Ok(());
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)?;

    for msg in new_messages {
        let line = serde_json::json!({
            "id": msg.id,
            "role": msg.role,
            "content": msg.content,
            "timestamp": msg.timestamp,
        });
        writeln!(file, "{}", serde_json::to_string(&line).unwrap_or_default())?;
    }
    Ok(())
}

pub fn clear_conversation(
    workspace_path: &str,
    agent_id: &str,
) -> Result<(), ConversationError> {
    validate_agent_id(agent_id)?;
    let file_path = conversations_dir(workspace_path).join(format!("{}.jsonl", agent_id));
    match std::fs::remove_file(&file_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ConversationError::Io(e)),
    }
}

pub fn list_agent_ids(workspace_path: &str) -> Vec<String> {
    let conv_dir = conversations_dir(workspace_path);
    if !conv_dir.is_dir() {
        return Vec::new();
    }
    let mut ids: Vec<String> = std::fs::read_dir(&conv_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.strip_suffix(".jsonl").map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    ids.sort();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_agent_id_valid() {
        assert!(validate_agent_id("main").is_ok());
        assert!(validate_agent_id("agent-1").is_ok());
        assert!(validate_agent_id("agent_2").is_ok());
    }

    #[test]
    fn test_validate_agent_id_invalid() {
        assert!(validate_agent_id("../etc").is_err());
        assert!(validate_agent_id("").is_err());
        assert!(validate_agent_id("a b").is_err());
    }

    #[test]
    fn test_parse_line_valid() {
        let line = r#"{"id":"m1","role":"user","content":"hello","timestamp":1000}"#;
        let msg = parse_line(line).unwrap();
        assert_eq!(msg.id, "m1");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn test_parse_line_invalid_role() {
        let line = r#"{"id":"m1","role":"system","content":"hello","timestamp":1000}"#;
        assert!(parse_line(line).is_none());
    }

    #[test]
    fn test_parse_line_missing_id() {
        let line = r#"{"role":"user","content":"hello"}"#;
        assert!(parse_line(line).is_none());
    }
}
