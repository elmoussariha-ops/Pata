use crate::model::{ask_ollama, ensure_model_available, settings_from_env};
use crate::types::RetrievalHit;

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
    let prompt = format!(
        "You are a Rust code assistant. Objective: {objective}. Return only unified diff patch. Context:\n{context}"
    );
    let response = ask_ollama(&settings, &prompt)?;
    Ok((response, settings.model))
}
