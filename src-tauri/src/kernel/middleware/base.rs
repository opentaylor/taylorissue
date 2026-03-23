use async_trait::async_trait;
use crate::kernel::agent::Agent;

pub type AgentError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Copy)]
pub enum Phase {
    Start,
    Llm,
    Tool,
    End,
}

pub struct Next<'a> {
    pub remaining: &'a [Box<dyn Middleware>],
    pub phase: Phase,
}

impl<'a> Next<'a> {
    pub async fn call(self, ctx: &mut Agent) -> Result<(), AgentError> {
        if let Some((first, rest)) = self.remaining.split_first() {
            let next = Next { remaining: rest, phase: self.phase };
            match self.phase {
                Phase::Start => first.wrap_start(ctx, next).await,
                Phase::Llm => first.wrap_llm(ctx, next).await,
                Phase::Tool => first.wrap_tool(ctx, next).await,
                Phase::End => first.wrap_end(ctx, next).await,
            }
        } else {
            match self.phase {
                Phase::Start => Ok(()),
                Phase::Llm => ctx.call_llm().await,
                Phase::Tool => ctx.execute_tool_calls().await,
                Phase::End => Ok(()),
            }
        }
    }
}

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn wrap_start(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        next.call(ctx).await
    }

    async fn wrap_llm(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        next.call(ctx).await
    }

    async fn wrap_tool(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        next.call(ctx).await
    }

    async fn wrap_end(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        next.call(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct NoopMiddleware;

    #[async_trait]
    impl Middleware for NoopMiddleware {}

    #[test]
    fn test_noop_middleware_compiles() {
        let _mw = NoopMiddleware;
    }

    struct TrackingMiddleware {
        label: String,
        log: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Middleware for TrackingMiddleware {
        async fn wrap_start(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
            self.log.lock().unwrap().push(format!("{}_start_before", self.label));
            let r = next.call(ctx).await;
            self.log.lock().unwrap().push(format!("{}_start_after", self.label));
            r
        }

        async fn wrap_llm(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
            self.log.lock().unwrap().push(format!("{}_llm_before", self.label));
            let r = next.call(ctx).await;
            self.log.lock().unwrap().push(format!("{}_llm_after", self.label));
            r
        }

        async fn wrap_tool(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
            self.log.lock().unwrap().push(format!("{}_tool_before", self.label));
            let r = next.call(ctx).await;
            self.log.lock().unwrap().push(format!("{}_tool_after", self.label));
            r
        }

        async fn wrap_end(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
            self.log.lock().unwrap().push(format!("{}_end_before", self.label));
            let r = next.call(ctx).await;
            self.log.lock().unwrap().push(format!("{}_end_after", self.label));
            r
        }
    }

    struct ErrorMiddleware;

    #[async_trait]
    impl Middleware for ErrorMiddleware {
        async fn wrap_llm(&self, _ctx: &mut Agent, _next: Next<'_>) -> Result<(), AgentError> {
            Err("middleware error".into())
        }
    }

    #[tokio::test]
    async fn test_next_no_middleware_start() {
        let mut agent = Agent::new();
        let mws: Vec<Box<dyn Middleware>> = vec![];
        let result = Next { remaining: &mws, phase: Phase::Start }.call(&mut agent).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_next_no_middleware_end() {
        let mut agent = Agent::new();
        let mws: Vec<Box<dyn Middleware>> = vec![];
        let result = Next { remaining: &mws, phase: Phase::End }.call(&mut agent).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_next_noop_passthrough() {
        let mut agent = Agent::new();
        let mws: Vec<Box<dyn Middleware>> = vec![Box::new(NoopMiddleware)];
        let result = Next { remaining: &mws, phase: Phase::Start }.call(&mut agent).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_chain_ordering_two_middleware() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mws: Vec<Box<dyn Middleware>> = vec![
            Box::new(TrackingMiddleware { label: "A".into(), log: log.clone() }),
            Box::new(TrackingMiddleware { label: "B".into(), log: log.clone() }),
        ];
        let mut agent = Agent::new();
        Next { remaining: &mws, phase: Phase::Start }.call(&mut agent).await.unwrap();
        let entries = log.lock().unwrap().clone();
        assert_eq!(entries, vec![
            "A_start_before", "B_start_before", "B_start_after", "A_start_after",
        ]);
    }

    #[tokio::test]
    async fn test_chain_wrapping_llm_phase() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mws: Vec<Box<dyn Middleware>> = vec![
            Box::new(TrackingMiddleware { label: "MW".into(), log: log.clone() }),
        ];
        let mut agent = Agent::new();
        agent.session = crate::kernel::agent::Session::with_messages(vec![
            serde_json::json!({"role": "user", "content": "hi"}),
        ]);
        let _ = Next { remaining: &mws, phase: Phase::Llm }.call(&mut agent).await;
        let entries = log.lock().unwrap().clone();
        assert_eq!(entries[0], "MW_llm_before");
        assert_eq!(entries[1], "MW_llm_after");
    }

    #[tokio::test]
    async fn test_chain_error_propagation() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mws: Vec<Box<dyn Middleware>> = vec![
            Box::new(TrackingMiddleware { label: "A".into(), log: log.clone() }),
            Box::new(ErrorMiddleware),
        ];
        let mut agent = Agent::new();
        let result = Next { remaining: &mws, phase: Phase::Llm }.call(&mut agent).await;
        assert!(result.is_err());
        let entries = log.lock().unwrap().clone();
        assert_eq!(entries, vec!["A_llm_before", "A_llm_after"]);
    }

    #[tokio::test]
    async fn test_chain_error_short_circuit() {
        struct ShortCircuitMiddleware {
            log: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait]
        impl Middleware for ShortCircuitMiddleware {
            async fn wrap_llm(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
                self.log.lock().unwrap().push("before".into());
                next.call(ctx).await?;
                self.log.lock().unwrap().push("after".into());
                Ok(())
            }
        }

        let log = Arc::new(Mutex::new(Vec::new()));
        let mws: Vec<Box<dyn Middleware>> = vec![
            Box::new(ShortCircuitMiddleware { log: log.clone() }),
            Box::new(ErrorMiddleware),
        ];
        let mut agent = Agent::new();
        let result = Next { remaining: &mws, phase: Phase::Llm }.call(&mut agent).await;
        assert!(result.is_err());
        let entries = log.lock().unwrap().clone();
        assert_eq!(entries, vec!["before"]);
    }

    #[tokio::test]
    async fn test_all_phases_dispatch() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mws: Vec<Box<dyn Middleware>> = vec![
            Box::new(TrackingMiddleware { label: "X".into(), log: log.clone() }),
        ];
        let mut agent = Agent::new();

        Next { remaining: &mws, phase: Phase::Start }.call(&mut agent).await.unwrap();
        let _ = Next { remaining: &mws, phase: Phase::Llm }.call(&mut agent).await;
        Next { remaining: &mws, phase: Phase::End }.call(&mut agent).await.unwrap();

        let entries = log.lock().unwrap().clone();
        assert!(entries.contains(&"X_start_before".to_string()));
        assert!(entries.contains(&"X_llm_before".to_string()));
        assert!(entries.contains(&"X_end_before".to_string()));
    }

    #[tokio::test]
    async fn test_tool_phase_dispatch() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mws: Vec<Box<dyn Middleware>> = vec![
            Box::new(TrackingMiddleware { label: "T".into(), log: log.clone() }),
        ];
        let mut agent = Agent::new();
        agent.session = crate::kernel::agent::Session::with_messages(vec![
            serde_json::json!({"role": "assistant", "content": "no tools"}),
        ]);
        Next { remaining: &mws, phase: Phase::Tool }.call(&mut agent).await.unwrap();
        let entries = log.lock().unwrap().clone();
        assert_eq!(entries, vec!["T_tool_before", "T_tool_after"]);
    }
}
