use agent_core::ToolRegistry;
use agent_traits::{ExecutionContext, ModelProvider, Tool, ToolOutput, ToolSpec};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

pub const DETERMINISTIC_MODE: &str = "deterministic";
pub const MAX_GOAL_LEN: usize = 2_000;

#[derive(Debug, Clone)]
pub struct DeterministicModelProvider;

#[async_trait]
impl ModelProvider for DeterministicModelProvider {
    fn name(&self) -> &'static str {
        "deterministic-model"
    }

    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _context: &ExecutionContext,
    ) -> Result<String> {
        let is_teacher = system_prompt.contains("pedagogical AI teacher");
        let is_personal = system_prompt.contains("structured personal productivity assistant");
        let is_smb = system_prompt.contains("SMB operations copilot");

        if is_personal {
            return Ok(format!(
                "CONTEXT_SUMMARY: You want better weekly organization with limited time and energy.\n\
PRIMARY_OBJECTIVE: Stabilize personal planning and focus on essential priorities.\n\
ACTION_STRUCTURE: Plan weekly priorities on Sunday, set daily top-3 tasks, and review each evening.\n\
RISK_CHECK: Main constraint is limited evening energy; keep tasks small and review trade-offs.\n\
NEXT_STEP: Write tomorrow's top-3 tasks now and block 20 minutes for planning.\n\
FINAL_ANSWER: Use a lightweight routine, adjust weekly, and keep the plan realistic with constraints.\n\
TRACE_NOTE: {user_prompt}"
            ));
        }

        if is_smb {
            return Ok(format!(
                "BUSINESS_CONTEXT: Small business with limited staff capacity and marketing budget.\n\
OPERATIONAL_OBJECTIVE: Increase repeat customer visits in the next 30 days.\n\
ACTION_BACKLOG: 1) Daily follow-up messages, 2) Weekly offer bundle, 3) Inventory focus on top sellers.\n\
DECISION_SUPPORT: Assumption: retention actions will outperform broad paid ads under current budget.\n\
FOLLOW_UP_METRICS: Repeat visits/week, offer conversion rate, average basket size.\n\
FINAL_ANSWER: Execute low-cost retention actions first, track weekly metrics, and iterate quickly.\n\
TRACE_NOTE: {user_prompt}"
            ));
        }

        if user_prompt.contains("Analyze") {
            return if is_teacher {
                Ok("LEARNING_OBJECTIVE: Identify what the learner should understand from the topic."
                    .to_string())
            } else {
                Ok("ANALYSIS: Investigate error scope and constraints".to_string())
            };
        }
        if user_prompt.contains("Hypothesis") {
            return if is_teacher {
                Ok("LEVEL_ADAPTATION: beginner level, use plain language and one concept at a time."
                    .to_string())
            } else {
                Ok("HYPOTHESIS: Root cause likely ownership mismatch".to_string())
            };
        }
        if user_prompt.contains("ActionOrTest") {
            return if is_teacher {
                Ok("EXPLANATION: Explain core concept with one concrete example before adding nuance."
                    .to_string())
            } else {
                Ok(
                    "ACTION_PLAN: Apply scoped borrow refactor. VALIDATION: cargo check"
                        .to_string(),
                )
            };
        }
        if user_prompt.contains("Validation") {
            return if is_teacher {
                Ok("LEARNING_OBJECTIVE: Understand Rust ownership for function arguments.\n\
LEVEL_ADAPTATION: beginner audience, short sentences and minimal jargon.\n\
EXPLANATION: In Rust, each value has one owner; moving a value transfers ownership.\n\
GUIDED_PRACTICE: Compare a function taking String by value versus &str by reference.\n\
UNDERSTANDING_CHECK: Why can you still use a variable after borrowing but not after move?\n\
FINAL_ANSWER: Start with ownership, then borrowing, then lifetimes using one compiled example each."
                    .to_string())
            } else if is_personal {
                Ok("CONTEXT_SUMMARY: You want better weekly organization with limited time and energy.\n\
PRIMARY_OBJECTIVE: Stabilize personal planning and focus on essential priorities.\n\
ACTION_STRUCTURE: Plan weekly priorities on Sunday, set daily top-3 tasks, and review each evening.\n\
RISK_CHECK: Main constraint is limited evening energy; keep tasks small and review trade-offs.\n\
NEXT_STEP: Write tomorrow's top-3 tasks now and block 20 minutes for planning.\n\
FINAL_ANSWER: Use a lightweight routine, adjust weekly, and keep the plan realistic with constraints."
                    .to_string())
            } else if is_smb {
                Ok("BUSINESS_CONTEXT: Small business with limited staff capacity and marketing budget.\n\
OPERATIONAL_OBJECTIVE: Increase repeat customer visits in the next 30 days.\n\
ACTION_BACKLOG: 1) Daily follow-up messages, 2) Weekly offer bundle, 3) Inventory focus on top sellers.\n\
DECISION_SUPPORT: Assumption: retention actions will outperform broad paid ads under current budget.\n\
FOLLOW_UP_METRICS: Repeat visits/week, offer conversion rate, average basket size.\n\
FINAL_ANSWER: Execute low-cost retention actions first, track weekly metrics, and iterate quickly."
                    .to_string())
            } else {
                Ok("ANALYSIS: Investigate error scope and constraints.\n\
HYPOTHESIS: Root cause likely ownership mismatch.\n\
ACTION_PLAN: Apply scoped borrow refactor and run cargo check.\n\
VALIDATION: Verify no compiler errors and tests remain green.\n\
DURABLE_RULES_CHECK: No contradiction with project rules.\n\
FINAL_ANSWER: Apply scoped borrow refactor, re-run cargo test, then review diff."
                    .to_string())
            };
        }

        Ok("FINAL_ANSWER: deterministic fallback response.".to_string())
    }
}

