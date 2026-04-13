use crate::coder;
use crate::types::ValidationResult;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
