use crate::types::ValidationReport;

pub fn summarize_validation(report: &ValidationReport) -> Vec<String> {
    let mut out = Vec::new();
    for cmd in [&report.check, &report.clippy, &report.tests] {
        if cmd.code != 0 {
            let head = cmd.stderr.lines().take(4).collect::<Vec<_>>().join(" | ");
            out.push(format!("{} failed: {}", cmd.command, head));
        }
    }
    if out.is_empty() {
        out.push("All checks passed".to_string());
    }
    out
}
