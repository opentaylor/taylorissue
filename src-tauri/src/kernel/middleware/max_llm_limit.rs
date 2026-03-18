use async_trait::async_trait;
use crate::kernel::agent::Agent;
use super::base::Middleware;

pub struct MaxLlmLimitMiddleware {
    pub limit: usize,
}

impl MaxLlmLimitMiddleware {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

#[async_trait]
impl Middleware for MaxLlmLimitMiddleware {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_llm_limit_creation() {
        let mw = MaxLlmLimitMiddleware::new(50);
        assert_eq!(mw.limit, 50);
    }
}
