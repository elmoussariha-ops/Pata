use crate::types::PatchProposal;
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn save_proposal(
    objective: &str,
    diff: String,
    touched_files: Vec<PathBuf>,
) -> Result<PatchProposal> {
    let id = format!("proposal-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let proposal = PatchProposal {
        id: id.clone(),
        created_at: Utc::now(),
        objective: objective.to_string(),
        unified_diff: diff,
        touched_files,
        risk_level: "medium".to_string(),
    };

    let base = PathBuf::from(".pata/proposals");
    fs::create_dir_all(&base)?;
    fs::write(base.join(format!("{}.diff", id)), &proposal.unified_diff)?;
    fs::write(
        base.join(format!("{}.json", id)),
        serde_json::to_string_pretty(&proposal)?,
    )?;
    Ok(proposal)
}

pub fn guarded_apply(root: &Path, proposal: &PatchProposal, protected: &[PathBuf]) -> Result<()> {
    for touched in &proposal.touched_files {
        if protected.iter().any(|p| touched.starts_with(p)) {
            return Err(anyhow!(
                "patch touches protected path {} and requires manual validation",
                touched.display()
            ));
        }
    }

    let file_path = root
        .join(".pata/proposals")
        .join(format!("{}.diff", proposal.id));
    if !file_path.exists() {
        return Err(anyhow!(
            "proposal diff file missing: {}",
            file_path.display()
        ));
    }
    Ok(())
}
