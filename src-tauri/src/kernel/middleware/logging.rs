use async_trait::async_trait;

use crate::kernel::agent::Agent;
use super::base::Middleware;

pub struct LoggingMiddleware {
    pub label: String,
}

impl LoggingMiddleware {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }
}

pub fn extract_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => {
            if s.len() > 200 {
                format!("{}...", &s[..200])
            } else {
                s.clone()
            }
        }
        serde_json::Value::Array(arr) => {
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

#[async_trait]
impl Middleware for LoggingMiddleware {}

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
        let content = serde_json::Value::String(long);
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
}
