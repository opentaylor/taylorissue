use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

use crate::kernel::agent::{Agent, Session};
use crate::kernel::util::store::JsonlFile;
use super::base::{AgentError, Middleware, Next};

pub struct CheckpointMiddleware {
    pub base_dir: PathBuf,
}

impl CheckpointMiddleware {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn checkpoint_path(&self, thread_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.jsonl", thread_id))
    }
}

pub fn sanitize_for_persistence(msg: &Value) -> Value {
    let mut clean = msg.clone();
    if let Some(obj) = clean.as_object_mut() {
        obj.remove("_media");
        if let Some(content) = obj.get("content") {
            if let Some(arr) = content.as_array() {
                let filtered: Vec<Value> = arr
                    .iter()
                    .filter(|part| {
                        part.get("type").and_then(|v| v.as_str()) != Some("image_url")
                    })
                    .cloned()
                    .collect();
                if filtered.is_empty() {
                    obj.insert("content".to_string(), Value::String(String::new()));
                } else {
                    obj.insert("content".to_string(), Value::Array(filtered));
                }
            }
        }
    }
    clean
}

pub fn load_session_jsonl(path: &PathBuf) -> Session {
    let file = JsonlFile::new(path.clone());
    let entries = file.read_all();
    let messages: Vec<Value> = entries
        .into_iter()
        .filter_map(|v| {
            if v.get("role").is_some() {
                Some(v)
            } else {
                None
            }
        })
        .collect();
    Session::with_messages(messages)
}

#[async_trait]
impl Middleware for CheckpointMiddleware {
    async fn wrap_start(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        let thread_id = ctx
            .metadata
            .get("thread_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        let path = self.checkpoint_path(&thread_id);
        if path.exists() && ctx.session.messages.is_empty() {
            let loaded = load_session_jsonl(&path);
            ctx.session.messages = loaded.messages;
        }
        next.call(ctx).await
    }

    async fn wrap_end(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        let result = next.call(ctx).await;

        let thread_id = ctx
            .metadata
            .get("thread_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        let path = self.checkpoint_path(&thread_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = JsonlFile::new(path);
        file.clear();
        for msg in &ctx.session.messages {
            file.append(&sanitize_for_persistence(msg));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_for_persistence_removes_media() {
        let msg = serde_json::json!({
            "role": "tool",
            "content": "result",
            "_media": [{"type": "image"}]
        });
        let clean = sanitize_for_persistence(&msg);
        assert!(clean.get("_media").is_none());
    }

    #[test]
    fn test_sanitize_for_persistence_strips_images() {
        let msg = serde_json::json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "look"},
                {"type": "image_url", "image_url": {"url": "data:image/png;base64,abc"}}
            ]
        });
        let clean = sanitize_for_persistence(&msg);
        let content = clean.get("content").unwrap().as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
    }

    #[test]
    fn test_load_session_jsonl_missing_file() {
        let path = PathBuf::from("/tmp/nonexistent_test_session.jsonl");
        let session = load_session_jsonl(&path);
        assert!(session.messages.is_empty());
    }
}
