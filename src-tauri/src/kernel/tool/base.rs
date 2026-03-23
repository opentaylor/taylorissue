use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait BaseTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn params_schema(&self) -> Value;

    fn to_openai_schema(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": self.description(),
                "parameters": self.params_schema(),
            }
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub fn schema_for<T: schemars::JsonSchema>() -> Value {
    serde_json::to_value(schemars::schema_for!(T)).unwrap()
}

pub struct FunctionTool {
    pub name: String,
    pub description: String,
    pub params_schema: Value,
    pub func: Box<dyn Fn(Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

#[async_trait]
impl BaseTool for FunctionTool {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn params_schema(&self) -> Value {
        self.params_schema.clone()
    }
    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        (self.func)(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_function_tool() {
        let tool = FunctionTool {
            name: "ping".to_string(),
            description: "Return pong".to_string(),
            params_schema: serde_json::json!({"type": "object", "properties": {}}),
            func: Box::new(|_| Ok("pong".to_string())),
        };
        assert_eq!(tool.name(), "ping");
        let result = tool.run(serde_json::json!({})).await.unwrap();
        assert_eq!(result, "pong");
    }

    #[test]
    fn test_to_openai_schema() {
        let tool = FunctionTool {
            name: "test".to_string(),
            description: "Test tool".to_string(),
            params_schema: serde_json::json!({"type": "object", "properties": {}}),
            func: Box::new(|_| Ok("ok".to_string())),
        };
        let schema = tool.to_openai_schema();
        assert_eq!(schema["type"], "function");
        assert_eq!(schema["function"]["name"], "test");
    }
}
