use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::base::BaseTool;

pub struct SubAgentSpec {
    pub name: String,
    pub description: String,
}

type RunnerFn = dyn Fn(String, String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>>
    + Send
    + Sync;

pub struct TaskTool {
    runner: Arc<RunnerFn>,
    specs: Vec<SubAgentSpec>,
    description: String,
}

impl TaskTool {
    pub fn new(
        runner: Arc<RunnerFn>,
        specs: Vec<SubAgentSpec>,
    ) -> Self {
        let agents_list: String = specs
            .iter()
            .map(|s| format!("- `{}`: {}", s.name, s.description))
            .collect::<Vec<_>>()
            .join("\n");
        let description = format!(
            "Launch a subagent to handle a task with an isolated context.\n\nAvailable agent types:\n{}",
            agents_list
        );
        Self { runner, specs, description }
    }

    fn build_schema(&self) -> Value {
        let type_enum: Vec<String> = self.specs.iter().map(|s| s.name.clone()).collect();
        serde_json::json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Detailed task description including all context the subagent needs."
                },
                "subagent_type": {
                    "type": "string",
                    "description": "Which subagent type to use.",
                    "enum": type_enum
                }
            },
            "required": ["description", "subagent_type"]
        })
    }
}

#[derive(Deserialize, JsonSchema)]
struct TaskArgs {
    /// Detailed task description including all context the subagent needs
    description: String,
    /// Which subagent type to use
    subagent_type: String,
}

#[async_trait]
impl BaseTool for TaskTool {
    fn name(&self) -> &str { "task" }

    fn description(&self) -> &str {
        &self.description
    }

    fn params_schema(&self) -> Value {
        self.build_schema()
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: TaskArgs = serde_json::from_value(args)?;

        if !self.specs.iter().any(|s| s.name == args.subagent_type) {
            let available: Vec<&str> = self.specs.iter().map(|s| s.name.as_str()).collect();
            return Ok(format!(
                "Error: Unknown subagent type '{}'. Available: {:?}",
                args.subagent_type, available
            ));
        }

        (self.runner)(args.subagent_type, args.description).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_tool_unknown_type() {
        let runner: Arc<RunnerFn> = Arc::new(|_, _| {
            Box::pin(async { Ok("done".to_string()) })
        });
        let tool = TaskTool::new(runner, vec![
            SubAgentSpec { name: "general".to_string(), description: "General purpose".to_string() },
        ]);

        let result = tool.run(serde_json::json!({
            "description": "do something",
            "subagent_type": "nonexistent"
        })).await.unwrap();

        assert!(result.contains("Unknown subagent type"));
    }

    #[tokio::test]
    async fn test_task_tool_calls_runner() {
        let runner: Arc<RunnerFn> = Arc::new(|agent_type, desc| {
            Box::pin(async move {
                Ok(format!("Ran {} with: {}", agent_type, desc))
            })
        });
        let tool = TaskTool::new(runner, vec![
            SubAgentSpec { name: "general".to_string(), description: "General purpose".to_string() },
        ]);

        let result = tool.run(serde_json::json!({
            "description": "analyze code",
            "subagent_type": "general"
        })).await.unwrap();

        assert!(result.contains("Ran general with: analyze code"));
    }

    #[test]
    fn test_task_tool_schema() {
        let runner: Arc<RunnerFn> = Arc::new(|_, _| {
            Box::pin(async { Ok("done".to_string()) })
        });
        let tool = TaskTool::new(runner, vec![
            SubAgentSpec { name: "general".to_string(), description: "GP".to_string() },
            SubAgentSpec { name: "coder".to_string(), description: "Code".to_string() },
        ]);

        let schema = tool.params_schema();
        let enum_values = schema["properties"]["subagent_type"]["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 2);
    }

    #[test]
    fn test_task_tool_description() {
        let runner: Arc<RunnerFn> = Arc::new(|_, _| {
            Box::pin(async { Ok("done".to_string()) })
        });
        let tool = TaskTool::new(runner, vec![
            SubAgentSpec { name: "general".to_string(), description: "General purpose agent".to_string() },
        ]);
        let desc = tool.description();
        assert!(desc.contains("`general`"));
        assert!(desc.contains("General purpose agent"));
        assert!(desc.contains("Available agent types"));
    }

    #[tokio::test]
    async fn test_task_tool_runner_error() {
        let runner: Arc<RunnerFn> = Arc::new(|_, _| {
            Box::pin(async {
                Err::<String, Box<dyn std::error::Error + Send + Sync>>("agent crashed".into())
            })
        });
        let tool = TaskTool::new(runner, vec![
            SubAgentSpec { name: "general".to_string(), description: "GP".to_string() },
        ]);
        let result = tool.run(serde_json::json!({
            "description": "do it",
            "subagent_type": "general"
        })).await;
        assert!(result.is_err());
    }
}
