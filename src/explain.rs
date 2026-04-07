use crate::types::PatchReview;

pub fn build_patch_explanation(objective: &str, review: &PatchReview) -> String {
    format!(
        "objective={}\nsummary={}\nfiles={}\nzones={}\nrisk={}\nreasons={}\nimpact={}\nvalidations={}\n",
        objective,
        review.summary,
        review.files.join(", "),
        review.risk.sensitive_zones.join(" | "),
        review.risk.score,
        review.risk.reasons.join(" | "),
        if review.risk.allowed { "low-to-medium" } else { "high / manual review" },
        "cargo check; cargo clippy -D warnings; cargo test"
    )
}
