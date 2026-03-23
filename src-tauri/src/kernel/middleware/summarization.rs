use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

use crate::kernel::agent::Agent;
use crate::kernel::llm::token_counter::CharEstimateCounter;
use crate::kernel::middleware::context_window::fix_tool_boundaries;
use super::base::{AgentError, Middleware, Next};

pub struct SummarizationMiddleware {
    trigger_threshold: usize,
    keep_recent: usize,
    max_tool_result_tokens: usize,
}

impl SummarizationMiddleware {
    pub fn new() -> Self {
        Self {
            trigger_threshold: 100_000,
            keep_recent: 10,
            max_tool_result_tokens: 20_000,
        }
    }

    pub fn with_trigger_threshold(mut self, threshold: usize) -> Self {
        self.trigger_threshold = threshold;
        self
    }

    pub fn with_keep_recent(mut self, keep: usize) -> Self {
        self.keep_recent = keep;
        self
    }

    pub fn with_max_tool_result_tokens(mut self, max: usize) -> Self {
        self.max_tool_result_tokens = max;
        self
    }

    fn summarize_messages(messages: &[Value]) -> String {
        let mut parts = Vec::new();
        for msg in messages {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
            if !content.is_empty() {
                let truncated: String = content.chars().take(500).collect();
                parts.push(format!("[{}] {}", role, truncated));
            }
            if let Some(tcs) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                for tc in tcs {
                    let name = tc
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");
                    parts.push(format!("[{}] called tool: {}", role, name));
                }
            }
        }
        format!(
            "The following is a summary of the conversation so far:\n\n{}",
            parts.join("\n")
        )
    }

    fn save_history(agent_dir: &str, messages: &[Value]) {
        if agent_dir.is_empty() {
            return;
        }
        let path = PathBuf::from(agent_dir).join("conversation_history.md");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content: String = messages
            .iter()
            .map(|m| {
                let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
                let content = m.get("content").and_then(|v| v.as_str()).unwrap_or("");
                format!("## {}\n\n{}\n", role, content)
            })
            .collect::<Vec<_>>()
            .join("\n---\n\n");
        let _ = std::fs::write(path, content);
    }

    fn truncate_tool_result(
        &self,
        agent_dir: &str,
        content: &str,
        tool_call_id: &str,
    ) -> Option<String> {
        let token_count = CharEstimateCounter::count_text(content);
        if token_count <= self.max_tool_result_tokens {
            return None;
        }

        let preview_chars = self.max_tool_result_tokens * 4;
        let preview: String = content.chars().take(preview_chars).collect();

        if !agent_dir.is_empty() {
            let path = PathBuf::from(agent_dir).join(format!("tool_output_{}.txt", tool_call_id));
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, content);
            Some(format!(
                "{}\n\n... [truncated, full output saved to {}]",
                preview,
                path.display()
            ))
        } else {
            Some(format!("{}\n\n... [truncated]", preview))
        }
    }
}

impl Default for SummarizationMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for SummarizationMiddleware {
    async fn wrap_llm(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        let token_count = CharEstimateCounter::count_messages(&ctx.session.messages);

        if token_count > self.trigger_threshold && ctx.session.messages.len() > self.keep_recent {
            let split_at = ctx.session.messages.len().saturating_sub(self.keep_recent);
            let to_summarize = &ctx.session.messages[..split_at];

            Self::save_history(&ctx.agent_dir, &ctx.session.messages);

            let summary = Self::summarize_messages(to_summarize);
            let recent = ctx.session.messages[split_at..].to_vec();

            let mut new_messages = vec![serde_json::json!({
                "role": "system",
                "content": summary
            })];
            new_messages.extend(recent);
            ctx.session.messages = fix_tool_boundaries(&new_messages);
        }

        next.call(ctx).await
    }

    async fn wrap_tool(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        let result = next.call(ctx).await;

        let len = ctx.session.messages.len();
        for i in 0..len {
            let role = ctx.session.messages[i]
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if role != "tool" {
                continue;
            }
            let content = ctx.session.messages[i]
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tc_id = ctx.session.messages[i]
                .get("tool_call_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(truncated) = self.truncate_tool_result(&ctx.agent_dir, &content, &tc_id) {
                ctx.session.messages[i]["content"] = Value::String(truncated);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_messages() {
        let msgs = vec![
            serde_json::json!({"role": "user", "content": "hello"}),
            serde_json::json!({"role": "assistant", "content": "hi there"}),
        ];
        let summary = SummarizationMiddleware::summarize_messages(&msgs);
        assert!(summary.contains("[user] hello"));
        assert!(summary.contains("[assistant] hi there"));
    }

    #[test]
    fn test_truncate_short_content() {
        let mw = SummarizationMiddleware::new();
        assert!(mw.truncate_tool_result("", "short content", "tc1").is_none());
    }

    #[test]
    fn test_truncate_long_content() {
        let mw = SummarizationMiddleware::new().with_max_tool_result_tokens(10);
        let long = "a".repeat(1000);
        let result = mw.truncate_tool_result("", &long, "tc1");
        assert!(result.is_some());
        assert!(result.unwrap().contains("[truncated]"));
    }

    #[test]
    fn test_builder() {
        let mw = SummarizationMiddleware::new()
            .with_trigger_threshold(50_000)
            .with_keep_recent(5)
            .with_max_tool_result_tokens(10_000);
        assert_eq!(mw.trigger_threshold, 50_000);
        assert_eq!(mw.keep_recent, 5);
        assert_eq!(mw.max_tool_result_tokens, 10_000);
    }

    #[test]
    fn test_save_history_empty_agent_dir() {
        SummarizationMiddleware::save_history("", &[
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
    }

    #[test]
    fn test_save_history_writes_file() {
        let dir = tempfile::tempdir().unwrap();
        let agent_dir = dir.path().to_str().unwrap();
        let msgs = vec![
            serde_json::json!({"role": "user", "content": "hello"}),
            serde_json::json!({"role": "assistant", "content": "hi there"}),
        ];
        SummarizationMiddleware::save_history(agent_dir, &msgs);
        let path = dir.path().join("conversation_history.md");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("## user"));
        assert!(content.contains("hello"));
        assert!(content.contains("## assistant"));
    }

    #[test]
    fn test_truncate_tool_result_saves_to_file() {
        let dir = tempfile::tempdir().unwrap();
        let agent_dir = dir.path().to_str().unwrap();
        let mw = SummarizationMiddleware::new().with_max_tool_result_tokens(10);
        let long = "x".repeat(1000);
        let result = mw.truncate_tool_result(agent_dir, &long, "tc42");
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.contains("truncated, full output saved to"));
        let saved = dir.path().join("tool_output_tc42.txt");
        assert!(saved.exists());
        assert_eq!(std::fs::read_to_string(&saved).unwrap(), long);
    }

    #[test]
    fn test_summarize_messages_with_tool_calls() {
        let msgs = vec![
            serde_json::json!({
                "role": "assistant",
                "content": "",
                "tool_calls": [{"function": {"name": "bash"}}]
            }),
        ];
        let summary = SummarizationMiddleware::summarize_messages(&msgs);
        assert!(summary.contains("called tool: bash"));
    }
}
