use crate::types::CommandReport;
use anyhow::Result;
use std::{path::Path, process::Stdio, time::Instant};
use tokio::process::Command;

pub async fn run_command(root: &Path, cmd: &str, args: &[&str]) -> Result<CommandReport> {
    let started = Instant::now();
    let output = Command::new(cmd)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    Ok(CommandReport {
        command: format!("{} {}", cmd, args.join(" ")),
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        duration_ms: started.elapsed().as_millis(),
    })
}
