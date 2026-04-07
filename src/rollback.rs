use crate::runner::run_command;
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::path::Path;

pub async fn create_checkpoint(root: &Path) -> Result<String> {
    let stamp = format!("pata-checkpoint-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let add = run_command(root, "git", &["add", "-A"]).await?;
    if add.exit_code != 0 {
        return Err(anyhow!("git add failed: {}", add.stderr));
    }
    let commit = run_command(root, "git", &["commit", "-m", &stamp, "--allow-empty"]).await?;
    if commit.exit_code != 0 {
        return Err(anyhow!("checkpoint commit failed: {}", commit.stderr));
    }
    Ok(stamp)
}

pub async fn rollback_to(root: &Path, checkpoint: &str) -> Result<()> {
    let reset = run_command(root, "git", &["reset", "--hard", checkpoint]).await?;
    if reset.exit_code != 0 {
        return Err(anyhow!("rollback failed: {}", reset.stderr));
    }
    Ok(())
}
