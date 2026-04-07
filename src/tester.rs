use crate::types::ValidationResult;
use std::path::Path;
use std::process::Command;

pub fn validate(root: &Path) -> ValidationResult {
    let mut logs = Vec::new();
    let check_ok = run(root, &["check", "--all-targets", "--offline"], &mut logs)
        || run(root, &["check", "--all-targets"], &mut logs);
    let clippy_ok = run(
        root,
        &["clippy", "--offline", "--", "-D", "warnings"],
        &mut logs,
    ) || run(root, &["clippy", "--", "-D", "warnings"], &mut logs);
    let test_ok = run(root, &["test", "--offline", "--quiet"], &mut logs)
        || run(root, &["test", "--quiet"], &mut logs);
    ValidationResult {
        check_ok,
        clippy_ok,
        test_ok,
        logs,
    }
}

fn run(root: &Path, args: &[&str], logs: &mut Vec<String>) -> bool {
    let out = Command::new("cargo").args(args).current_dir(root).output();
    match out {
        Ok(o) => {
            logs.push(format!(
                "$ cargo {} => {}",
                args.join(" "),
                o.status.code().unwrap_or(-1)
            ));
            if !o.stdout.is_empty() {
                logs.push(String::from_utf8_lossy(&o.stdout).to_string());
            }
            if !o.stderr.is_empty() {
                logs.push(String::from_utf8_lossy(&o.stderr).to_string());
            }
            o.status.success()
        }
        Err(e) => {
            logs.push(format!("cargo {:?} failed: {e}", args));
            false
        }
    }
}
