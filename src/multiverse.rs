use crate::coder;
use crate::long_memory;
use crate::patcher;
use crate::planner;
use crate::retriever;
use crate::reviewer;
use crate::scanner;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BranchOutcome {
    pub strategy: String,
    pub patch_id: String,
    pub risk: u8,
    pub apply_check_ok: bool,
    pub check_ok: bool,
    pub test_ok: bool,
    pub diff_lines: usize,
    pub score: i32,
    pub note: String,
}

fn strategies() -> Vec<(&'static str, &'static str)> {
    vec![
        ("minimal", "minimal patch surface"),
        ("readability", "clear naming and readability first"),
        ("robustness", "defensive checks and edge cases"),
        ("reviewer-prudent", "risk-aware conservative edits"),
    ]
}

fn diff_lines(diff: &str) -> usize {
    diff.lines()
        .filter(|l| l.starts_with('+') || l.starts_with('-'))
        .count()
}

fn run_in_temp_worktree(root: &Path, patch_id: &str, run_tests: bool) -> (bool, bool, String) {
    let temp = std::env::temp_dir().join(format!("pata-multiverse-{patch_id}"));
    let _ = fs::remove_dir_all(&temp);
    let add = Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(&temp)
        .arg("HEAD")
        .current_dir(root)
        .output();
    let Ok(add_out) = add else {
        return (false, false, "worktree add failed".to_string());
    };
    if !add_out.status.success() {
        return (
            false,
            false,
            String::from_utf8_lossy(&add_out.stderr).trim().to_string(),
        );
    }

    let patch_file = root.join(format!(".pata/patches/{patch_id}.diff"));
    let ap = Command::new("git")
        .arg("apply")
        .arg(&patch_file)
        .current_dir(&temp)
        .output();
    let mut check_ok = false;
    let mut test_ok = false;
    let mut note = String::new();
    if let Ok(ap_out) = ap {
        if ap_out.status.success() {
            let ck = Command::new("cargo")
                .arg("check")
                .current_dir(&temp)
                .output();
            if let Ok(ck_out) = ck {
                check_ok = ck_out.status.success();
                if !check_ok {
                    note = String::from_utf8_lossy(&ck_out.stderr).trim().to_string();
                }
            }
            if run_tests {
                let ts = Command::new("cargo")
                    .arg("test")
                    .arg("-q")
                    .current_dir(&temp)
                    .output();
                if let Ok(ts_out) = ts {
                    test_ok = ts_out.status.success();
                    if !test_ok && note.is_empty() {
                        note = String::from_utf8_lossy(&ts_out.stderr).trim().to_string();
                    }
                }
            }
        } else {
            note = String::from_utf8_lossy(&ap_out.stderr).trim().to_string();
        }
    }

    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&temp)
        .current_dir(root)
        .output();
    let _ = fs::remove_dir_all(&temp);
    (check_ok, test_ok, note)
}

pub fn run_multiverse(
    root: &Path,
    objective: &str,
    low_power: bool,
    branches: usize,
) -> Result<Vec<BranchOutcome>, String> {
    let idx = scanner::scan_repo(root)?;
    let mut out = Vec::new();
    let n = branches.clamp(2, 4);
    for (i, (name, style)) in strategies().into_iter().take(n).enumerate() {
        let objective_variant = format!("{objective} [{name}: {style}]");
        let hits = retriever::top_n(
            &idx,
            &objective_variant,
            if low_power { 3 } else { 5 },
            &idx.workspace_root,
        );
        let _plan = planner::build_plan(&objective_variant, &hits);
        let (diff, _model_used) = coder::generate_patch(&objective_variant, &hits)?;
        let proposal = patcher::create(
            &objective_variant,
            diff.clone(),
            hits.iter().map(|h| h.path.clone()).collect(),
        )?;
        let mut review = reviewer::review(
            &proposal,
            &[
                "src/main.rs",
                "src/model.rs",
                "src/rollback.rs",
                "AGENTS.md",
            ],
        );
        long_memory::adjust_review_with_memory(root, &mut review);
        patcher::save_review(&proposal.id, &review)?;

        let ap_check = Command::new("git")
            .arg("apply")
            .arg("--check")
            .arg(root.join(format!(".pata/patches/{}.diff", proposal.id)))
            .current_dir(root)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let run_tests = !low_power && i == 0;
        let (check_ok, test_ok, note) = if ap_check {
            run_in_temp_worktree(root, &proposal.id, run_tests)
        } else {
            (false, false, "git apply --check failed".to_string())
        };

        let mut score = 100i32;
        score -= review.risk.score as i32;
        score -= (diff_lines(&proposal.diff) as i32 / 12).min(18);
        if !ap_check {
            score -= 40;
        }
        if check_ok {
            score += 20;
        } else {
            score -= 25;
        }
        if run_tests && test_ok {
            score += 15;
        }

        out.push(BranchOutcome {
            strategy: name.to_string(),
            patch_id: proposal.id,
            risk: review.risk.score,
            apply_check_ok: ap_check,
            check_ok,
            test_ok,
            diff_lines: diff_lines(&proposal.diff),
            score,
            note,
        });
    }
    out.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_line_count_works() {
        let d = "+a\n+b\n-c\n context\n";
        assert_eq!(diff_lines(d), 3);
    }
}
