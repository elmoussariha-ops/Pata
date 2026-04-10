use agent_traits::{AgentError, Persona};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmbQualityProfile {
    pub objectives: Vec<&'static str>,
    pub expected_behaviors: Vec<&'static str>,
    pub quality_criteria: Vec<&'static str>,
    pub guardrails: Vec<&'static str>,
}

#[derive(Debug, Default, Clone)]
pub struct SmbPersona;

impl SmbPersona {
    pub fn quality_profile(&self) -> SmbQualityProfile {
        SmbQualityProfile {
            objectives: vec![
                "Improve small-business operational clarity",
                "Support practical day-to-day decisions",
                "Structure business follow-up actions",
            ],
            expected_behaviors: vec![
                "Summarize business context and constraints",
                "Propose a prioritized operational action backlog",
                "Provide simple decision support with assumptions",
                "Define follow-up metrics and next checkpoints",
            ],
            quality_criteria: vec![
                "Operational usefulness",
                "Prioritization clarity",
                "Decision pragmatism",
                "Execution follow-through",
            ],
            guardrails: vec![
                "Do not provide legal/financial certainty claims",
                "Do not ignore business constraints (budget/time/capacity)",
                "Do not skip assumptions and follow-up metrics",
            ],
        }
    }

    fn expected_sections(&self) -> [&'static str; 6] {
        [
            "BUSINESS_CONTEXT:",
            "OPERATIONAL_OBJECTIVE:",
            "ACTION_BACKLOG:",
            "DECISION_SUPPORT:",
            "FOLLOW_UP_METRICS:",
            "FINAL_ANSWER:",
        ]
    }
}

impl Persona for SmbPersona {
    fn name(&self) -> &'static str {
        "smb"
    }

    fn system_prompt(&self) -> String {
        let profile = self.quality_profile();

        format!(
            "You are a practical SMB operations copilot.
\
             Objectives: {}.
\
             Behaviors: {}.
\
             Quality criteria: {}.
\
             Guardrails: {}.
\
             Output contract (in order): BUSINESS_CONTEXT, OPERATIONAL_OBJECTIVE, ACTION_BACKLOG, DECISION_SUPPORT, FOLLOW_UP_METRICS, FINAL_ANSWER.
\
             Keep answers pragmatic, constrained, and execution-oriented.",
            profile.objectives.join(" | "),
            profile.expected_behaviors.join(" | "),
            profile.quality_criteria.join(" | "),
            profile.guardrails.join(" | "),
        )
    }

    fn allowed_tools(&self) -> Vec<String> {
        vec!["filesystem.read".to_string(), "git.diff".to_string()]
    }

    fn validate(&self, draft: &str) -> std::result::Result<(), AgentError> {
        if draft.trim().is_empty() {
            return Err(AgentError::Validation(
                "smb persona response cannot be empty".to_string(),
            ));
        }

        let missing = self
            .expected_sections()
            .into_iter()
            .filter(|section| !draft.contains(section))
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            return Err(AgentError::Validation(format!(
                "smb persona response is missing required sections: {}",
                missing.join(", ")
            )));
        }

        if !draft.to_ascii_lowercase().contains("assumption") {
            return Err(AgentError::Validation(
                "smb persona must include explicit assumptions for decision support".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_exposes_smb_contract() {
        let persona = SmbPersona;
        let prompt = persona.system_prompt();

        assert!(prompt.contains("SMB operations copilot"));
        assert!(prompt.contains("ACTION_BACKLOG"));
        assert!(prompt.contains("FOLLOW_UP_METRICS"));
    }

    #[test]
    fn validate_accepts_structured_smb_output() {
        let persona = SmbPersona;
        let draft = "BUSINESS_CONTEXT: Small local shop with limited staff and marketing budget.\n\
OPERATIONAL_OBJECTIVE: Increase weekly repeat customers in 30 days.\n\
ACTION_BACKLOG: 1) Daily follow-up messages, 2) Weekly promo bundle, 3) Track top-selling items.\n\
DECISION_SUPPORT: Assumption: existing customers respond better to loyalty nudges than broad ads.\n\
FOLLOW_UP_METRICS: Repeat visits/week, promo conversion rate, average basket size.\n\
FINAL_ANSWER: Start with low-cost retention actions, review metrics weekly, then adjust.";

        assert!(persona.validate(draft).is_ok());
    }
}
