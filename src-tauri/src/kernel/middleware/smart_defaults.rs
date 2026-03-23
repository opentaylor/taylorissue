use async_trait::async_trait;

use crate::kernel::agent::Agent;
use super::base::{AgentError, Middleware, Next};

const BASE_AGENT_PROMPT: &str = include_str!("../../prompts/kernel/base_agent.md");

pub struct SmartDefaultsMiddleware {
    prompt: String,
}

impl SmartDefaultsMiddleware {
    pub fn new() -> Self {
        Self {
            prompt: BASE_AGENT_PROMPT.to_string(),
        }
    }

    pub fn with_extra(mut self, extra: &str) -> Self {
        self.prompt.push_str("\n\n");
        self.prompt.push_str(extra);
        self
    }
}

impl Default for SmartDefaultsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for SmartDefaultsMiddleware {
    async fn wrap_llm(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        ctx.prefix_messages.insert(0, serde_json::json!({
            "role": "system",
            "content": self.prompt
        }));
        next.call(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_defaults_has_prompt() {
        let mw = SmartDefaultsMiddleware::new();
        assert!(mw.prompt.contains("autonomous agent"));
    }

    #[test]
    fn test_smart_defaults_with_extra() {
        let mw = SmartDefaultsMiddleware::new().with_extra("Always respond in JSON.");
        assert!(mw.prompt.contains("Always respond in JSON."));
    }

    #[test]
    fn test_smart_defaults_default() {
        let mw = SmartDefaultsMiddleware::default();
        assert!(mw.prompt.contains("autonomous agent"));
    }

    #[tokio::test]
    async fn test_smart_defaults_inserts_at_position_zero() {
        use crate::kernel::agent::Agent;
        use crate::kernel::middleware::base::{Next, Phase};

        let mw = SmartDefaultsMiddleware::new();
        let mut agent = Agent::new();
        agent.prefix_messages.push(serde_json::json!({"role": "system", "content": "existing"}));

        let mws: Vec<Box<dyn crate::kernel::middleware::base::Middleware>> = vec![];
        let next = Next { remaining: &mws, phase: Phase::Start };

        mw.wrap_llm(&mut agent, next).await.unwrap();

        assert_eq!(agent.prefix_messages.len(), 2);
        assert!(agent.prefix_messages[0]["content"].as_str().unwrap().contains("autonomous agent"));
        assert_eq!(agent.prefix_messages[1]["content"], "existing");
    }
}
