use agent_traits::{AgentError, Persona};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalQualityProfile {
    pub objectives: Vec<&'static str>,
    pub expected_behaviors: Vec<&'static str>,
    pub quality_criteria: Vec<&'static str>,
    pub guardrails: Vec<&'static str>,
}

#[derive(Debug, Default, Clone)]
pub struct PersonalPersona;

impl PersonalPersona {
    pub fn quality_profile(&self) -> PersonalQualityProfile {
        PersonalQualityProfile {
            objectives: vec![
                "Clarify personal goals and priorities",
                "Structure realistic action plans",
                "Support consistent and safe decision-making",
            ],
            expected_behaviors: vec![
                "Summarize user context before proposing actions",
                "Break plans into concrete and feasible steps",
                "Surface risks, constraints and trade-offs",
                "End with one immediately actionable next step",
            ],
            quality_criteria: vec![
                "Practical relevance",
                "Clarity of priorities",
                "Feasibility of action plan",
                "Prudence and coherence",
            ],
            guardrails: vec![
                "Do not provide unsafe or absolute advice",
                "Do not ignore constraints provided by user",
                "Do not skip risk/constraint check before final answer",
            ],
        }
    }

    fn expected_sections(&self) -> [&'static str; 6] {
        [
            "CONTEXT_SUMMARY:",
            "PRIMARY_OBJECTIVE:",
            "ACTION_STRUCTURE:",
            "RISK_CHECK:",
            "NEXT_STEP:",
            "FINAL_ANSWER:",
        ]
    }
}

impl Persona for PersonalPersona {
    fn name(&self) -> &'static str {
        "personal"
    }

    fn system_prompt(&self) -> String {
        let profile = self.quality_profile();

        format!(
            "You are a structured personal productivity assistant.
\
             Objectives: {}.
\
             Behaviors: {}.
\
             Quality criteria: {}.
\
             Guardrails: {}.
\
             Output contract (in order): CONTEXT_SUMMARY, PRIMARY_OBJECTIVE, ACTION_STRUCTURE, RISK_CHECK, NEXT_STEP, FINAL_ANSWER.
\
             Be practical, coherent, and prudent.",
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
                "personal persona response cannot be empty".to_string(),
            ));
        }

        let missing = self
            .expected_sections()
            .into_iter()
            .filter(|section| !draft.contains(section))
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            return Err(AgentError::Validation(format!(
                "personal persona response is missing required sections: {}",
                missing.join(", ")
            )));
        }

        if !draft.to_ascii_lowercase().contains("constraint")
            && !draft.to_ascii_lowercase().contains("risk")
        {
            return Err(AgentError::Validation(
                "personal persona must explicitly mention a risk or constraint check".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_exposes_personal_contract() {
        let persona = PersonalPersona;
        let prompt = persona.system_prompt();

        assert!(prompt.contains("personal productivity assistant"));
        assert!(prompt.contains("CONTEXT_SUMMARY"));
        assert!(prompt.contains("NEXT_STEP"));
    }

    #[test]
    fn validate_accepts_structured_personal_output() {
        let persona = PersonalPersona;
        let draft =
            "CONTEXT_SUMMARY: You need better weekly organization with limited evening time.\n\
PRIMARY_OBJECTIVE: Build a realistic weekly routine for priorities.\n\
ACTION_STRUCTURE: Block planning Sunday, daily top-3 tasks, evening review in 10 minutes.\n\
RISK_CHECK: Main constraint is limited energy after work; reduce task load accordingly.\n\
NEXT_STEP: Tonight, write tomorrow's top-3 priorities in one note.\n\
FINAL_ANSWER: Start with a light routine, review weekly, and adapt by energy constraints.";

        assert!(persona.validate(draft).is_ok());
    }

    #[test]
    fn validate_rejects_missing_sections() {
        let persona = PersonalPersona;
        let draft = "CONTEXT_SUMMARY: busy\nFINAL_ANSWER: plan";

        let err = persona
            .validate(draft)
            .expect_err("missing sections should be rejected");
        assert!(format!("{err}").contains("missing required sections"));
    }
}
