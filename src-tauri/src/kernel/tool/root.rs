use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use super::base::{schema_for, BaseTool};
use crate::services::shell_env;

#[derive(Deserialize, JsonSchema)]
struct RootArgs {
    /// The shell command to execute with administrator privileges
    command: String,
    /// Max seconds to wait before killing the command
    #[serde(default)]
    timeout: Option<u64>,
}

pub struct RootTool {
    pub timeout: Duration,
}

impl RootTool {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(300),
        }
    }
}

impl Default for RootTool {
    fn default() -> Self {
        Self::new()
    }
}

const DESCRIPTION: &str = "Run a shell command with administrator/root privileges. \
The OS will show a native password dialog to the user for authorization.

ONLY use this tool when a regular bash/powershell command fails with a permission error \
(EACCES, permission denied, access denied). Do NOT use it as the first attempt — \
always try the regular bash tool first.

Usage notes:
- For long-running commands, use the timeout parameter (default is 300 seconds).
- The user will see an OS-native authentication dialog (macOS password prompt, Linux polkit, or Windows UAC).
- If the user cancels the dialog, the command will fail.";

const PIPE_GRACE_PERIOD: Duration = Duration::from_secs(2);

#[async_trait]
impl BaseTool for RootTool {
    fn name(&self) -> &str {
        "root"
    }

    fn description(&self) -> &str {
        DESCRIPTION
    }

    fn params_schema(&self) -> Value {
        schema_for::<RootArgs>()
    }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: RootArgs = serde_json::from_value(args)?;
        let timeout = args
            .timeout
            .map(Duration::from_secs)
            .unwrap_or(self.timeout);

        let mut cmd = build_elevated_command(&args.command);
        shell_env::apply_env_async(&mut cmd);

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

#[cfg(target_os = "macos")]
fn build_elevated_command(command: &str) -> Command {
    let escaped = command
        .replace('\\', "\\\\\\\\")
        .replace('"', "\\\\\"")
        .replace('\'', "'\\''");
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        escaped
    );
    let mut cmd = Command::new("osascript");
    cmd.args(["-e", &script]);
    cmd.stdin(Stdio::null());
    cmd
}

#[cfg(all(unix, not(target_os = "macos")))]
fn build_elevated_command(command: &str) -> Command {
    let mut cmd = Command::new("pkexec");
    cmd.args(["sh", "-c", command]);
    cmd.stdin(Stdio::null());
    cmd
}

#[cfg(windows)]
fn build_elevated_command(command: &str) -> Command {
    let temp_dir =
        std::env::var("TEMP").unwrap_or_else(|_| std::env::var("TMP").unwrap_or_default());
    let out_file = format!("{}\\root_tool_{}.txt", temp_dir, std::process::id());
    let err_file = format!("{}\\root_tool_{}_err.txt", temp_dir, std::process::id());

    let inner_script = command
        .replace('\'', "''")
        .replace('"', "`\"");

    let wrapped = format!(
        "$ErrorActionPreference='Continue'; \
         try {{ {inner_script} }} catch {{ $_.Exception.Message }} \
         *> '{out_file}'; \
         $LASTEXITCODE | Set-Content '{err_file}'"
    );

    let outer = format!(
        "$p = Start-Process powershell -Verb RunAs -Wait -PassThru \
         -ArgumentList '-NoProfile','-NonInteractive','-Command','{wrapped}'; \
         if (Test-Path '{out_file}') {{ Get-Content '{out_file}' -Raw }}; \
         $rc = if (Test-Path '{err_file}') {{ (Get-Content '{err_file}' -Raw).Trim() }} else {{ '0' }}; \
         Remove-Item '{out_file}','{err_file}' -EA SilentlyContinue; \
         exit ([int]$rc)"
    );

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &outer]);
    cmd.stdin(Stdio::null());
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
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
    async fn test_schema() {
        let tool = RootTool::new();
        assert_eq!(tool.name(), "root");
        let schema = tool.params_schema();
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_description_mentions_permission() {
        let tool = RootTool::new();
        assert!(tool.description().contains("permission"));
        assert!(tool.description().contains("administrator"));
    }

    #[test]
    fn test_format_output() {
        assert_eq!(format_output("hello", "", 0), "hello");
        assert_eq!(format_output("", "err", 0), "err");
        assert_eq!(format_output("out", "err", 0), "out\nerr");
        assert_eq!(format_output("", "", 1), "\nExit code: 1");
    }

    #[test]
    fn test_default() {
        let tool = RootTool::default();
        assert_eq!(tool.timeout, Duration::from_secs(300));
    }
}
