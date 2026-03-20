use async_trait::async_trait;
use serde_json::Value;

use super::base::Middleware;

pub struct CompactMiddleware {
    pub max_messages: usize,
}

impl CompactMiddleware {
    pub fn new(max_messages: usize) -> Self {
        Self { max_messages }
    }
}

pub fn content_to_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            arr.iter()
                .filter_map(|part| {
                    if part.get("type").and_then(|v| v.as_str()) == Some("text") {
                        part.get("text").and_then(|v| v.as_str()).map(String::from)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn serialize_messages(messages: &[Value]) -> String {
    messages
        .iter()
        .map(|m| {
            let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
            let content = content_to_text(m.get("content").unwrap_or(&Value::Null));
            format!("[{}] {}", role, content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[async_trait]
impl Middleware for CompactMiddleware {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_to_text_string() {
        assert_eq!(content_to_text(&Value::String("hello".to_string())), "hello");
    }

    #[test]
    fn test_content_to_text_array() {
        let content = serde_json::json!([
            {"type": "text", "text": "part1"},
            {"type": "text", "text": "part2"}
        ]);
        assert_eq!(content_to_text(&content), "part1\npart2");
    }

    #[test]
    fn test_serialize_messages() {
        let msgs = vec![
            serde_json::json!({"role": "user", "content": "hello"}),
            serde_json::json!({"role": "assistant", "content": "hi"}),
        ];
        let result = serialize_messages(&msgs);
        assert!(result.contains("[user] hello"));
        assert!(result.contains("[assistant] hi"));
    }
}
