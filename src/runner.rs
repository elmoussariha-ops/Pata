use crate::types::CommandReport;
use anyhow::Result;
use std::{path::Path, time::Instant};
use tokio::process::Command;

pub async fn run(root: &Path, bin: &str, args: &[&str]) -> Result<CommandReport> {
    let start = Instant::now();
    let out = Command::new(bin)
        .args(args)
        .current_dir(root)
        .output()
        .await?;
    Ok(CommandReport {
        command: format!("{} {}", bin, args.join(" ")),
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        duration_ms: start.elapsed().as_millis(),
    })
}
