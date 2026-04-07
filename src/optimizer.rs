use crate::{
    config::AppConfig,
    diagnostics::run_diagnostics,
    history::{HistoryEntry, HistoryStore},
    model::suggest_patch,
    patcher::{guarded_apply, save_proposal},
    rollback::{create_checkpoint, rollback_to},
    types::{OptimizationRecord, PerfSnapshot},
};
use anyhow::Result;
use chrono::Utc;
use std::path::Path;
use tokio::time::{self, Duration};

pub async fn run_optimization_cycle(
    root: &Path,
    cfg: &AppConfig,
    history: &HistoryStore,
) -> Result<OptimizationRecord> {
    let diagnostics = run_diagnostics(root).await?;
    let snapshot = PerfSnapshot {
        timestamp: Utc::now(),
        avg_check_ms: diagnostics.check.duration_ms,
        avg_test_ms: diagnostics
            .tests
            .as_ref()
            .map(|t| t.duration_ms)
            .unwrap_or_default(),
        memory_note: "Memory budget target: <= 6GB RSS on MacBook Air M4".to_string(),
    };

    let weak_points = diagnostics.findings.clone();
    let prompt = format!(
        "You are optimizing a Rust code assistant. Snapshot: {:?}. Give a short patch suggestion.",
        snapshot
    );
    let suggestion = suggest_patch(cfg, &prompt)
        .await
        .unwrap_or_else(|e| format!("model unavailable: {e}"));

    let mut patch_path = None;
    let mut tests_passed = diagnostics.findings.iter().all(|f| !f.contains("failed"));
    let mut rolled_back = false;

    if cfg.allow_auto_apply {
        let checkpoint = create_checkpoint(root).await?;
        let proposal = save_proposal("scheduled optimization", suggestion.clone(), vec![])?;
        patch_path = Some(
            root.join(".pata/proposals")
                .join(format!("{}.diff", proposal.id)),
        );
        if guarded_apply(root, &proposal, &cfg.protected_paths).is_err() {
            rollback_to(root, &checkpoint).await?;
            rolled_back = true;
            tests_passed = false;
        }
    }

    let record = OptimizationRecord {
        timestamp: Utc::now(),
        weak_points,
        suggestion,
        patch_path,
        tests_passed,
        rolled_back,
    };
    history.append(&HistoryEntry::Optimization(record.clone()))?;
    Ok(record)
}

pub async fn spawn_optimizer(root: &Path, cfg: AppConfig, history: HistoryStore) {
    let root = root.to_path_buf();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(cfg.optimization_interval_seconds));
        loop {
            interval.tick().await;
            let _ = run_optimization_cycle(&root, &cfg, &history).await;
        }
    });
}
