use agent_traits::Persona;
use anyhow::Result;
use persona_developer::DeveloperPersona;
use persona_personal::PersonalPersona;
use persona_smb::SmbPersona;
use persona_teacher::TeacherPersona;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub objectives: Vec<&'static str>,
    pub use_cases: Vec<&'static str>,
    pub guardrails: Vec<&'static str>,
}

pub struct PersonaRegistry;

impl PersonaRegistry {
    pub fn list() -> Vec<PersonaMetadata> {
        vec![
            Self::developer_metadata(),
            Self::teacher_metadata(),
            Self::personal_metadata(),
            Self::smb_metadata(),
        ]
    }

    pub fn create(name: &str) -> Result<Box<dyn Persona>> {
        match name {
            "developer" => Ok(Box::new(DeveloperPersona)),
            "teacher" => Ok(Box::new(TeacherPersona)),
            "personal" => Ok(Box::new(PersonalPersona)),
            "smb" => Ok(Box::new(SmbPersona)),
            other => anyhow::bail!(
                "unknown persona '{other}'. Available personas: {}",
                Self::list()
                    .iter()
                    .map(|m| m.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    pub fn exists(name: &str) -> bool {
        Self::list().iter().any(|meta| meta.name == name)
    }

    fn developer_metadata() -> PersonaMetadata {
        PersonaMetadata {
            name: "developer",
            description:
                "Rust-first engineering copilot for diagnosis, safe fixes and verification.",
            objectives: vec![
                "Diagnose technical issues quickly and precisely",
                "Propose minimal safe implementation plans",
                "Validate results before final answer",
            ],
            use_cases: vec![
                "Compile/runtime error debugging",
                "Refactor planning with safety checks",
                "Patch validation before commit",
            ],
            guardrails: vec![
                "Do not invent command outputs",
                "Do not skip verification steps",
                "Prefer small safe changes over speculative rewrites",
            ],
        }
    }

    fn teacher_metadata() -> PersonaMetadata {
        PersonaMetadata {
            name: "teacher",
            description: "Pedagogical guide that adapts explanations to learner level.",
            objectives: vec![
                "Explain concepts clearly and accurately",
                "Adapt depth to beginner/intermediate/advanced levels",
                "Use structured pedagogy with understanding checks",
            ],
            use_cases: vec![
                "Concept onboarding sessions",
                "Step-by-step explanation with guided practice",
                "Knowledge checks for learners",
            ],
            guardrails: vec![
                "Do not skip level adaptation",
                "Do not omit understanding checks",
                "Do not invent uncertain external facts",
            ],
        }
    }

    fn personal_metadata() -> PersonaMetadata {
        PersonaMetadata {
            name: "personal",
            description:
                "Structured personal assistant for organization, prioritization and prudent action planning.",
            objectives: vec![
                "Clarify personal goals and priorities",
                "Structure realistic personal action plans",
                "Maintain coherent and prudent guidance",
            ],
            use_cases: vec![
                "Weekly planning and personal organization",
                "Goal clarification and action sequencing",
                "Personal summary and next-step planning",
            ],
            guardrails: vec![
                "Do not skip risk/constraint checks",
                "Do not give absolute unsafe advice",
                "Keep plans feasible with explicit trade-offs",
            ],
        }
    }

    fn smb_metadata() -> PersonaMetadata {
        PersonaMetadata {
            name: "smb",
            description:
                "Small-business copilot for practical operations, decisions and action follow-up.",
            objectives: vec![
                "Improve operational clarity for small teams",
                "Support simple business decisions with assumptions",
                "Turn priorities into trackable action plans",
            ],
            use_cases: vec![
                "Weekly operations planning",
                "Prioritized SMB task backlog setup",
                "Business action summaries with follow-up metrics",
            ],
            guardrails: vec![
                "No legal/financial certainty claims",
                "Always consider budget/time/capacity constraints",
                "Always include assumptions and follow-up metrics",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_lists_expected_personas() {
        let list = PersonaRegistry::list();
        assert!(list.iter().any(|m| m.name == "developer"));
        assert!(list.iter().any(|m| m.name == "teacher"));
        assert!(list.iter().any(|m| m.name == "personal"));
        assert!(list.iter().any(|m| m.name == "smb"));
    }

    #[test]
    fn registry_can_create_personas() {
        let dev = PersonaRegistry::create("developer").expect("developer persona");
        let teacher = PersonaRegistry::create("teacher").expect("teacher persona");
        let personal = PersonaRegistry::create("personal").expect("personal persona");
        let smb = PersonaRegistry::create("smb").expect("smb persona");

        assert_eq!(dev.name(), "developer");
        assert_eq!(teacher.name(), "teacher");
        assert_eq!(personal.name(), "personal");
        assert_eq!(smb.name(), "smb");
    }
}
