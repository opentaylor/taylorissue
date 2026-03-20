use async_trait::async_trait;
use super::base::Middleware;

pub struct MaxToolLimitMiddleware {
    pub limit: usize,
}

impl MaxToolLimitMiddleware {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

#[async_trait]
impl Middleware for MaxToolLimitMiddleware {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_tool_limit_creation() {
        let mw = MaxToolLimitMiddleware::new(10);
        assert_eq!(mw.limit, 10);
    }
}
