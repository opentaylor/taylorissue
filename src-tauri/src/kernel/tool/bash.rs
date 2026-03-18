use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;

use super::base::BaseTool;
use crate::services::shell_env;

pub struct BashTool {
    pub timeout: Duration,
    pub workdir: Option<PathBuf>,
}

impl BashTool {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(120),
            workdir: None,
        }
    }

    pub fn with_workdir(mut self, workdir: PathBuf) -> Self {
        self.workdir = Some(workdir);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseTool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Run a shell command and return stdout + stderr."
    }

    fn params_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'command' argument")?;

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        shell_env::apply_env_async(&mut cmd);

        if let Some(ref workdir) = self.workdir {
            cmd.current_dir(workdir);
        }

        let output = tokio::time::timeout(self.timeout, cmd.output()).await;

        match output {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let mut result = String::new();
                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(&stderr);
                }
                if exit_code != 0 {
                    result.push_str(&format!("\nExit code: {}", exit_code));
                }
                Ok(result)
            }
            Ok(Err(e)) => Err(Box::new(e)),
            Err(_) => Ok(format!(
                "Command timed out after {} seconds",
                self.timeout.as_secs()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool::new();
        let result = tool
            .run(serde_json::json!({"command": "echo hello"}))
            .await
            .unwrap();
        assert!(result.contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_exit_code() {
        let tool = BashTool::new();
        let result = tool
            .run(serde_json::json!({"command": "exit 1"}))
            .await
            .unwrap();
        assert!(result.contains("Exit code: 1"));
    }

    #[tokio::test]
    async fn test_bash_stderr() {
        let tool = BashTool::new();
        let result = tool
            .run(serde_json::json!({"command": "echo err >&2"}))
            .await
            .unwrap();
        assert!(result.contains("err"));
    }

    #[tokio::test]
    async fn test_bash_workdir() {
        let tool = BashTool::new().with_workdir(PathBuf::from("/tmp"));
        let result = tool
            .run(serde_json::json!({"command": "pwd"}))
            .await
            .unwrap();
        assert!(result.contains("/tmp") || result.contains("/private/tmp"));
    }

    #[tokio::test]
    async fn test_bash_timeout() {
        let tool = BashTool::new().with_timeout(Duration::from_millis(100));
        let result = tool
            .run(serde_json::json!({"command": "sleep 10"}))
            .await
            .unwrap();
        assert!(result.contains("timed out"));
    }

    #[test]
    fn test_bash_schema() {
        let tool = BashTool::new();
        assert_eq!(tool.name(), "bash");
        let schema = tool.params_schema();
        assert!(schema["properties"]["command"].is_object());
    }
}
