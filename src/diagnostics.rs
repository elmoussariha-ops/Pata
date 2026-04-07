use crate::{runner::run_command, types::DiagnosticReport};
use anyhow::Result;
use std::path::Path;

pub async fn run_diagnostics(root: &Path) -> Result<DiagnosticReport> {
    let check = run_command(root, "cargo", &["check", "--all-targets"]).await?;
    let tests = Some(run_command(root, "cargo", &["test", "--quiet"]).await?);
    let clippy = Some(run_command(root, "cargo", &["clippy", "--", "-D", "warnings"]).await?);

    let mut findings = Vec::new();
    for report in [&check, tests.as_ref().unwrap(), clippy.as_ref().unwrap()] {
        if report.exit_code != 0 {
            let preview = report
                .stderr
                .lines()
                .take(6)
                .collect::<Vec<_>>()
                .join(" | ");
            findings.push(format!("{} failed: {}", report.command, preview));
        }
    }

    if findings.is_empty() {
        findings.push("No compiler/test/lint failures detected.".to_string());
    }

    Ok(DiagnosticReport {
        check,
        tests,
        clippy,
        findings,
    })
}
