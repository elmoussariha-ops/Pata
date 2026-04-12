//! Structured and validable reasoning model.
//!
//! This crate introduces a deterministic single-branch reasoning pipeline:
//! Analyze -> Hypothesis -> Action/Test -> Validation,
//! plus a first internal verify/correct loop compatible with future Chain-of-Verification.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Minimal phases required by the current reasoning protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningPhase {
    Analyze,
    Hypothesis,
    ActionOrTest,
    Validation,
}

/// A deterministic reasoning step in a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub phase: ReasoningPhase,
    pub goal: String,
    pub expected_artifact: String,
}

impl ReasoningStep {
    pub fn new(
        phase: ReasoningPhase,
        goal: impl Into<String>,
        expected_artifact: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            goal: goal.into(),
            expected_artifact: expected_artifact.into(),
        }
    }
}

/// Full structured reasoning plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningPlan {
    pub objective: String,
    pub steps: Vec<ReasoningStep>,
}

impl ReasoningPlan {
    pub fn new(objective: impl Into<String>, steps: Vec<ReasoningStep>) -> Self {
        Self {
            objective: objective.into(),
            steps,
        }
    }

    /// Validate required phase coverage and ordering.
    pub fn validate_structure(&self) -> Result<(), ValidationFailure> {
        if self.steps.len() < 4 {
            return Err(ValidationFailure::IncompletePlan {
                expected_min_steps: 4,
                actual_steps: self.steps.len(),
            });
        }

        let required = [
            ReasoningPhase::Analyze,
            ReasoningPhase::Hypothesis,
            ReasoningPhase::ActionOrTest,
            ReasoningPhase::Validation,
        ];

        let mut idx = 0usize;
        for phase in self.steps.iter().map(|s| s.phase) {
            if idx < required.len() && phase == required[idx] {
                idx += 1;
            }
        }

        if idx != required.len() {
            return Err(ValidationFailure::InvalidStepOrder {
                expected_order: required.to_vec(),
                found: self.steps.iter().map(|s| s.phase).collect(),
            });
        }

        Ok(())
    }
}

/// Status of an executed reasoning step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Succeeded,
    Failed,
}

/// Result of one reasoning step execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    pub phase: ReasoningPhase,
    pub output: String,
    pub status: StepStatus,
}

impl StepResult {
    pub fn succeeded(phase: ReasoningPhase, output: impl Into<String>) -> Self {
        Self {
            phase,
            output: output.into(),
            status: StepStatus::Succeeded,
        }
    }
}

/// Typed validation failures for plan and execution.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationFailure {
    #[error("reasoning plan is incomplete: expected at least {expected_min_steps} steps, got {actual_steps}")]
    IncompletePlan {
        expected_min_steps: usize,
        actual_steps: usize,
    },
    #[error("reasoning step order is invalid")]
    InvalidStepOrder {
        expected_order: Vec<ReasoningPhase>,
        found: Vec<ReasoningPhase>,
    },
    #[error("step phase mismatch at index {index}: expected {expected:?}, got {found:?}")]
    StepPhaseMismatch {
        index: usize,
        expected: ReasoningPhase,
        found: ReasoningPhase,
    },
}

/// Failure motif reported by internal verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationFailureKind {
    InternalInconsistency,
    InvalidOrInsufficientStep,
    DurableRuleContradiction,
}

/// Detailed verification failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationFailure {
    pub kind: VerificationFailureKind,
    pub message: String,
}

/// Deterministic correction decision after verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionDecision {
    Accept,
    ReviseStep {
        phase: ReasoningPhase,
        reason: VerificationFailure,
    },
    RecalculateFrom {
        phase: ReasoningPhase,
        reason: VerificationFailure,
    },
}

/// Verification report for one step output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepVerificationResult {
    pub phase: ReasoningPhase,
    pub passed: bool,
    pub failure: Option<VerificationFailure>,
    pub decision: CorrectionDecision,
}

