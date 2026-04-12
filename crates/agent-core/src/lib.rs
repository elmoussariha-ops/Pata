use std::collections::HashMap;
use std::sync::Mutex;

use agent_memory::{
    HeuristicSummarizer, InteractionRecord, MemoryEngine, MemoryRetriever, PermanentMemoryKind,
    RetrievalIntent, RetrievalQuery,
};
use agent_reasoning::{
    DurableRuleChecker, GlobalDecision, GlobalReasoningVerifier, ReasoningExecution,
    ReasoningPhase, ReasoningPlan, ReasoningStep, StepResult,
};
use agent_traits::{
    Agent, AgentEvent, AgentResult, ExecutionContext, ModelProvider, Persona, Tool, ToolCall,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Serialize;
use serde_json::json;
use tracing::instrument;

/// Event severity for pipeline observability.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum EventLevel {
    Info,
    Warning,
}

/// Event kinds emitted by orchestrated execution.
#[derive(Debug, Clone, Serialize)]
pub enum PipelineEventType {
    ExecutionStarted,
    MemoryRetrieved,
    ReasoningPlanPrepared,
    ReasoningStepExecuted,
    LocalVerificationCompleted,
    CorrectionAttemptStarted,
    CorrectionAttemptSucceeded,
    CorrectionAttemptFailed,
    GlobalVerificationCompleted,
    CorrectionDecisionIssued,
    FinalResultProduced,
}

/// Structured pipeline event.
#[derive(Debug, Clone, Serialize)]
pub struct PipelineEvent {
    pub index: usize,
    pub level: EventLevel,
    pub event_type: PipelineEventType,
    pub detail: String,
}

/// Full execution trace for one orchestrated run.
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionTrace {
    pub run_id: String,
    pub events: Vec<PipelineEvent>,
}

/// Aggregated summary for one orchestrated run.
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionSummary {
    pub verification_status: String,
    pub confidence: f32,
    pub reasoning_steps_executed: usize,
    pub local_verifications: usize,
    pub global_failures: Vec<String>,
}

/// One correction attempt recorded during local retry handling.
#[derive(Debug, Clone, Serialize)]
pub struct CorrectionAttemptRecord {
    pub phase: String,
    pub attempt_number: usize,
    pub previous_failure_reason: String,
    pub succeeded: bool,
}

/// Quality dimensions used for deterministic pipeline evaluation.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum EvaluationDimension {
    Completeness,
    ReasoningCoherence,
    VerificationStatus,
    ConfidenceLevel,
    FlowRobustness,
}

/// One scored quality dimension.
#[derive(Debug, Clone, Serialize)]
pub struct DimensionScore {
    pub dimension: EvaluationDimension,
    pub score: f32,
    pub note: String,
}

/// Evaluation case descriptor for a single orchestrated run.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationCase {
    pub case_id: String,
    pub minimum_overall_score: f32,
    pub require_accept_verification: bool,
}

/// Evaluation result for one case.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationResult {
    pub case_id: String,
    pub passed: bool,
    pub overall_score: f32,
    pub verification_status: String,
    pub confidence: f32,
    pub dimension_scores: Vec<DimensionScore>,
}

/// Aggregated summary across evaluation results.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationSummary {
    pub total_cases: usize,
    pub passed_cases: usize,
    pub average_score: f32,
    pub average_by_dimension: Vec<DimensionScore>,
}

/// Deterministic evaluator for orchestrated agent outputs.
pub struct DeterministicPipelineEvaluator;

