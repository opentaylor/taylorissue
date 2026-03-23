use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use super::base::{schema_for, BaseTool};
use crate::services::shell_env;

#[derive(Deserialize, JsonSchema)]
struct BashArgs {
    /// The shell command to execute
    command: String,
    /// Max seconds to wait before killing the command
    #[serde(default)]
    timeout: Option<u64>,
}

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

const BASH_DESCRIPTION: &str = "Run a shell command and return stdout + stderr.

Usage notes:
- For long-running commands, use the timeout parameter to override the default (e.g. timeout=300).
- For background/daemon processes, ALWAYS redirect stdout and stderr to avoid hanging:
    nohup cmd > /dev/null 2>&1 &
  Without redirection the tool will wait for the background process to finish.
- When chaining commands use && or ; — do NOT use newlines.
- Avoid using cat/head/tail to read files — use the read tool instead.
- Avoid find/grep commands — use the dedicated search tools instead.";

#[cfg(windows)]
const POWERSHELL_DESCRIPTION: &str = "Run a PowerShell command on Windows and return stdout + stderr.

Usage notes:
- For long-running commands, use the timeout parameter to override the default (e.g. timeout=300).
- For background processes, use Start-Process to avoid hanging:
    Start-Process cmd -WindowStyle Hidden
- When chaining commands use ; — do NOT use newlines.";

const PIPE_GRACE_PERIOD: Duration = Duration::from_secs(2);

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
        { POWERSHELL_DESCRIPTION }
        #[cfg(not(windows))]
        { BASH_DESCRIPTION }
    }

    fn params_schema(&self) -> Value { schema_for::<BashArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: BashArgs = serde_json::from_value(args)?;
        let timeout = args.timeout.map(Duration::from_secs).unwrap_or(self.timeout);

        let mut cmd = {
            #[cfg(windows)]
            {
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let wrapped = format!(
                    "$ErrorActionPreference='Continue'; {}",
                    args.command
                );
                let mut c = Command::new("powershell.exe");
                c.args(["-NoProfile", "-NonInteractive", "-Command", &wrapped]);
                c.stdin(Stdio::null());
                c.creation_flags(CREATE_NO_WINDOW);
                c
            }
            #[cfg(not(windows))]
            {
                let mut c = Command::new("sh");
                c.arg("-c").arg(&args.command);
                c.stdin(Stdio::null());
                c
            }
        };
        shell_env::apply_env_async(&mut cmd);

        if let Some(ref workdir) = self.workdir {
            cmd.current_dir(workdir);
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        let stdout_task = tokio::spawn(async move {
            let mut buf = String::new();
            if let Some(mut pipe) = stdout_pipe {
                let _ = pipe.read_to_string(&mut buf).await;
            }
            buf
        });

        let stderr_task = tokio::spawn(async move {
            let mut buf = String::new();
            if let Some(mut pipe) = stderr_pipe {
                let _ = pipe.read_to_string(&mut buf).await;
            }
            buf
        });

        let wait_result = tokio::time::timeout(timeout, child.wait()).await;

        match wait_result {
            Ok(Ok(status)) => {
                let (stdout_r, stderr_r) = tokio::join!(
                    tokio::time::timeout(PIPE_GRACE_PERIOD, stdout_task),
                    tokio::time::timeout(PIPE_GRACE_PERIOD, stderr_task),
                );
                let stdout = stdout_r.ok().and_then(|r| r.ok()).unwrap_or_default();
                let stderr = stderr_r.ok().and_then(|r| r.ok()).unwrap_or_default();
                let exit_code = status.code().unwrap_or(-1);

                Ok(format_output(&stdout, &stderr, exit_code))
            }
            Ok(Err(e)) => Err(Box::new(e)),
            Err(_) => {
                let _ = child.start_kill();
                stdout_task.abort();
                stderr_task.abort();
                Ok(format!(
                    "Command timed out after {} seconds",
                    timeout.as_secs()
                ))
            }
        }
    }
}

fn format_output(stdout: &str, stderr: &str, exit_code: i32) -> String {
    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(stdout);
    }
    if !stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(stderr);
    }
    if exit_code != 0 {
        result.push_str(&format!("\nExit code: {}", exit_code));
    }
    result
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
        let result = tool
            .run(serde_json::json!({"command": "exit 1"}))
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
        let tmp = std::env::temp_dir().canonicalize().unwrap_or_else(|_| std::env::temp_dir());
        let tool = BashTool::new().with_workdir(tmp.clone());
        let result = tool
            .run(serde_json::json!({"command": "pwd"}))
            .await
            .unwrap();
        let normalize = |s: &str| {
            let s = s.trim().to_lowercase().replace('\\', "/");
            s.strip_prefix("//?/").unwrap_or(&s).to_string()
        };
        let result_norm = normalize(&result);
        let tmp_norm = normalize(&tmp.to_string_lossy());
        assert!(
            result_norm.contains(tmp_norm.trim_end_matches('/')),
            "workdir output should contain temp path; got: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_timeout() {
        let tool = BashTool::new().with_timeout(Duration::from_millis(100));
        let result = tool
            .run(serde_json::json!({"command": "sleep 10"}))
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
        assert!(schema.get("properties").is_some());
    }

    #[tokio::test]
    async fn test_timeout_param_override() {
        let tool = BashTool::new();
        let result = tool
            .run(serde_json::json!({"command": "sleep 10", "timeout": 1}))
            .await
            .unwrap();
        assert!(result.contains("timed out"));
    }

    #[tokio::test]
    async fn test_background_process_returns_promptly() {
        let tool = BashTool::new();
        let start = std::time::Instant::now();
        #[cfg(windows)]
        let cmd = "Start-Process powershell -ArgumentList '-Command Start-Sleep 60' -WindowStyle Hidden; Write-Output 'started'";
        #[cfg(not(windows))]
        let cmd = "sleep 60 > /dev/null 2>&1 &\necho started";
        let result = tool
            .run(serde_json::json!({"command": cmd}))
            .await
            .unwrap();
        let elapsed = start.elapsed();
        assert!(result.contains("started"));
        assert!(elapsed < Duration::from_secs(10), "should return quickly, took {:?}", elapsed);
    }

    #[test]
    fn test_format_output() {
        assert_eq!(format_output("hello", "", 0), "hello");
        assert_eq!(format_output("", "err", 0), "err");
        assert_eq!(format_output("out", "err", 0), "out\nerr");
        assert_eq!(format_output("", "", 1), "\nExit code: 1");
        assert_eq!(format_output("ok", "", 0), "ok");
    }

    #[test]
    fn test_description_mentions_background() {
        let tool = BashTool::new();
        assert!(tool.description().contains("nohup") || tool.description().contains("background"));
    }
}