#[derive(Debug, Clone)]
struct NoopTool {
    name: String,
}

#[async_trait]
impl Tool for NoopTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name.clone(),
            description: "No-op placeholder tool for local runtime".to_string(),
            input_schema: json!({"type": "object"}),
            output_schema: json!({"type": "object"}),
            timeout_ms: 1_000,
            required_permissions: Vec::new(),
        }
    }

    async fn run(&self, _input: Value, _context: &ExecutionContext) -> Result<ToolOutput> {
        Ok(ToolOutput {
            value: json!({"ok": true}),
        })
    }
}

pub fn build_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    for name in [
        "filesystem.read",
        "filesystem.write",
        "cargo.check",
        "cargo.test",
        "git.diff",
    ] {
        registry.register(NoopTool {
            name: name.to_string(),
        });
    }
    registry
}

pub fn ensure_deterministic_mode(mode: &str) -> Result<()> {
    if mode == DETERMINISTIC_MODE {
        return Ok(());
    }

    bail!(
        "unsupported model mode '{}': only {} is available in V2 runtime",
        mode,
        DETERMINISTIC_MODE
    )
}

pub fn validate_goal(goal: &str) -> Result<()> {
    if goal.trim().is_empty() {
        bail!("goal must not be empty")
    }

    if goal.len() > MAX_GOAL_LEN {
        bail!("goal is too long (max {MAX_GOAL_LEN} chars)")
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_model_mode() {
        assert!(ensure_deterministic_mode("openai").is_err());
    }

    #[test]
    fn registry_contains_expected_tools() {
        let registry = build_tool_registry();
        for tool in [
            "filesystem.read",
            "filesystem.write",
            "cargo.check",
            "cargo.test",
            "git.diff",
        ] {
            assert!(registry.contains(tool));
        }
    }

    #[test]
    fn validate_goal_rejects_empty_and_too_long_inputs() {
        assert!(validate_goal("  ").is_err());
        assert!(validate_goal(&"x".repeat(MAX_GOAL_LEN + 1)).is_err());
    }

    #[test]
    fn validate_goal_accepts_valid_input() {
        assert!(validate_goal("Fix rust compile error").is_ok());
    }
}
