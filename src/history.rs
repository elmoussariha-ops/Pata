use crate::types::{ActionPlan, DiagnosticReport, OptimizationRecord, ProjectAnalysis};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HistoryEntry {
    Analysis(ProjectAnalysis),
    Diagnostics(DiagnosticReport),
    Plan(ActionPlan),
    Optimization(OptimizationRecord),
    Note { timestamp: String, message: String },
}

pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn new() -> Self {
        Self {
            path: PathBuf::from(".pata/history.jsonl"),
        }
    }

    pub fn append(&self, entry: &HistoryEntry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = serde_json::to_string(entry)?;
        let mut current = if self.path.exists() {
            fs::read_to_string(&self.path)?
        } else {
            String::new()
        };
        current.push_str(&payload);
        current.push('\n');
        fs::write(&self.path, current)?;
        Ok(())
    }

    pub fn note(&self, message: impl Into<String>) -> Result<()> {
        self.append(&HistoryEntry::Note {
            timestamp: Utc::now().to_rfc3339(),
            message: message.into(),
        })
    }

    pub fn tail(&self, n: usize) -> Result<Vec<HistoryEntry>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&self.path)?;
        let entries: Vec<HistoryEntry> = content
            .lines()
            .filter_map(|l| serde_json::from_str::<HistoryEntry>(l).ok())
            .collect();
        let len = entries.len();
        Ok(entries.into_iter().skip(len.saturating_sub(n)).collect())
    }
}
