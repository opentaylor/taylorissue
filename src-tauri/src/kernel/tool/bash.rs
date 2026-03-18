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
        #[cfg(windows)]
        { "powershell" }
        #[cfg(not(windows))]
        { "bash" }
    }

    fn description(&self) -> &str {
        #[cfg(windows)]
        { "Run a PowerShell command on Windows and return stdout + stderr." }
        #[cfg(not(windows))]
        { "Run a bash shell command and return stdout + stderr." }
    }

    fn params_schema(&self) -> Value {
        #[cfg(windows)]
        let cmd_desc = "The PowerShell command to execute (runs via powershell.exe -Command)";
        #[cfg(not(windows))]
        let cmd_desc = "The bash shell command to execute (runs via sh -c)";

        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": cmd_desc,
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

        let mut cmd = {
            #[cfg(windows)]
            {
                let mut c = Command::new("powershell.exe");
                c.args(["-NoProfile", "-NonInteractive", "-Command", command]);
                c
            }
            #[cfg(not(windows))]
            {
                let mut c = Command::new("sh");
                c.arg("-c").arg(command);
                c
            }
        };
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
    async fn test_echo() {
        let tool = BashTool::new();
        let result = tool
            .run(serde_json::json!({"command": "echo hello"}))
            .await
            .unwrap();
        assert!(result.contains("hello"));
    }

    #[tokio::test]
    async fn test_exit_code() {
        let tool = BashTool::new();
        #[cfg(windows)]
        let cmd = "exit 1";
        #[cfg(not(windows))]
        let cmd = "exit 1";
        let result = tool
            .run(serde_json::json!({"command": cmd}))
            .await
            .unwrap();
        assert!(result.contains("Exit code: 1"));
    }

    #[tokio::test]
    async fn test_stderr() {
        let tool = BashTool::new();
        #[cfg(windows)]
        let cmd = "Write-Error 'err'";
        #[cfg(not(windows))]
        let cmd = "echo err >&2";
        let result = tool
            .run(serde_json::json!({"command": cmd}))
            .await
            .unwrap();
        assert!(result.contains("err"));
    }

    #[tokio::test]
    async fn test_workdir() {
        let tmp = std::env::temp_dir();
        let tool = BashTool::new().with_workdir(tmp.clone());
        #[cfg(windows)]
        let cmd = "(Get-Location).Path";
        #[cfg(not(windows))]
        let cmd = "pwd";
        let result = tool
            .run(serde_json::json!({"command": cmd}))
            .await
            .unwrap();
        let tmp_lower = tmp.to_string_lossy().to_lowercase();
        let result_lower = result.to_lowercase().replace('/', "\\");
        assert!(
            result_lower.contains(tmp_lower.trim_end_matches('\\'))
                || result_lower.contains(tmp_lower.trim_end_matches('/')),
            "workdir output should contain temp path; got: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_timeout() {
        let tool = BashTool::new().with_timeout(Duration::from_millis(100));
        #[cfg(windows)]
        let cmd = "Start-Sleep -Seconds 10";
        #[cfg(not(windows))]
        let cmd = "sleep 10";
        let result = tool
            .run(serde_json::json!({"command": cmd}))
            .await
            .unwrap();
        assert!(result.contains("timed out"));
    }

    #[test]
    fn test_schema() {
        let tool = BashTool::new();
        #[cfg(windows)]
        assert_eq!(tool.name(), "powershell");
        #[cfg(not(windows))]
        assert_eq!(tool.name(), "bash");
        let schema = tool.params_schema();
        assert!(schema["properties"]["command"].is_object());
    }
}