impl StepVerificationResult {
    fn accepted(phase: ReasoningPhase) -> Self {
        Self {
            phase,
            passed: true,
            failure: None,
            decision: CorrectionDecision::Accept,
        }
    }

    fn revise(phase: ReasoningPhase, failure: VerificationFailure) -> Self {
        Self {
            phase,
            passed: false,
            failure: Some(failure.clone()),
            decision: CorrectionDecision::ReviseStep {
                phase,
                reason: failure,
            },
        }
    }

    fn recalculate_from(phase: ReasoningPhase, failure: VerificationFailure) -> Self {
        Self {
            phase,
            passed: false,
            failure: Some(failure.clone()),
            decision: CorrectionDecision::RecalculateFrom {
                phase,
                reason: failure,
            },
        }
    }
}

/// Contract for future contradiction checks against durable memory/rules.
pub trait DurableRuleChecker {
    fn detect_contradiction(&self, step: &ReasoningStep, result: &StepResult) -> Option<String>;
}

/// Default checker that accepts all steps.
#[derive(Debug, Default, Clone)]
pub struct NoopDurableRuleChecker;

impl DurableRuleChecker for NoopDurableRuleChecker {
    fn detect_contradiction(&self, _step: &ReasoningStep, _result: &StepResult) -> Option<String> {
        None
    }
}

/// Execution state for a single deterministic reasoning branch.
#[derive(Debug, Clone)]
pub struct ReasoningExecution<C = NoopDurableRuleChecker>
where
    C: DurableRuleChecker,
{
    pub plan: ReasoningPlan,
    pub results: Vec<StepResult>,
    pub verifications: Vec<StepVerificationResult>,
    checker: C,
}

/// Global failure motifs for the lightweight Chain-of-Verification layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalFailureKind {
    IncompleteReasoning,
    CrossStepInconsistency,
    HypothesisActionValidationMisalignment,
    DurableRuleContradiction,
}

/// Detailed global verification failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalFailure {
    pub kind: GlobalFailureKind,
    pub message: String,
}

/// Deterministic confidence level derived from the global score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

/// Final decision after global verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalDecision {
    Accept,
    NeedsRevision,
}

/// Global verification report for a full reasoning attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalVerificationReport {
    pub passed: bool,
    pub score: f32,
    pub confidence: ConfidenceLevel,
    pub failures: Vec<GlobalFailure>,
    pub decision: GlobalDecision,
}

/// Interface to expose full-reasoning verification to outer orchestrators.
pub trait GlobalReasoningVerifier {
    fn verify_global(&self) -> GlobalVerificationReport;
}

impl<C> ReasoningExecution<C>
where
    C: DurableRuleChecker,
{
    pub fn new(plan: ReasoningPlan, checker: C) -> Result<Self, ValidationFailure> {
        plan.validate_structure()?;

        Ok(Self {
            plan,
            results: Vec::new(),
            verifications: Vec::new(),
            checker,
        })
    }

    /// Verify and attempt to append one step result.
    ///
    /// If verification fails, returns a correction decision and does not append the result.
    pub fn verify_and_push(
        &mut self,
        result: StepResult,
    ) -> Result<StepVerificationResult, ValidationFailure> {
        let idx = self.results.len();
        let step = self
            .plan
            .steps
            .get(idx)
            .ok_or_else(|| ValidationFailure::IncompletePlan {
                expected_min_steps: self.plan.steps.len(),
                actual_steps: idx,
            })?;

        if step.phase != result.phase {
            return Err(ValidationFailure::StepPhaseMismatch {
                index: idx,
                expected: step.phase,
                found: result.phase,
            });
        }

        let verification = self.verify_step(step, &result);
        self.verifications.push(verification.clone());

        if verification.passed {
            self.results.push(result);
        }

        Ok(verification)
    }

    fn verify_step(&self, step: &ReasoningStep, result: &StepResult) -> StepVerificationResult {
        if result.status == StepStatus::Failed || result.output.trim().is_empty() {
            let failure = VerificationFailure {
                kind: VerificationFailureKind::InvalidOrInsufficientStep,
                message: "step output is empty or failed".to_string(),
            };
            return StepVerificationResult::revise(step.phase, failure);
        }

        let coherence = text_overlap_ratio(&step.expected_artifact, &result.output);
        if coherence == 0.0 {
            let failure = VerificationFailure {
                kind: VerificationFailureKind::InternalInconsistency,
                message: "step output is inconsistent with expected artifact".to_string(),
            };
            return StepVerificationResult::revise(step.phase, failure);
        }

        if let Some(rule) = self.checker.detect_contradiction(step, result) {
            let failure = VerificationFailure {
                kind: VerificationFailureKind::DurableRuleContradiction,
                message: format!("potential contradiction with durable rule: {rule}"),
            };
            return StepVerificationResult::recalculate_from(step.phase, failure);
        }

        StepVerificationResult::accepted(step.phase)
    }
}

