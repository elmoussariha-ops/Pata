use crate::types::{PatchProposal, PatchReview, RiskReport};

pub fn review(proposal: &PatchProposal, protected_paths: &[&str]) -> PatchReview {
    let (added_lines, removed_lines, hunk_count) = diff_stats(&proposal.diff);
    let files = extract_diff_files(&proposal.diff);

    let mut score = 0u8;
    let mut reasons = Vec::new();
    let mut sensitive = Vec::new();
    let mut critical = Vec::new();

    if files.len() > 6 {
        score = score.saturating_add(25);
        reasons.push("Many files modified".to_string());
    }
    if added_lines + removed_lines > 250 || hunk_count > 12 {
        score = score.saturating_add(30);
        reasons.push("Large patch size".to_string());
    }
    if proposal.diff.contains("unsafe") {
        score = score.saturating_add(20);
        sensitive.push("unsafe usage".to_string());
    }
    if proposal.diff.contains("std::process::Command") || proposal.diff.contains("Command::new") {
        score = score.saturating_add(15);
        sensitive.push("process execution changes".to_string());
    }

    for f in &files {
        if protected_paths.iter().any(|p| f.starts_with(p)) {
            score = score.saturating_add(25);
            critical.push(f.clone());
        }
        if f.contains("rollback") || f.contains("model") || f.contains("main") {
            sensitive.push(format!("core-flow file: {f}"));
        }
    }

    if reasons.is_empty() {
        reasons.push("Patch scope is limited".to_string());
    }
    sensitive.sort();
    sensitive.dedup();
    critical.sort();
    critical.dedup();

    let allowed = score < 70 && critical.is_empty();
    let recommendation = if allowed {
        "approve_with_validation".to_string()
    } else {
        "refuse_or_manual_review".to_string()
    };

    PatchReview {
        summary: format!(
            "Patch {}: files={} hunks={} lines(+{} -{})",
            proposal.id,
            files.len(),
            hunk_count,
            added_lines,
            removed_lines
        ),
        files,
        added_lines,
        removed_lines,
        hunk_count,
        risk: RiskReport {
            score,
            reasons,
            sensitive_zones: sensitive,
            critical_files: critical,
            allowed,
            recommendation,
        },
    }
}

pub fn extract_diff_files(diff: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            files.push(path.trim().to_string());
        }
    }
    files.sort();
    files.dedup();
    files
}

pub fn diff_stats(diff: &str) -> (usize, usize, usize) {
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut hunks = 0usize;
    for line in diff.lines() {
        if line.starts_with("@@") {
            hunks += 1;
            continue;
        }
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if line.starts_with('+') {
            added += 1;
        } else if line.starts_with('-') {
            removed += 1;
        }
    }
    (added, removed, hunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PatchProposal;
    use std::path::PathBuf;

    #[test]
    fn reviewer_scores_critical_files_high() {
        let p = PatchProposal {
            id: "p1".to_string(),
            objective: "x".to_string(),
            diff: "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n+unsafe fn x(){}\n"
                .to_string(),
            files: vec![PathBuf::from("src/main.rs")],
        };
        let r = review(&p, &["src/main.rs"]);
        assert!(!r.risk.allowed);
        assert!(r.risk.score >= 45);
        assert!(!r.risk.critical_files.is_empty());
    }
}