impl DeterministicPipelineEvaluator {
    pub fn evaluate_case(case: &EvaluationCase, result: &AgentResult) -> EvaluationResult {
        let structured = result.structured_output.clone().unwrap_or_default();
        let verification_status = structured["verification_status"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let reasoning_steps = structured["reasoning_steps_executed"].as_u64().unwrap_or(0) as f32;
        let local_verifications = structured["local_verifications"].as_u64().unwrap_or(0) as f32;
        let global_failures_count = structured["global_failures"]
            .as_array()
            .map(|x| x.len())
            .unwrap_or(0) as f32;

        let completeness = if !result.answer.trim().is_empty() && reasoning_steps > 0.0 {
            1.0
        } else {
            0.2
        };

        let reasoning_coherence = if global_failures_count == 0.0 {
            1.0
        } else {
            0.4
        };
        let verification_score = if verification_status == "Accept" {
            1.0
        } else {
            0.3
        };
        let confidence_score = result.confidence.clamp(0.0, 1.0);
        let robustness = ((local_verifications / (reasoning_steps.max(1.0))) + verification_score)
            .clamp(0.0, 1.0)
            / 2.0;

        let dimension_scores = vec![
            DimensionScore {
                dimension: EvaluationDimension::Completeness,
                score: completeness,
                note: "final content presence and minimal reasoning execution".to_string(),
            },
            DimensionScore {
                dimension: EvaluationDimension::ReasoningCoherence,
                score: reasoning_coherence,
                note: "absence/presence of global verification failures".to_string(),
            },
            DimensionScore {
                dimension: EvaluationDimension::VerificationStatus,
                score: verification_score,
                note: "Accept vs NeedsRevision".to_string(),
            },
            DimensionScore {
                dimension: EvaluationDimension::ConfidenceLevel,
                score: confidence_score,
                note: "confidence score emitted by orchestrated pipeline".to_string(),
            },
            DimensionScore {
                dimension: EvaluationDimension::FlowRobustness,
                score: robustness,
                note: "ratio of local verifications over executed steps + verification status"
                    .to_string(),
            },
        ];

        let overall_score =
            dimension_scores.iter().map(|d| d.score).sum::<f32>() / dimension_scores.len() as f32;

        let verification_ok = !case.require_accept_verification || verification_status == "Accept";
        let passed = verification_ok && overall_score >= case.minimum_overall_score;

        EvaluationResult {
            case_id: case.case_id.clone(),
            passed,
            overall_score,
            verification_status,
            confidence: result.confidence,
            dimension_scores,
        }
    }

    pub fn summarize(results: &[EvaluationResult]) -> EvaluationSummary {
        let total_cases = results.len();
        let passed_cases = results.iter().filter(|r| r.passed).count();
        let average_score = if total_cases == 0 {
            0.0
        } else {
            results.iter().map(|r| r.overall_score).sum::<f32>() / total_cases as f32
        };

        let mut by_dimension: HashMap<EvaluationDimension, Vec<f32>> = HashMap::new();
        for result in results {
            for dimension in &result.dimension_scores {
                by_dimension
                    .entry(dimension.dimension)
                    .or_default()
                    .push(dimension.score);
            }
        }

        let average_by_dimension = by_dimension
            .into_iter()
            .map(|(dimension, scores)| DimensionScore {
                dimension,
                score: scores.iter().sum::<f32>() / scores.len() as f32,
                note: "average score across evaluated cases".to_string(),
            })
            .collect::<Vec<_>>();

        EvaluationSummary {
            total_cases,
            passed_cases,
            average_score,
            average_by_dimension,
        }
    }
}

#[derive(Debug, Default)]
struct TraceRecorder {
    events: Vec<PipelineEvent>,
}

impl TraceRecorder {
    fn push(
        &mut self,
        level: EventLevel,
        event_type: PipelineEventType,
        detail: impl Into<String>,
    ) {
        let index = self.events.len();
        self.events.push(PipelineEvent {
            index,
            level,
            event_type,
            detail: detail.into(),
        });
    }
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.spec().name.clone(), Box::new(tool));
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub async fn invoke(
        &self,
        call: ToolCall,
        context: &ExecutionContext,
    ) -> Result<agent_traits::ToolOutput> {
        let tool = self
            .tools
            .get(&call.name)
            .ok_or_else(|| anyhow!("unknown tool: {}", call.name))?;

        tool.run(call.input, context).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SimpleAgent<P, M> {
    pub persona: P,
    pub model: M,
    pub tools: ToolRegistry,
}

impl<P, M> SimpleAgent<P, M>
where
    P: Persona,
    M: ModelProvider,
{
    pub fn new(persona: P, model: M, tools: ToolRegistry) -> Self {
        Self {
            persona,
            model,
            tools,
        }
    }
}

#[async_trait]
impl<P, M> Agent for SimpleAgent<P, M>
where
    P: Persona,
    M: ModelProvider,
{
    #[instrument(skip(self, context), fields(persona = self.persona.name(), model = self.model.name()))]
    async fn run(&self, goal: &str, context: ExecutionContext) -> Result<AgentResult> {
        let allowed_tools = self.persona.allowed_tools();

        for tool in &allowed_tools {
            if !self.tools.contains(tool) {
                return Err(anyhow!("persona expects unavailable tool: {tool}"));
            }
        }

        let draft = self
            .model
            .complete(&self.persona.system_prompt(), goal, &context)
            .await?;

        self.persona
            .validate(&draft)
            .map_err(|err| anyhow!(err.to_string()))?;

        Ok(AgentResult {
            answer: draft,
            confidence: 0.5,
            structured_output: None,
            events: vec![
                AgentEvent::PlanCreated {
                    steps: vec![
                        "Analyse goal".to_string(),
                        "Generate first draft".to_string(),
                        "Validate output".to_string(),
                    ],
                },
                AgentEvent::ValidationPassed,
            ],
        })
    }
}

/// Lightweight orchestrated agent using memory + structured reasoning + verification.
pub struct OrchestratedAgent<P, M> {
    pub persona: P,
    pub model: M,
    pub tools: ToolRegistry,
    pub memory: Mutex<MemoryEngine>,
}

impl<P, M> OrchestratedAgent<P, M>
where
    P: Persona,
    M: ModelProvider,
{
    pub fn new(persona: P, model: M, tools: ToolRegistry) -> Result<Self> {
        Ok(Self {
            persona,
            model,
            tools,
            memory: Mutex::new(MemoryEngine::new(12, HeuristicSummarizer)?),
        })
    }

    fn build_reasoning_plan(goal: &str) -> ReasoningPlan {
        ReasoningPlan::new(
            goal,
            vec![
                ReasoningStep::new(
                    ReasoningPhase::Analyze,
                    "Clarify user objective and constraints",
                    "problem statement",
                ),
                ReasoningStep::new(
                    ReasoningPhase::Hypothesis,
                    "Propose best approach",
                    "root cause hypothesis",
                ),
                ReasoningStep::new(
                    ReasoningPhase::ActionOrTest,
                    "Propose concrete action or test",
                    "test result",
                ),
                ReasoningStep::new(
                    ReasoningPhase::Validation,
                    "Validate with checks and durable rules",
                    "validation report",
                ),
            ],
        )
    }
}

#[derive(Debug, Clone)]
struct DurableMemoryChecker {
    rules: Vec<String>,
}

impl DurableRuleChecker for DurableMemoryChecker {
    fn detect_contradiction(&self, _step: &ReasoningStep, result: &StepResult) -> Option<String> {
        let lowered = result.output.to_lowercase();
        self.rules
            .iter()
            .find(|rule| lowered.contains(&format!("violate:{}", rule.to_lowercase())))
            .cloned()
    }
}

#[async_trait]
impl<P, M> Agent for OrchestratedAgent<P, M>
where
    P: Persona,
    M: ModelProvider,
{
    #[instrument(skip(self, context), fields(persona = self.persona.name(), model = self.model.name()))]
    async fn run(&self, goal: &str, context: ExecutionContext) -> Result<AgentResult> {
        let mut trace = TraceRecorder::default();
        trace.push(
            EventLevel::Info,
            PipelineEventType::ExecutionStarted,
            format!("run started for goal: {goal}"),
        );

        let allowed_tools = self.persona.allowed_tools();
        for tool in &allowed_tools {
            if !self.tools.contains(tool) {
                return Err(anyhow!("persona expects unavailable tool: {tool}"));
            }
        }

        let (retrieved_context, durable_rules) = {
            let memory = self
                .memory
                .lock()
                .map_err(|_| anyhow!("memory lock poisoned"))?;
            let hits = memory.retrieve(&RetrievalQuery::new(RetrievalIntent::Balanced, goal, 5));
            let context_lines = hits.into_iter().map(|h| h.value).collect::<Vec<_>>();

            let rules = memory
                .permanent
                .all()
                .into_iter()
                .filter(|e| {
                    matches!(
                        e.kind,
                        PermanentMemoryKind::SystemRule | PermanentMemoryKind::ArchitectureDecision
                    )
                })
                .map(|e| e.key.clone())
                .collect::<Vec<_>>();

            (context_lines, rules)
        };
        trace.push(
            EventLevel::Info,
            PipelineEventType::MemoryRetrieved,
            format!(
                "retrieved {} context items and {} durable rules",
                retrieved_context.len(),
                durable_rules.len()
            ),
        );

        let plan = Self::build_reasoning_plan(goal);
        trace.push(
            EventLevel::Info,
            PipelineEventType::ReasoningPlanPrepared,
            format!("prepared plan with {} steps", plan.steps.len()),
        );
        let mut execution = ReasoningExecution::new(
            plan.clone(),
            DurableMemoryChecker {
                rules: durable_rules,
            },
        )?;

        const MAX_LOCAL_RETRIES: usize = 2;
        let mut step_retry_count: HashMap<String, usize> = HashMap::new();
        let mut correction_attempts = 0usize;
        let mut correction_history: Vec<CorrectionAttemptRecord> = Vec::new();
        let mut final_failure_phase: Option<String> = None;
        'steps: for step in &plan.steps {
            let mut attempt = 0usize;
            let mut correction_feedback: Option<String> = None;

            loop {
                trace.push(
                    EventLevel::Info,
                    PipelineEventType::ReasoningStepExecuted,
                    format!(
                        "executing step {:?} (attempt {}/{})",
                        step.phase,
                        attempt + 1,
                        MAX_LOCAL_RETRIES + 1
                    ),
                );
                let prompt = format!(
                    "Goal: {goal}\nPersona: {}\nStep: {:?}\nStep objective: {}\nExpected artifact: {}\nMemory context: {:?}\nCorrection feedback: {}",
                    self.persona.name(),
                    step.phase,
                    step.goal,
                    step.expected_artifact,
                    retrieved_context,
                    correction_feedback
                        .clone()
                        .unwrap_or_else(|| "none".to_string())
                );

                let output = self
                    .model
                    .complete(&self.persona.system_prompt(), &prompt, &context)
                    .await?;

                let verification =
                    execution.verify_and_push(StepResult::succeeded(step.phase, output))?;
                trace.push(
                    EventLevel::Info,
                    PipelineEventType::LocalVerificationCompleted,
                    format!(
                        "local verification for {:?} attempt {}: passed={}",
                        step.phase,
                        attempt + 1,
                        verification.passed
                    ),
                );
                if verification.passed {
                    break;
                }

                trace.push(
                    EventLevel::Warning,
                    PipelineEventType::CorrectionDecisionIssued,
                    format!("decision: {:?}", verification.decision),
                );

                if attempt >= MAX_LOCAL_RETRIES {
                    final_failure_phase = Some(format!("{:?}", step.phase));
                    trace.push(
                        EventLevel::Warning,
                        PipelineEventType::CorrectionAttemptFailed,
                        format!("retry budget exhausted for phase {:?}", step.phase),
                    );
                    break 'steps;
                }

                attempt += 1;
                let failure_reason = verification
                    .failure
                    .as_ref()
                    .map(|failure| format!("{:?} - {}", failure.kind, failure.message))
                    .unwrap_or_else(|| "unknown local verification failure".to_string());
                correction_feedback = Some(failure_reason.clone());
                correction_attempts += 1;
                step_retry_count
                    .entry(format!("{:?}", step.phase))
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                trace.push(
                    EventLevel::Info,
                    PipelineEventType::CorrectionAttemptStarted,
                    format!(
                        "starting correction attempt {} for phase {:?}: {}",
                        attempt + 1,
                        step.phase,
                        failure_reason
                    ),
                );
                correction_history.push(CorrectionAttemptRecord {
                    phase: format!("{:?}", step.phase),
                    attempt_number: attempt + 1,
                    previous_failure_reason: failure_reason,
                    succeeded: false,
                });
            }

            if attempt > 0 {
                trace.push(
                    EventLevel::Info,
                    PipelineEventType::CorrectionAttemptSucceeded,
                    format!(
                        "correction attempt {} succeeded for phase {:?}",
                        attempt + 1,
                        step.phase
                    ),
                );
                if let Some(last) = correction_history.last_mut() {
                    if last.phase == format!("{:?}", step.phase) {
                        last.succeeded = true;
                    }
                }
            }
        }

        let global_report = execution.verify_global();
        trace.push(
            EventLevel::Info,
            PipelineEventType::GlobalVerificationCompleted,
            format!(
                "global verification decision={:?}, score={:.2}",
                global_report.decision, global_report.score
            ),
        );
        if matches!(global_report.decision, GlobalDecision::NeedsRevision) {
            trace.push(
                EventLevel::Warning,
                PipelineEventType::CorrectionDecisionIssued,
                "global decision requires revision".to_string(),
            );
        }

        let final_content = execution
            .results
            .iter()
            .find(|r| r.phase == ReasoningPhase::Validation)
            .map(|r| r.output.clone())
            .or_else(|| execution.results.last().map(|r| r.output.clone()))
            .unwrap_or_else(|| "Révision requise avant réponse finale.".to_string());

        self.persona
            .validate(&final_content)
            .map_err(|err| anyhow!(err.to_string()))?;

        {
            let mut memory = self
                .memory
                .lock()
                .map_err(|_| anyhow!("memory lock poisoned"))?;
            memory.ingest_interaction(InteractionRecord::new(goal, &final_content));
        }
        trace.push(
            EventLevel::Info,
            PipelineEventType::FinalResultProduced,
            "final result assembled and interaction stored".to_string(),
        );

        let summary = ExecutionSummary {
            verification_status: format!("{:?}", global_report.decision),
            confidence: global_report.score,
            reasoning_steps_executed: execution.results.len(),
            local_verifications: execution.verifications.len(),
            global_failures: global_report
                .failures
                .iter()
                .map(|f| format!("{:?}: {}", f.kind, f.message))
                .collect(),
        };
        let execution_trace = ExecutionTrace {
            run_id: context.run_id.to_string(),
            events: trace.events,
        };

        let make_structured_payload = |summary: &ExecutionSummary, trace: &ExecutionTrace| {
            json!({
                "verification_status": summary.verification_status,
                "confidence_level": format!("{:?}", global_report.confidence),
                "reasoning_steps_executed": summary.reasoning_steps_executed,
                "local_verifications": summary.local_verifications,
                "global_failures": summary.global_failures,
                "step_retry_count": step_retry_count,
                "correction_attempts": correction_attempts,
                "final_failure_phase": final_failure_phase,
                "correction_history": correction_history,
                "execution_summary": summary,
                "execution_trace": trace,
            })
        };

        let provisional_result = AgentResult {
            answer: final_content.clone(),
            confidence: global_report.score,
            structured_output: Some(make_structured_payload(&summary, &execution_trace)),
            events: Vec::new(),
        };

        let self_eval_case = EvaluationCase {
            case_id: "default-orchestrated-run".to_string(),
            minimum_overall_score: 0.6,
            require_accept_verification: false,
        };
        let self_evaluation =
            DeterministicPipelineEvaluator::evaluate_case(&self_eval_case, &provisional_result);

        Ok(AgentResult {
            answer: final_content,
            confidence: global_report.score,
            structured_output: Some({
                let mut payload = make_structured_payload(&summary, &execution_trace);
                payload["evaluation"] = serde_json::to_value(self_evaluation)
                    .unwrap_or_else(|_| json!({"error": "evaluation serialization failed"}));
                payload
            }),
            events: vec![
                AgentEvent::PlanCreated {
                    steps: plan
                        .steps
                        .iter()
                        .map(|s| format!("{:?}", s.phase))
                        .collect(),
                },
                if matches!(global_report.decision, GlobalDecision::Accept) {
                    AgentEvent::ValidationPassed
                } else {
                    AgentEvent::ToolReturned {
                        tool: "reasoning-revision-required".to_string(),
                    }
                },
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[derive(Debug, Clone)]
    struct TestPersona;

    impl Persona for TestPersona {
        fn name(&self) -> &'static str {
            "test"
        }

        fn system_prompt(&self) -> String {
            "You are a deterministic test persona".to_string()
        }

        fn allowed_tools(&self) -> Vec<String> {
            Vec::new()
        }

        fn validate(&self, draft: &str) -> std::result::Result<(), agent_traits::AgentError> {
            if draft.trim().is_empty() {
                return Err(agent_traits::AgentError::Validation(
                    "empty response".to_string(),
                ));
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    struct TestModel;

    #[async_trait]
    impl ModelProvider for TestModel {
        fn name(&self) -> &'static str {
            "test-model"
        }

        async fn complete(
            &self,
            _system_prompt: &str,
            user_prompt: &str,
            _context: &ExecutionContext,
        ) -> Result<String> {
            if user_prompt.contains("Analyze") {
                return Ok("problem statement extracted".to_string());
            }
            if user_prompt.contains("Hypothesis") {
                return Ok("root cause hypothesis: ownership mismatch".to_string());
            }
            if user_prompt.contains("ActionOrTest") {
                return Ok("test result confirms root cause hypothesis".to_string());
            }
            Ok("validation report confirms test result and hypothesis".to_string())
        }
    }

    #[derive(Debug)]
    struct RetryModel {
        fail_analyze_always: bool,
        attempts_by_phase: Mutex<HashMap<String, usize>>,
    }

    impl RetryModel {
        fn new(fail_analyze_always: bool) -> Self {
            Self {
                fail_analyze_always,
                attempts_by_phase: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ModelProvider for RetryModel {
        fn name(&self) -> &'static str {
            "retry-model"
        }

        async fn complete(
            &self,
            _system_prompt: &str,
            user_prompt: &str,
            _context: &ExecutionContext,
        ) -> Result<String> {
            let phase = if user_prompt.contains("Step: Analyze") {
                "Analyze"
            } else if user_prompt.contains("Step: Hypothesis") {
                "Hypothesis"
            } else if user_prompt.contains("Step: ActionOrTest") {
                "ActionOrTest"
            } else {
                "Validation"
            };

            let mut attempts = self
                .attempts_by_phase
                .lock()
                .map_err(|_| anyhow!("attempts lock poisoned"))?;
            let current = attempts.entry(phase.to_string()).or_insert(0);
            *current += 1;
            let attempt_number = *current;
            drop(attempts);

            if phase == "Analyze" {
                if self.fail_analyze_always {
                    return Ok("content intentionally incoherent".to_string());
                }
                if attempt_number == 1 {
                    return Ok("content intentionally incoherent".to_string());
                }
                return Ok("problem statement extracted".to_string());
            }

            if phase == "Hypothesis" {
                return Ok("root cause hypothesis: ownership mismatch".to_string());
            }
            if phase == "ActionOrTest" {
                return Ok("test result confirms root cause hypothesis".to_string());
            }
            Ok("validation report confirms test result and hypothesis".to_string())
        }
    }

    #[tokio::test]
    async fn orchestrated_flow_nominal_end_to_end() {
        let agent = OrchestratedAgent::new(TestPersona, TestModel, ToolRegistry::new())
            .expect("agent should initialize");

        let result = agent
            .run("Fix rust compile error", ExecutionContext::default())
            .await
            .expect("orchestrated run should succeed");

        assert!(result.answer.contains("validation report"));
        assert!(result.confidence >= 0.75);
        let structured = result
            .structured_output
            .expect("structured output expected");
        assert_eq!(structured["verification_status"], "Accept");
        assert_eq!(structured["reasoning_steps_executed"], 4);

        let events = structured["execution_trace"]["events"]
            .as_array()
            .expect("trace events should be present");
        assert!(!events.is_empty());

        let event_types = events
            .iter()
            .map(|e| e["event_type"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();

        assert!(event_types.contains(&"ExecutionStarted".to_string()));
        assert!(event_types.contains(&"MemoryRetrieved".to_string()));
        assert!(event_types.contains(&"ReasoningPlanPrepared".to_string()));
        assert!(event_types.contains(&"GlobalVerificationCompleted".to_string()));
        assert!(event_types.contains(&"FinalResultProduced".to_string()));

        let pos = |name: &str| {
            event_types
                .iter()
                .position(|t| t == name)
                .expect("event must exist")
        };

        assert!(pos("ExecutionStarted") < pos("MemoryRetrieved"));
        assert!(pos("MemoryRetrieved") < pos("ReasoningPlanPrepared"));
        assert!(pos("ReasoningPlanPrepared") < pos("GlobalVerificationCompleted"));
        assert!(pos("GlobalVerificationCompleted") < pos("FinalResultProduced"));
    }

    #[tokio::test]
    async fn retry_success_visible_in_structured_output() {
        let model = RetryModel::new(false);
        let agent = OrchestratedAgent::new(TestPersona, model, ToolRegistry::new())
            .expect("agent should initialize");

        let result = agent
            .run("Fix rust compile error", ExecutionContext::default())
            .await
            .expect("orchestrated run should succeed");

        let structured = result
            .structured_output
            .expect("structured output expected");

        assert_eq!(structured["step_retry_count"]["Analyze"], 1);
        assert_eq!(structured["correction_attempts"], 1);
        assert_eq!(structured["final_failure_phase"], serde_json::Value::Null);

        let history = structured["correction_history"]
            .as_array()
            .expect("correction history array expected");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0]["phase"], "Analyze");
        assert_eq!(history[0]["attempt_number"], 2);
        assert_eq!(history[0]["succeeded"], true);
    }

    #[tokio::test]
    async fn retry_failure_visible_in_structured_output() {
        let model = RetryModel::new(true);
        let agent = OrchestratedAgent::new(TestPersona, model, ToolRegistry::new())
            .expect("agent should initialize");

        let result = agent
            .run("Fix rust compile error", ExecutionContext::default())
            .await
            .expect("orchestrated run should complete with revision required");

        let structured = result
            .structured_output
            .expect("structured output expected");
        assert_eq!(structured["verification_status"], "NeedsRevision");
        assert_eq!(structured["step_retry_count"]["Analyze"], 2);
        assert_eq!(structured["correction_attempts"], 2);

        let history = structured["correction_history"]
            .as_array()
            .expect("correction history array expected");
        assert_eq!(history.len(), 2);
        assert!(history.iter().all(|h| h["phase"] == "Analyze"));
        assert!(history.iter().all(|h| h["succeeded"] == false));
    }

    #[tokio::test]
    async fn correction_events_order() {
        let model = RetryModel::new(false);
        let agent = OrchestratedAgent::new(TestPersona, model, ToolRegistry::new())
            .expect("agent should initialize");

        let result = agent
            .run("Fix rust compile error", ExecutionContext::default())
            .await
            .expect("orchestrated run should succeed");
        let structured = result
            .structured_output
            .expect("structured output expected");

        let events = structured["execution_trace"]["events"]
            .as_array()
            .expect("trace events should be present");
        let event_types = events
            .iter()
            .map(|e| e["event_type"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();

        let started = event_types
            .iter()
            .position(|t| *t == "CorrectionAttemptStarted")
            .expect("correction attempt started event should exist");
        let succeeded = event_types
            .iter()
            .position(|t| *t == "CorrectionAttemptSucceeded")
            .expect("correction attempt succeeded event should exist");

        assert!(started < succeeded);
    }

    #[tokio::test]
    async fn final_failure_phase_present_when_retry_budget_exhausted() {
        let model = RetryModel::new(true);
        let agent = OrchestratedAgent::new(TestPersona, model, ToolRegistry::new())
            .expect("agent should initialize");

        let result = agent
            .run("Fix rust compile error", ExecutionContext::default())
            .await
            .expect("orchestrated run should complete");
        let structured = result
            .structured_output
            .expect("structured output expected");

        assert_eq!(structured["final_failure_phase"], "Analyze");

        let events = structured["execution_trace"]["events"]
            .as_array()
            .expect("trace events should be present");
        let has_failed_event = events
            .iter()
            .any(|e| e["event_type"].as_str() == Some("CorrectionAttemptFailed"));
        assert!(has_failed_event);
    }

    fn make_result_for_eval(
        answer: &str,
        confidence: f32,
        verification_status: &str,
        steps: u64,
        local_verifications: u64,
        global_failures: Vec<&str>,
    ) -> AgentResult {
        AgentResult {
            answer: answer.to_string(),
            confidence,
            structured_output: Some(json!({
                "verification_status": verification_status,
                "reasoning_steps_executed": steps,
                "local_verifications": local_verifications,
                "global_failures": global_failures,
            })),
            events: Vec::new(),
        }
    }

    #[test]
    fn evaluation_nominal_case_passes() {
        let case = EvaluationCase {
            case_id: "nominal".to_string(),
            minimum_overall_score: 0.7,
            require_accept_verification: true,
        };
        let result = make_result_for_eval("final answer", 0.9, "Accept", 4, 4, vec![]);
        let eval = DeterministicPipelineEvaluator::evaluate_case(&case, &result);

        assert!(eval.passed);
        assert!(eval.overall_score >= 0.7);
    }

    #[test]
    fn evaluation_low_confidence_case_is_penalized() {
        let case = EvaluationCase {
            case_id: "low-confidence".to_string(),
            minimum_overall_score: 0.8,
            require_accept_verification: false,
        };
        let result = make_result_for_eval("final answer", 0.2, "Accept", 4, 4, vec![]);
        let eval = DeterministicPipelineEvaluator::evaluate_case(&case, &result);

        assert!(!eval.passed);
        let confidence_dim = eval
            .dimension_scores
            .iter()
            .find(|d| d.dimension == EvaluationDimension::ConfidenceLevel)
            .expect("confidence dimension must exist");
        assert!(confidence_dim.score <= 0.2);
    }

    #[test]
    fn evaluation_revision_required_case_fails_when_required() {
        let case = EvaluationCase {
            case_id: "must-accept".to_string(),
            minimum_overall_score: 0.4,
            require_accept_verification: true,
        };
        let result = make_result_for_eval(
            "partial answer",
            0.8,
            "NeedsRevision",
            3,
            3,
            vec!["IncompleteReasoning: missing validation"],
        );
        let eval = DeterministicPipelineEvaluator::evaluate_case(&case, &result);

        assert!(!eval.passed);
        assert_eq!(eval.verification_status, "NeedsRevision");
    }

    #[test]
    fn evaluation_summary_is_coherent() {
        let case = EvaluationCase {
            case_id: "summary-case".to_string(),
            minimum_overall_score: 0.6,
            require_accept_verification: false,
        };

        let a = DeterministicPipelineEvaluator::evaluate_case(
            &case,
            &make_result_for_eval("ok", 0.9, "Accept", 4, 4, vec![]),
        );
        let b = DeterministicPipelineEvaluator::evaluate_case(
            &case,
            &make_result_for_eval("needs work", 0.3, "NeedsRevision", 2, 2, vec!["failure"]),
        );
        let summary = DeterministicPipelineEvaluator::summarize(&[a, b]);

        assert_eq!(summary.total_cases, 2);
        assert!(summary.average_score >= 0.0 && summary.average_score <= 1.0);
        assert!(!summary.average_by_dimension.is_empty());
    }
}