impl<C> GlobalReasoningVerifier for ReasoningExecution<C>
where
    C: DurableRuleChecker,
{
    fn verify_global(&self) -> GlobalVerificationReport {
        let mut failures = Vec::new();

        if self.results.len() != self.plan.steps.len() {
            failures.push(GlobalFailure {
                kind: GlobalFailureKind::IncompleteReasoning,
                message: format!(
                    "reasoning is incomplete: expected {} results, got {}",
                    self.plan.steps.len(),
                    self.results.len()
                ),
            });
        }

        if self.verifications.iter().any(|v| {
            matches!(
                v.failure,
                Some(VerificationFailure {
                    kind: VerificationFailureKind::InternalInconsistency,
                    ..
                })
            )
        }) {
            failures.push(GlobalFailure {
                kind: GlobalFailureKind::CrossStepInconsistency,
                message: "one or more steps are internally inconsistent".to_string(),
            });
        }

        if self.verifications.iter().any(|v| {
            matches!(
                v.failure,
                Some(VerificationFailure {
                    kind: VerificationFailureKind::DurableRuleContradiction,
                    ..
                })
            )
        }) {
            failures.push(GlobalFailure {
                kind: GlobalFailureKind::DurableRuleContradiction,
                message: "a potential contradiction with durable rules was detected".to_string(),
            });
        }

        let hypothesis = self.result_for_phase(ReasoningPhase::Hypothesis);
        let action = self.result_for_phase(ReasoningPhase::ActionOrTest);
        let validation = self.result_for_phase(ReasoningPhase::Validation);

        if let (Some(h), Some(a), Some(v)) = (hypothesis, action, validation) {
            let ha = text_overlap_ratio(&h.output, &a.output);
            let av = text_overlap_ratio(&a.output, &v.output);
            if ha == 0.0 || av == 0.0 {
                failures.push(GlobalFailure {
                    kind: GlobalFailureKind::HypothesisActionValidationMisalignment,
                    message: "hypothesis/action/validation chain is not semantically aligned"
                        .to_string(),
                });
            }
        }

        let penalty = (failures.len() as f32) * 0.25;
        let score = (1.0 - penalty).max(0.0);
        let confidence = if score >= 0.85 {
            ConfidenceLevel::High
        } else if score >= 0.60 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        };

        let passed = failures.is_empty();
        let decision = if passed {
            GlobalDecision::Accept
        } else {
            GlobalDecision::NeedsRevision
        };

        GlobalVerificationReport {
            passed,
            score,
            confidence,
            failures,
            decision,
        }
    }
}

