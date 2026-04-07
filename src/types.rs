use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CargoTargetStats {
    pub bins: usize,
    pub libs: usize,
    pub examples: usize,
    pub tests: usize,
}

#[derive(Debug, Clone)]
pub struct FileSummary {
    pub path: PathBuf,
    pub lines: usize,
    pub symbols: Vec<String>,
    pub module_summary: String,
}

#[derive(Debug, Clone)]
pub struct ProjectIndex {
    pub workspace_root: PathBuf,
    pub packages: Vec<String>,
    pub manifests: Vec<PathBuf>,
    pub target_stats: CargoTargetStats,
    pub file_summaries: Vec<FileSummary>,
}

#[derive(Debug, Clone)]
pub struct RetrievalHit {
    pub path: PathBuf,
    pub score: usize,
    pub excerpt: String,
}

#[derive(Debug, Clone)]
pub struct PatchProposal {
    pub id: String,
    pub objective: String,
    pub diff: String,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RiskReport {
    pub score: u8,
    pub reasons: Vec<String>,
    pub sensitive_zones: Vec<String>,
    pub critical_files: Vec<String>,
    pub allowed: bool,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub struct PatchReview {
    pub summary: String,
    pub files: Vec<String>,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub hunk_count: usize,
    pub risk: RiskReport,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub check_ok: bool,
    pub clippy_ok: bool,
    pub test_ok: bool,
    pub logs: Vec<String>,
}

impl ValidationResult {
    pub fn ok(&self) -> bool {
        self.check_ok && self.clippy_ok && self.test_ok
    }
}

#[derive(Debug, Clone)]
pub struct OllamaSettings {
    pub endpoint: String,
    pub model: String,
    pub timeout_sec: u64,
    pub retries: u32,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct OllamaStatus {
    pub reachable: bool,
    pub installed_models: Vec<String>,
    pub selected_model: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct OllamaDiagnostic {
    pub state: String,
    pub message: String,
    pub hint: String,
}
