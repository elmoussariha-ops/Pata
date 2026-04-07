use crate::types::RetrievalHit;

pub fn build_plan(objective: &str, hits: &[RetrievalHit]) -> Vec<String> {
    vec![
        format!("Objective: {objective}"),
        format!("Load only {} top files", hits.len()),
        "Generate minimal unified diff".to_string(),
        "Run cargo check/clippy/test".to_string(),
        "Review patch impact and store memory".to_string(),
    ]
}
