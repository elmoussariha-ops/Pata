use crate::types::{FileSummary, RetrievalHit, ValidationResult};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn write_file_summaries(root: &Path, summaries: &[FileSummary]) -> Result<(), String> {
    fs::create_dir_all(root.join(".pata/memory")).map_err(|e| e.to_string())?;
    let mut text = String::new();
    let mut signatures = String::new();
    for s in summaries {
        text.push_str(&format!(
            "path={}\nlines={}\nmodule_summary={}\nsymbols={}\n---\n",
            s.path.display(),
            s.lines,
            s.module_summary,
            s.symbols.join(" | ")
        ));
        let full_path = root.join(&s.path);
        let mtime = file_mtime_sec(&full_path).unwrap_or(0);
        let sig = file_signature(&full_path).unwrap_or_else(|_| "missing".to_string());
        signatures.push_str(&format!("{}\t{}\t{}\n", s.path.display(), mtime, sig));
    }
    fs::write(root.join(".pata/memory/file_summaries.txt"), text).map_err(|e| e.to_string())?;
    fs::write(root.join(".pata/memory/file_signatures.tsv"), signatures).map_err(|e| e.to_string())
}

pub fn append_task_event(root: &Path, action: &str, detail: &str) -> Result<(), String> {
    fs::create_dir_all(root.join(".pata/memory")).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let line = format!("{{\"ts\":{ts},\"action\":\"{action}\",\"detail\":\"{detail}\"}}\n");
    let path = root.join(".pata/memory/task_memory.jsonl");
    let mut existing = fs::read_to_string(&path).unwrap_or_default();
    existing.push_str(&line);
    fs::write(path, existing).map_err(|e| e.to_string())
}

pub fn write_retrieval_snapshot(
    root: &Path,
    query: &str,
    hits: &[RetrievalHit],
) -> Result<(), String> {
    fs::create_dir_all(root.join(".pata/memory")).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let mut text = format!("query={query}\nmax_loaded={}\nts={ts}\n", hits.len());
    for (idx, h) in hits.iter().enumerate() {
        let recency_score = hits.len().saturating_sub(idx);
        text.push_str(&format!(
            "hit={} score={} recency_score={}\n",
            h.path.display(),
            h.score,
            recency_score
        ));
    }
    fs::write(root.join(".pata/memory/retrieval_latest.txt"), text).map_err(|e| e.to_string())
}

pub fn cache_validation_errors(root: &Path, report: &ValidationResult) -> Result<(), String> {
    fs::create_dir_all(root.join(".pata/memory")).map_err(|e| e.to_string())?;
    let mut lines = String::new();
    for l in &report.logs {
        if l.contains("error") || l.contains("failed") {
            lines.push_str(l);
            lines.push('\n');
        }
    }
    fs::write(root.join(".pata/memory/cargo_errors_latest.log"), lines).map_err(|e| e.to_string())
}

pub fn append_patch_history(root: &Path, patch_id: &str, risk_score: u8) -> Result<(), String> {
    fs::create_dir_all(root.join(".pata/memory")).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let line = format!("{ts}\t{patch_id}\trisk={risk_score}\n");
    let path = root.join(".pata/memory/patch_history.tsv");
    let mut cur = fs::read_to_string(&path).unwrap_or_default();
    cur.push_str(&line);
    fs::write(path, cur).map_err(|e| e.to_string())
}

pub fn compress_task_memory(root: &Path) -> Result<(), String> {
    let path = root.join(".pata/memory/task_memory.jsonl");
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = raw.lines().collect();
    if lines.len() <= 200 {
        return Ok(());
    }
    let keep = &lines[lines.len() - 120..];
    let mut out = format!(
        "{{\"compressed\":true,\"dropped\":{}}}\n",
        lines.len() - 120
    );
    out.push_str(&keep.join("\n"));
    out.push('\n');
    fs::write(path, out).map_err(|e| e.to_string())
}

pub fn file_signature(path: &Path) -> Result<String, String> {
    let content = fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

pub fn file_mtime_sec(path: &Path) -> Option<u64> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_change_when_file_changes() {
        let dir = std::env::temp_dir().join("pata-memory-test-signatures");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.rs");
        fs::write(&file, "fn a(){}\n").unwrap();
        let s1 = file_signature(&file).unwrap();
        fs::write(&file, "fn a(){println!(\"x\");}\n").unwrap();
        let s2 = file_signature(&file).unwrap();
        assert_ne!(s1, s2);
    }
}