impl<C> ReasoningExecution<C>
where
    C: DurableRuleChecker,
{
    fn result_for_phase(&self, phase: ReasoningPhase) -> Option<&StepResult> {
        self.results.iter().find(|r| r.phase == phase)
    }
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn text_overlap_ratio(expected: &str, output: &str) -> f32 {
    let expected_tokens = tokenize(expected);
    if expected_tokens.is_empty() {
        return 1.0;
    }

    let output_tokens = tokenize(output);
    if output_tokens.is_empty() {
        return 0.0;
    }

    let overlap = expected_tokens
        .iter()
        .filter(|token| output_tokens.contains(*token))
        .count();

    overlap as f32 / expected_tokens.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_plan() -> ReasoningPlan {
        ReasoningPlan::new(
            "Diagnose rust borrow checker issue",
            vec![
                ReasoningStep::new(
                    ReasoningPhase::Analyze,
                    "Read compiler error and context",
                    "problem statement",
                ),
                ReasoningStep::new(
                    ReasoningPhase::Hypothesis,
                    "Propose likely root cause",
                    "root cause hypothesis",
                ),
                ReasoningStep::new(
                    ReasoningPhase::ActionOrTest,
                    "Run focused code change/test",
                    "test result",
                ),
                ReasoningStep::new(
                    ReasoningPhase::Validation,
                    "Verify with tests and constraints",
                    "validation report",
                ),
            ],
        )
    }

    #[test]
    fn builds_valid_reasoning_plan() {
        let plan = valid_plan();
        assert!(plan.validate_structure().is_ok());
    }

    #[test]
    fn rejects_incomplete_reasoning_plan() {
        let plan = ReasoningPlan::new(
            "Incomplete",
            vec![ReasoningStep::new(
                ReasoningPhase::Analyze,
                "Only analyze",
                "analysis",
            )],
        );

        let err = plan
            .validate_structure()
            .expect_err("must reject incomplete plan");
        assert!(matches!(err, ValidationFailure::IncompletePlan { .. }));
    }

    #[test]
    fn rejects_invalid_step_order() {
        let plan = ReasoningPlan::new(
            "Invalid order",
            vec![
                ReasoningStep::new(ReasoningPhase::Hypothesis, "h", "h"),
                ReasoningStep::new(ReasoningPhase::Analyze, "a", "a"),
                ReasoningStep::new(ReasoningPhase::ActionOrTest, "t", "t"),
                ReasoningStep::new(ReasoningPhase::Validation, "v", "v"),
            ],
        );

        let err = plan
            .validate_structure()
            .expect_err("must reject invalid order");
        assert!(matches!(err, ValidationFailure::InvalidStepOrder { .. }));
    }

    #[derive(Debug, Default, Clone)]
    struct ContradictionChecker;

    impl DurableRuleChecker for ContradictionChecker {
        fn detect_contradiction(
            &self,
            _step: &ReasoningStep,
            result: &StepResult,
        ) -> Option<String> {
            if result.output.contains("violate:core-decoupling") {
                return Some("core-decoupling".to_string());
            }
            None
        }
    }

    #[test]
    fn valid_path_without_correction() {
        let mut execution = ReasoningExecution::new(valid_plan(), NoopDurableRuleChecker)
            .expect("valid plan should initialize execution");

        let v1 = execution
            .verify_and_push(StepResult::succeeded(
                ReasoningPhase::Analyze,
                "problem statement extracted",
            ))
            .expect("step should verify");
        assert!(v1.passed);
        assert_eq!(v1.decision, CorrectionDecision::Accept);

        let v2 = execution
            .verify_and_push(StepResult::succeeded(
                ReasoningPhase::Hypothesis,
                "root cause hypothesis: overlapping borrow",
            ))
            .expect("step should verify");
        assert!(v2.passed);

        let v3 = execution
            .verify_and_push(StepResult::succeeded(
                ReasoningPhase::ActionOrTest,
                "test result: compile succeeds",
            ))
            .expect("step should verify");
        assert!(v3.passed);

        let v4 = execution
            .verify_and_push(StepResult::succeeded(
                ReasoningPhase::Validation,
                "validation report: tests pass",
            ))
            .expect("step should verify");
        assert!(v4.passed);
        assert_eq!(execution.results.len(), 4);
    }

    #[test]
    fn detects_internal_inconsistency() {
        let mut execution = ReasoningExecution::new(valid_plan(), NoopDurableRuleChecker)
            .expect("valid plan should initialize execution");

        let verification = execution
            .verify_and_push(StepResult::succeeded(
                ReasoningPhase::Analyze,
                "totally unrelated output",
            ))
            .expect("verification should run");

        assert!(!verification.passed);
        assert!(matches!(
            verification.failure,
            Some(VerificationFailure {
                kind: VerificationFailureKind::InternalInconsistency,
                ..
            })
        ));
        assert_eq!(execution.results.len(), 0);
    }

    #[test]
    fn detects_contradiction_with_durable_rule() {
        let mut execution = ReasoningExecution::new(valid_plan(), ContradictionChecker)
            .expect("valid plan should initialize execution");

        let _ = execution
            .verify_and_push(StepResult::succeeded(
                ReasoningPhase::Analyze,
                "problem statement complete",
            ))
            .expect("first step should pass");

        let verification = execution
            .verify_and_push(StepResult::succeeded(
                ReasoningPhase::Hypothesis,
                "root cause hypothesis violate:core-decoupling",
            ))
            .expect("verification should run");

        assert!(!verification.passed);
        assert!(matches!(
            verification.failure,
            Some(VerificationFailure {
                kind: VerificationFailureKind::DurableRuleContradiction,
                ..
            })
        ));
    }

    #[test]
    fn produces_correction_decision_on_invalid_step() {
        let mut execution = ReasoningExecution::new(valid_plan(), NoopDurableRuleChecker)
            .expect("valid plan should initialize execution");

        let verification = execution
            .verify_and_push(StepResult {
                phase: ReasoningPhase::Analyze,
                output: String::new(),
                status: StepStatus::Failed,
            })
            .expect("verification should run");

        assert_eq!(
            verification.decision,
            CorrectionDecision::ReviseStep {
                phase: ReasoningPhase::Analyze,
                reason: VerificationFailure {
                    kind: VerificationFailureKind::InvalidOrInsufficientStep,
                    message: "step output is empty or failed".to_string(),
                },
            }
        );
    }

    #[test]
    fn global_verification_accepts_coherent_reasoning() {
        let mut execution = ReasoningExecution::new(valid_plan(), NoopDurableRuleChecker)
            .expect("valid plan should initialize execution");

        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::Analyze,
            "problem statement extracted",
        ));
        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::Hypothesis,
            "root cause hypothesis: overlapping borrow",
        ));
        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::ActionOrTest,
            "test result confirms root cause hypothesis",
        ));
        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::Validation,
            "validation report confirms test result and hypothesis",
        ));

        let report = execution.verify_global();
        assert!(report.passed);
        assert_eq!(report.decision, GlobalDecision::Accept);
        assert_eq!(report.confidence, ConfidenceLevel::High);
    }

    #[test]
    fn global_verification_detects_incomplete_reasoning() {
        let mut execution = ReasoningExecution::new(valid_plan(), NoopDurableRuleChecker)
            .expect("valid plan should initialize execution");

        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::Analyze,
            "problem statement extracted",
        ));

        let report = execution.verify_global();
        assert!(!report.passed);
        assert!(report
            .failures
            .iter()
            .any(|f| f.kind == GlobalFailureKind::IncompleteReasoning));
    }

    #[test]
    fn global_verification_detects_durable_rule_contradiction() {
        let mut execution = ReasoningExecution::new(valid_plan(), ContradictionChecker)
            .expect("valid plan should initialize execution");

        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::Analyze,
            "problem statement complete",
        ));
        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::Hypothesis,
            "root cause hypothesis violate:core-decoupling",
        ));

        let report = execution.verify_global();
        assert!(report
            .failures
            .iter()
            .any(|f| f.kind == GlobalFailureKind::DurableRuleContradiction));
    }

    #[test]
    fn global_verification_can_request_revision() {
        let mut execution = ReasoningExecution::new(valid_plan(), NoopDurableRuleChecker)
            .expect("valid plan should initialize execution");

        let _ = execution.verify_and_push(StepResult::succeeded(
            ReasoningPhase::Analyze,
            "totally unrelated output",
        ));

        let report = execution.verify_global();
        assert_eq!(report.decision, GlobalDecision::NeedsRevision);
        assert!(!report.passed);
    }
}
