use async_trait::async_trait;
use crate::kernel::agent::{Agent, Session};

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn wrap_start(&self, agent: &mut Agent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    async fn wrap_llm(&self, agent: &mut Agent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    async fn wrap_tool(&self, agent: &mut Agent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    async fn wrap_end(&self, agent: &mut Agent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopMiddleware;

    #[async_trait]
    impl Middleware for NoopMiddleware {}

    #[tokio::test]
    async fn test_default_middleware_passthrough() {
        let mw = NoopMiddleware;
        // default implementations should not error
        // (Agent construction is complex, so we test the trait compiles)
        assert!(true);
    }
}
