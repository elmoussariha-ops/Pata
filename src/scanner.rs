use crate::json::{arr, obj, parse, strv};
use crate::types::{CargoTargetStats, FileSummary, ProjectIndex};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn scan_repo(root: &Path) -> Result<ProjectIndex, String> {
    let metadata = cargo_metadata(root, true).or_else(|_| cargo_metadata(root, false))?;
    let json = parse(&metadata)?;
    let root_obj = obj(&json).ok_or_else(|| "cargo metadata json root invalid".to_string())?;

    let workspace_root = root_obj
        .get("workspace_root")
        .and_then(strv)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.to_path_buf());

    let packages_raw = root_obj
        .get("packages")
        .and_then(arr)
        .ok_or_else(|| "cargo metadata missing packages".to_string())?;

    let mut packages = Vec::new();
    let mut manifests = Vec::new();
    let mut stats = CargoTargetStats {
        bins: 0,
        libs: 0,
        examples: 0,
        tests: 0,
    };

    for p in packages_raw {
        if let Some(po) = obj(p) {
            if let Some(name) = po.get("name").and_then(strv) {
                packages.push(name.to_string());
            }
            if let Some(mp) = po.get("manifest_path").and_then(strv) {
                manifests.push(PathBuf::from(mp));
            }
            if let Some(targets) = po.get("targets").and_then(arr) {
                for t in targets {
                    if let Some(to) = obj(t) {
                        if let Some(kinds) = to.get("kind").and_then(arr) {
                            for k in kinds {
                                if let Some(kind) = strv(k) {
                                    match kind {
                                        "bin" => stats.bins += 1,
                                        "lib" | "proc-macro" => stats.libs += 1,
                                        "example" => stats.examples += 1,
                                        "test" => stats.tests += 1,
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    packages.sort();
    packages.dedup();
    manifests.sort();
    manifests.dedup();

    let files = collect_rs_files(&workspace_root)?;
    Ok(ProjectIndex {
        workspace_root,
        packages,
        manifests,
        target_stats: stats,
        file_summaries: files,
    })
}

fn cargo_metadata(root: &Path, offline: bool) -> Result<String, String> {
    let mut cmd = Command::new("cargo");
    cmd.arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .current_dir(root);
    if offline {
        cmd.arg("--offline");
    }
    let out = cmd
        .output()
        .map_err(|e| format!("cargo metadata exec error: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

fn collect_rs_files(root: &Path) -> Result<Vec<FileSummary>, String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
        for entry in entries {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                if p.ends_with("target") || p.ends_with(".git") {
                    continue;
                }
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|x| x.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(&p).map_err(|e| format!("read {}: {e}", p.display()))?;
            out.push(FileSummary {
                path: p.strip_prefix(root).unwrap_or(&p).to_path_buf(),
                lines: text.lines().count(),
                symbols: extract_symbols(&text),
                module_summary: summarize_module(&text),
            });
        }
    }
    out.sort_by(|a, b| b.lines.cmp(&a.lines));
    Ok(out)
}

fn extract_symbols(text: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("pub fn ") || t.starts_with("fn ") || t.starts_with("struct ") {
            symbols.push(t.split('{').next().unwrap_or(t).trim().to_string());
        }
        if symbols.len() >= 16 {
            break;
        }
    }
    symbols
}

fn summarize_module(text: &str) -> String {
    let mut counts = (0, 0, 0);
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("fn ") || t.starts_with("pub fn ") {
            counts.0 += 1;
        }
        if t.starts_with("struct ") || t.starts_with("enum ") {
            counts.1 += 1;
        }
        if t.contains("unsafe") {
            counts.2 += 1;
        }
    }
    format!(
        "functions={}, data_types={}, unsafe_mentions={}",
        counts.0, counts.1, counts.2
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_workspace_root() {
        let j = parse(r#"{"workspace_root":"/tmp/w","packages":[]}"#).unwrap();
        let o = obj(&j).unwrap();
        assert_eq!(o.get("workspace_root").and_then(strv), Some("/tmp/w"));
    }
}
