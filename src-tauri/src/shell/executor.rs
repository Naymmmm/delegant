use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ShellResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub async fn run_command(cmd: &str, timeout_secs: u64) -> AppResult<ShellResult> {
    let child = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
    } else {
        Command::new("bash")
            .args(["-c", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| AppError::Shell(format!("Command timed out after {}s: {}", timeout_secs, cmd)))?
    .map_err(|e| AppError::Shell(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();
    let exit_code = result.status.code().unwrap_or(-1);

    // Truncate very long outputs
    let max_len = 10000;
    let stdout = if stdout.len() > max_len {
        format!("{}...[truncated]", &stdout[..max_len])
    } else {
        stdout
    };
    let stderr = if stderr.len() > max_len {
        format!("{}...[truncated]", &stderr[..max_len])
    } else {
        stderr
    };

    Ok(ShellResult {
        stdout,
        stderr,
        exit_code,
    })
}
