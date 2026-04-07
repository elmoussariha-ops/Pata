use crate::types::{ActionPlan, DiagnosticReport, PlannedAction, ProjectAnalysis};
use chrono::Utc;

pub fn build_plan(analysis: &ProjectAnalysis, diagnostics: &DiagnosticReport) -> ActionPlan {
    let mut actions = Vec::new();

    if diagnostics.findings.iter().any(|f| f.contains("failed")) {
        actions.push(PlannedAction {
            title: "Stabilize build pipeline".to_string(),
            rationale: "Compiler/tests/lints currently fail; restore baseline first.".to_string(),
            target_files: vec![],
            requires_human_validation: true,
        });
    }

    for warning in analysis.warnings.iter().take(5) {
        actions.push(PlannedAction {
            title: "Address static warning".to_string(),
            rationale: warning.clone(),
            target_files: vec![],
            requires_human_validation: true,
        });
    }

    if actions.is_empty() {
        actions.push(PlannedAction {
            title: "Refactor high-complexity modules".to_string(),
            rationale: "No critical errors found; focus on maintainability and speed.".to_string(),
            target_files: analysis
                .insights
                .iter()
                .take(3)
                .map(|i| i.path.clone())
                .collect(),
            requires_human_validation: false,
        });
    }

    ActionPlan {
        created_at: Utc::now(),
        summary: format!(
            "Generated {} actions from {} files and {} findings",
            actions.len(),
            analysis.rust_files,
            diagnostics.findings.len()
        ),
        actions,
    }
}
