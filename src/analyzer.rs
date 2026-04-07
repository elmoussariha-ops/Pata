use crate::types::{FileInsight, ProjectAnalysis};
use anyhow::{Context, Result};
use regex::Regex;
use std::{fs, path::Path};
use walkdir::WalkDir;

pub fn analyze_project(root: &Path) -> Result<ProjectAnalysis> {
    let mut insights = Vec::new();
    let mut warnings = Vec::new();
    let mut total_lines = 0usize;
    let fn_regex = Regex::new(r"\bfn\b").unwrap();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.components().any(|c| c.as_os_str() == "target") {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read Rust source {}", path.display()))?;
        let lines = text.lines().count();
        total_lines += lines;
        let todo_count = text.matches("TODO").count() + text.matches("FIXME").count();
        let has_unsafe = text.contains("unsafe {") || text.contains("unsafe fn");
        let complexity_score = fn_regex.find_iter(&text).count() + text.matches("match ").count();

        if todo_count > 5 {
            warnings.push(format!("{} has a high TODO/FIXME count", path.display()));
        }
        if has_unsafe {
            warnings.push(format!("{} uses unsafe blocks", path.display()));
        }

        insights.push(FileInsight {
            path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
            lines,
            has_unsafe,
            todo_count,
            complexity_score,
        });
    }

    insights.sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));

    Ok(ProjectAnalysis {
        root: root.to_path_buf(),
        rust_files: insights.len(),
        total_lines,
        insights,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_unsafe_and_todos() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = std::fs::File::create(dir.path().join("mod.rs")).unwrap();
        writeln!(file, "unsafe fn bad() {{}} // TODO").unwrap();
        let analysis = analyze_project(dir.path()).unwrap();
        assert_eq!(analysis.rust_files, 1);
        assert!(analysis.warnings.iter().any(|w| w.contains("unsafe")));
    }
}
