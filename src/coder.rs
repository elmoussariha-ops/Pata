use crate::model::{ask_ollama, ensure_model_available, settings_from_env};
use crate::types::RetrievalHit;

pub const PERSONA_CONTRACT_VERSION: &str = "persona.v2";
const PERSONA_CONTRACT: [&str; 6] = [
    "Role: senior Rust engineer focused on production reliability.",
    "Output contract: return unified diff only, no markdown fences.",
    "Safety contract: avoid destructive shell commands and secrets.",
    "Testing contract: include changes that keep check/clippy/test green.",
    "Scope contract: limit edits to files needed for the objective.",
    "Rollback contract: keep patch reversible and deterministic.",
];

#[derive(Debug, Clone)]
pub struct PersonaContractSnapshot {
    pub version: &'static str,
    pub clauses: Vec<&'static str>,
}

pub fn persona_contract_snapshot() -> PersonaContractSnapshot {
    PersonaContractSnapshot {
        version: PERSONA_CONTRACT_VERSION,
        clauses: PERSONA_CONTRACT.to_vec(),
    }
}

pub fn generate_patch(objective: &str, hits: &[RetrievalHit]) -> Result<(String, String), String> {
    if let Ok(fake) = std::env::var("PATA_FAKE_PATCH") {
        return Ok((fake, "mock-model".to_string()));
    }

    let settings = settings_from_env();
    ensure_model_available(&settings)?;
    let context = hits
        .iter()
        .map(|h| format!("FILE:{}\n{}", h.path.display(), h.excerpt))
        .collect::<Vec<_>>()
        .join("\n\n");
    let contract = PERSONA_CONTRACT.join("\n");
    let prompt = format!(
        "You are a Rust code assistant.\nPersona contract version: {PERSONA_CONTRACT_VERSION}\n{contract}\nObjective: {objective}.\nReturn only unified diff patch.\nContext:\n{context}"
    );
    let response = ask_ollama(&settings, &prompt)?;
    Ok((response, settings.model))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_contract_is_versioned_and_strict() {
        let c = persona_contract_snapshot();
        assert!(c.version.starts_with("persona."));
        assert!(c.clauses.len() >= 5);
        assert!(c
            .clauses
            .iter()
            .any(|line| line.contains("unified diff only")));
    }
}
