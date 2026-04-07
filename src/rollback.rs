use crate::fssec;
use crate::lock;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn checkpoint(root: &Path, msg: &str) -> Result<(), String> {
    let _guard = lock::acquire(root, "git-write")?;
    run(root, &["add", "-A"])?;
    run(root, &["commit", "-m", msg, "--allow-empty"])?;
    Ok(())
}

pub fn rollback(root: &Path, rev: &str) -> Result<(), String> {
    let _guard = lock::acquire(root, "git-write")?;
    run(root, &["reset", "--hard", rev])?;
    fs::create_dir_all(root.join(".pata")).map_err(|e| e.to_string())?;
    fssec::set_secure_dir(&root.join(".pata"))?;
    let path = root.join(".pata/last_rollback.txt");
    fs::write(&path, format!("rolled back to {rev}\n")).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)
}

fn run(root: &Path, args: &[&str]) -> Result<(), String> {
    let out = Command::new("git").args(args).current_dir(root).output();
    match out {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => Err(e.to_string()),
    }
}
