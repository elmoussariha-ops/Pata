use crate::coder;
use crate::types::ValidationResult;
use std::fs;
use std::path::Path;

pub const EVAL_SUITE_VERSION: &str = "evals.2026-04-13.v1";
const MAX_RUNTIME_MS: u128 = 180_000;

#[derive(Debug, Clone)]
pub struct EvalCaseResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct EvalRun {
    pub suite_version: String,
    pub blocked: bool,
    pub score_pct: u8,
    pub cases: Vec<EvalCaseResult>,
}

pub fn run(_root: &Path, validation: &ValidationResult) -> EvalRun {
    let persona = coder::persona_contract_snapshot();
    let cases = vec![
        EvalCaseResult {
            name: "pipeline_green".to_string(),
            passed: validation.ok(),
            detail: format!(
                "check={} clippy={} test={}",
                validation.check_ok, validation.clippy_ok, validation.test_ok
            ),
        },
        EvalCaseResult {
            name: "regression_threshold_gate".to_string(),
            passed: validation.regression_alerts.is_empty(),
            detail: if validation.regression_alerts.is_empty() {
                "no regression".to_string()
            } else {
                validation.regression_alerts.join(" | ")
            },
        },
        EvalCaseResult {
            name: "persona_contract_integrity".to_string(),
            passed: persona.clauses.len() >= 5 && persona.version.starts_with("persona."),
            detail: format!(
                "version={} clauses={}",
                persona.version,
                persona.clauses.len()
            ),
        },
        EvalCaseResult {
            name: "runtime_budget".to_string(),
            passed: validation.total_duration_ms <= MAX_RUNTIME_MS,
            detail: format!(
                "duration_ms={} budget_ms={}",
                validation.total_duration_ms, MAX_RUNTIME_MS
            ),
        },
    ];
    let passed = cases.iter().filter(|c| c.passed).count();
    let score_pct = ((passed * 100) / cases.len()) as u8;
    let blocked = score_pct < 100;
    EvalRun {
        suite_version: EVAL_SUITE_VERSION.to_string(),
        blocked,
        score_pct,
        cases,
    }
}

pub fn persist(
    root: &Path,
    validation: &ValidationResult,
    run: &EvalRun,
) -> Result<String, String> {
    let dir = root.join(".pata/evals/runs");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.txt", run.suite_version));
    let mut body = String::new();
    body.push_str(&format!("suite_version={}\n", run.suite_version));
    body.push_str(&format!("validate_version={}\n", validation.eval_version));
    body.push_str(&format!("check_ok={}\n", validation.check_ok));
    body.push_str(&format!("clippy_ok={}\n", validation.clippy_ok));
    body.push_str(&format!("test_ok={}\n", validation.test_ok));
    body.push_str(&format!(
        "total_duration_ms={}\n",
        validation.total_duration_ms
    ));
    body.push_str(&format!(
        "regressions={}\n",
        validation.regression_alerts.len()
    ));
    body.push_str(&format!("score_pct={}\n", run.score_pct));
    body.push_str(&format!("blocked={}\n", run.blocked));
    for case in &run.cases {
        body.push_str(&format!(
            "case\t{}\t{}\t{}\n",
            case.name, case.passed, case.detail
        ));
    }
    fs::write(&path, body).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
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
        let root = std::env::temp_dir().join(format!("pata-evals-{ts}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn eval_suite_blocks_on_regression() {
        let v = ValidationResult {
            eval_version: "x".to_string(),
            check_ok: true,
            clippy_ok: true,
            test_ok: true,
            total_duration_ms: 10,
            regression_alerts: vec!["slowdown".to_string()],
            logs: Vec::new(),
        };
        let run = run(Path::new("."), &v);
        assert!(run.blocked);
        assert_eq!(run.score_pct, 75);
    }

    #[test]
    fn eval_run_is_persisted_in_versioned_path() {
        let root = temp_root();
        let v = ValidationResult {
            eval_version: "validate.v2".to_string(),
            check_ok: true,
            clippy_ok: true,
            test_ok: true,
            total_duration_ms: 10,
            regression_alerts: Vec::new(),
            logs: Vec::new(),
        };
        let run = run(&root, &v);
        let written = persist(&root, &v, &run).unwrap();
        assert!(written.ends_with("evals.2026-04-13.v1.txt"));
        let content =
            fs::read_to_string(root.join(".pata/evals/runs/evals.2026-04-13.v1.txt")).unwrap();
        assert!(content.contains("score_pct=100"));
        assert!(content.contains("case\tpipeline_green\ttrue"));
    }
}
