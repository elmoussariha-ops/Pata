use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInsight {
    pub path: PathBuf,
    pub lines: usize,
    pub has_unsafe: bool,
    pub todo_count: usize,
    pub complexity_score: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub root: PathBuf,
    pub rust_files: usize,
    pub total_lines: usize,
    pub insights: Vec<FileInsight>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandReport {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub check: CommandReport,
    pub tests: Option<CommandReport>,
    pub clippy: Option<CommandReport>,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub title: String,
    pub rationale: String,
    pub target_files: Vec<PathBuf>,
    pub requires_human_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub created_at: DateTime<Utc>,
    pub summary: String,
    pub actions: Vec<PlannedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub objective: String,
    pub unified_diff: String,
    pub touched_files: Vec<PathBuf>,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSnapshot {
    pub timestamp: DateTime<Utc>,
    pub avg_check_ms: u128,
    pub avg_test_ms: u128,
    pub memory_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    pub timestamp: DateTime<Utc>,
    pub weak_points: Vec<String>,
    pub suggestion: String,
    pub patch_path: Option<PathBuf>,
    pub tests_passed: bool,
    pub rolled_back: bool,
}
