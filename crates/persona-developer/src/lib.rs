use agent_traits::{AgentError, Persona};

/// Public profile describing what the developer persona optimizes for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeveloperQualityProfile {
    pub objectives: Vec<&'static str>,
    pub expected_behaviors: Vec<&'static str>,
    pub quality_criteria: Vec<&'static str>,
    pub guardrails: Vec<&'static str>,
}

#[derive(Debug, Default, Clone)]
pub struct DeveloperPersona;

impl DeveloperPersona {
    pub fn quality_profile(&self) -> DeveloperQualityProfile {
        DeveloperQualityProfile {
            objectives: vec![
                "Diagnose developer issues quickly and precisely",
                "Propose safe Rust-first implementation plans",
                "Validate proposed fixes before final answer",
            ],
            expected_behaviors: vec![
                "Analyze the problem and explicit constraints",
                "Formulate a testable hypothesis",
                "Propose concrete actions/tests",
                "Self-verify and flag contradictions with durable rules",
            ],
            quality_criteria: vec![
                "Technical precision",
                "Actionable next steps",
                "Verification evidence",
                "Consistency with project rules and architecture",
            ],
            guardrails: vec![
                "Do not invent commands/results that were not executed",
                "Do not skip verification step",
                "Prefer minimal safe changes over speculative rewrites",
            ],
        }
    }

    fn expected_sections(&self) -> [&'static str; 6] {
        [
            "ANALYSIS:",
            "HYPOTHESIS:",
            "ACTION_PLAN:",
            "VALIDATION:",
            "DURABLE_RULES_CHECK:",
            "FINAL_ANSWER:",
        ]
    }
}

impl Persona for DeveloperPersona {
    fn name(&self) -> &'static str {
        "developer"
    }

    fn system_prompt(&self) -> String {
        let profile = self.quality_profile();

        format!(
            "You are a senior Rust engineering copilot.\n\
             Objectives: {}.\n\
             Behaviors: {}.\n\
             Quality criteria: {}.\n\
             Guardrails: {}.\n\
             Output contract (in order): ANALYSIS, HYPOTHESIS, ACTION_PLAN, VALIDATION, DURABLE_RULES_CHECK, FINAL_ANSWER.\n\
             Be explicit, deterministic, and avoid unverifiable claims.",
            profile.objectives.join(" | "),
            profile.expected_behaviors.join(" | "),
            profile.quality_criteria.join(" | "),
            profile.guardrails.join(" | "),
        )
    }

    fn allowed_tools(&self) -> Vec<String> {
        vec![
            "filesystem.read".to_string(),
            "filesystem.write".to_string(),
            "cargo.check".to_string(),
            "cargo.test".to_string(),
            "git.diff".to_string(),
        ]
    }

    fn validate(&self, draft: &str) -> std::result::Result<(), AgentError> {
        if draft.trim().is_empty() {
            return Err(AgentError::Validation(
                "developer persona response cannot be empty".to_string(),
            ));
        }

        let missing = self
            .expected_sections()
            .into_iter()
            .filter(|section| !draft.contains(section))
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            return Err(AgentError::Validation(format!(
                "developer persona response is missing required sections: {}",
                missing.join(", ")
            )));
        }

        if draft.contains("CONTRADICTION_DETECTED") && !draft.contains("FINAL_ANSWER:") {
            return Err(AgentError::Validation(
                "contradiction was detected but no final answer fallback was provided".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_exposes_objectives_behaviors_and_contract() {
        let persona = DeveloperPersona;
        let prompt = persona.system_prompt();

        assert!(prompt.contains("Objectives:"));
        assert!(prompt.contains("Behaviors:"));
        assert!(prompt.contains("Quality criteria:"));
        assert!(prompt.contains("Guardrails:"));
        assert!(prompt.contains("ANALYSIS"));
        assert!(prompt.contains("FINAL_ANSWER"));
    }

    #[test]
    fn validate_accepts_structured_developer_output() {
        let persona = DeveloperPersona;
        let draft = "ANALYSIS: Investigate compile failure\n\
HYPOTHESIS: Lifetime mismatch\n\
ACTION_PLAN: Narrow mutable borrow scope\n\
VALIDATION: cargo test passes\n\
DURABLE_RULES_CHECK: No rule contradiction\n\
FINAL_ANSWER: Apply scoped borrow refactor.";

        assert!(persona.validate(draft).is_ok());
    }

    #[test]
    fn validate_rejects_missing_sections() {
        let persona = DeveloperPersona;
        let draft = "ANALYSIS: issue\nFINAL_ANSWER: do this";

        let err = persona
            .validate(draft)
            .expect_err("missing sections should be rejected");
        assert!(format!("{err}").contains("missing required sections"));
    }

    #[test]
    fn allowed_tools_cover_developer_workflow() {
        let persona = DeveloperPersona;
        let tools = persona.allowed_tools();

        assert!(tools.contains(&"filesystem.read".to_string()));
        assert!(tools.contains(&"cargo.check".to_string()));
        assert!(tools.contains(&"cargo.test".to_string()));
    }
}
