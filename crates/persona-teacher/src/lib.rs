use agent_traits::{AgentError, Persona};

/// Public profile describing what the teacher persona optimizes for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeacherQualityProfile {
    pub objectives: Vec<&'static str>,
    pub expected_behaviors: Vec<&'static str>,
    pub quality_criteria: Vec<&'static str>,
    pub guardrails: Vec<&'static str>,
}

#[derive(Debug, Default, Clone)]
pub struct TeacherPersona;

impl TeacherPersona {
    pub fn quality_profile(&self) -> TeacherQualityProfile {
        TeacherQualityProfile {
            objectives: vec![
                "Explain concepts clearly and accurately",
                "Adapt explanation depth to learner level",
                "Provide structured pedagogy with feedback loops",
            ],
            expected_behaviors: vec![
                "State a clear learning objective",
                "Adjust complexity with explicit level adaptation",
                "Deliver scaffolded explanation and worked guidance",
                "Include coherence and misconception checks before final answer",
            ],
            quality_criteria: vec![
                "Conceptual clarity",
                "Difficulty adaptation",
                "Pedagogical structure",
                "Internal consistency and factual caution",
            ],
            guardrails: vec![
                "Do not invent external facts when uncertain",
                "Do not skip learner-level adaptation",
                "Do not provide final answer without understanding check",
            ],
        }
    }

    fn expected_sections(&self) -> [&'static str; 6] {
        [
            "LEARNING_OBJECTIVE:",
            "LEVEL_ADAPTATION:",
            "EXPLANATION:",
            "GUIDED_PRACTICE:",
            "UNDERSTANDING_CHECK:",
            "FINAL_ANSWER:",
        ]
    }
}

impl Persona for TeacherPersona {
    fn name(&self) -> &'static str {
        "teacher"
    }

    fn system_prompt(&self) -> String {
        let profile = self.quality_profile();

        format!(
            "You are a pedagogical AI teacher.
\
             Objectives: {}.
\
             Behaviors: {}.
\
             Quality criteria: {}.
\
             Guardrails: {}.
\
             Output contract (in order): LEARNING_OBJECTIVE, LEVEL_ADAPTATION, EXPLANATION, GUIDED_PRACTICE, UNDERSTANDING_CHECK, FINAL_ANSWER.
\
             Keep explanations explicit, coherent, and adapted to learner level.",
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
                "teacher persona response cannot be empty".to_string(),
            ));
        }

        let missing = self
            .expected_sections()
            .into_iter()
            .filter(|section| !draft.contains(section))
            .collect::<Vec<_>>();

        if !missing.is_empty() {
            return Err(AgentError::Validation(format!(
                "teacher persona response is missing required sections: {}",
                missing.join(", ")
            )));
        }

        if draft.contains("LEVEL_ADAPTATION:")
            && !draft.to_ascii_lowercase().contains("beginner")
            && !draft.to_ascii_lowercase().contains("intermediate")
            && !draft.to_ascii_lowercase().contains("advanced")
        {
            return Err(AgentError::Validation(
                "teacher persona must state an explicit learner level (beginner/intermediate/advanced)"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_exposes_teacher_contract() {
        let persona = TeacherPersona;
        let prompt = persona.system_prompt();

        assert!(prompt.contains("pedagogical AI teacher"));
        assert!(prompt.contains("Objectives:"));
        assert!(prompt.contains("LEVEL_ADAPTATION"));
        assert!(prompt.contains("UNDERSTANDING_CHECK"));
    }

    #[test]
    fn validate_accepts_structured_teacher_output() {
        let persona = TeacherPersona;
        let draft = "LEARNING_OBJECTIVE: Understand ownership basics\n\
LEVEL_ADAPTATION: beginner audience with simple language\n\
EXPLANATION: Ownership means one owner at a time with clear transfer rules.\n\
GUIDED_PRACTICE: Try rewriting one function to avoid moving values unexpectedly.\n\
UNDERSTANDING_CHECK: Can you explain why moving invalidates prior binding?\n\
FINAL_ANSWER: Start with ownership then borrowing; practice with one short example.";

        assert!(persona.validate(draft).is_ok());
    }

    #[test]
    fn validate_rejects_missing_sections() {
        let persona = TeacherPersona;
        let draft = "EXPLANATION: ownership\nFINAL_ANSWER: done";

        let err = persona
            .validate(draft)
            .expect_err("missing sections should be rejected");
        assert!(format!("{err}").contains("missing required sections"));
    }

    #[test]
    fn validate_requires_explicit_level() {
        let persona = TeacherPersona;
        let draft = "LEARNING_OBJECTIVE: Understand ownership\n\
LEVEL_ADAPTATION: adjust tone\n\
EXPLANATION: Ownership means one owner at a time.\n\
GUIDED_PRACTICE: Explain with one example.\n\
UNDERSTANDING_CHECK: What happens after move?\n\
FINAL_ANSWER: ownership basics summary.";

        let err = persona
            .validate(draft)
            .expect_err("missing learner level should be rejected");
        assert!(format!("{err}").contains("explicit learner level"));
    }
}
