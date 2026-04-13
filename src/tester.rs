use crate::types::ValidationResult;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

const EVAL_VERSION: &str = "validate.v2";
const MAX_DURATION_REGRESSION_PCT: u128 = 25;

pub fn validate(root: &Path) -> ValidationResult {
    let mut logs = Vec::new();
    let t0 = Instant::now();
    let check_ok = run(
        root,
        &["check", "--all-targets", "--offline"],
        &mut logs,
        true,
    ) || run(root, &["check", "--all-targets"], &mut logs, true);
    let clippy_ok = run(
        root,
        &["clippy", "--offline", "--", "-D", "warnings"],
        &mut logs,
        true,
    ) || run(root, &["clippy", "--", "-D", "warnings"], &mut logs, true);
    let test_ok = run(root, &["test", "--offline", "--quiet"], &mut logs, true)
        || run(root, &["test", "--quiet"], &mut logs, true);
    let total_duration_ms = t0.elapsed().as_millis();
    let regression_alerts =
        detect_regressions(root, check_ok, clippy_ok, test_ok, total_duration_ms);

    if check_ok && clippy_ok && test_ok && regression_alerts.is_empty() {
        let _ = write_baseline(root, total_duration_ms);
    }
    ValidationResult {
        eval_version: EVAL_VERSION.to_string(),
        check_ok,
        clippy_ok,
        test_ok,
        total_duration_ms,
        regression_alerts,
        logs,
    }
}

fn run(root: &Path, args: &[&str], logs: &mut Vec<String>, retry_transient: bool) -> bool {
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
            if o.status.success() {
                true
            } else if retry_transient && looks_transient(&o.stderr) {
                logs.push(format!(
                    "retrying transient failure: cargo {}",
                    args.join(" ")
                ));
                run(root, args, logs, false)
            } else {
                false
            }
        }
        Err(e) => {
            logs.push(format!("cargo {:?} failed: {e}", args));
            false
        }
    }
}

fn looks_transient(stderr: &[u8]) -> bool {
    let s = String::from_utf8_lossy(stderr).to_lowercase();
    s.contains("timed out")
        || s.contains("timeout")
        || s.contains("temporary")
        || s.contains("blocking waiting for file lock")
}

fn baseline_path(root: &Path) -> std::path::PathBuf {
    root.join(".pata/evals/validation_baseline.v1")
}

fn write_baseline(root: &Path, duration_ms: u128) -> Result<(), String> {
    let path = baseline_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, format!("duration_ms={duration_ms}\n")).map_err(|e| e.to_string())
}

fn read_baseline(root: &Path) -> Option<u128> {
    let path = baseline_path(root);
    let text = fs::read_to_string(path).ok()?;
    let v = text.trim().strip_prefix("duration_ms=")?;
    v.parse::<u128>().ok()
}

fn detect_regressions(
    root: &Path,
    check_ok: bool,
    clippy_ok: bool,
    test_ok: bool,
    duration_ms: u128,
) -> Vec<String> {
    let mut alerts = Vec::new();
    if !(check_ok && clippy_ok && test_ok) {
        alerts.push("build-pipeline-not-green".to_string());
    }
    if let Some(base) = read_baseline(root) {
        let allowed = base + (base * MAX_DURATION_REGRESSION_PCT / 100);
        if duration_ms > allowed {
            alerts.push(format!(
                "runtime-regression:{}ms>{}ms(base={}ms,+{}%)",
                duration_ms, allowed, base, MAX_DURATION_REGRESSION_PCT
            ));
        }
    }
    alerts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("pata-tester-{ts}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn detects_runtime_regression_against_baseline() {
        let root = temp_root();
        write_baseline(&root, 100).unwrap();
        let alerts = detect_regressions(&root, true, true, true, 130);
        assert!(alerts.iter().any(|a| a.contains("runtime-regression")));
    }
}
