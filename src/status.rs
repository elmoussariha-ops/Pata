use crate::model;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GlobalStatus {
    pub git: String,
    pub ollama: String,
    pub model: String,
    pub memory: String,
    pub validation: String,
    pub last_patch: String,
    pub last_rollback: String,
    pub low_power: bool,
    pub warning: String,
}

pub fn gather(root: &Path, low_power: bool) -> GlobalStatus {
    let git = git_status(root);
    let settings = model::settings_from_env();
    let ollama_diag = model::diagnose_ollama(&settings);
    let ollama = format!("{} ({})", ollama_diag.state, ollama_diag.hint);

    let validation_path = root.join(".pata/memory/cargo_errors_latest.log");
    let validation = if validation_path.exists() {
        let txt = fs::read_to_string(&validation_path).unwrap_or_default();
        if txt.trim().is_empty() {
            "last-validate: clean".to_string()
        } else {
            format!("last-validate: issues ({})", txt.lines().count())
        }
    } else {
        "last-validate: unknown".to_string()
    };

    let last_patch = latest_patch_id(root).unwrap_or_else(|| "none".to_string());
    let last_rollback = fs::read_to_string(root.join(".pata/last_rollback.txt"))
        .unwrap_or_else(|_| "none".to_string())
        .trim()
        .to_string();

    let memory = if root.join(".pata/memory/file_signatures.tsv").exists() {
        "memory cache ready".to_string()
    } else {
        "memory cache missing".to_string()
    };

    GlobalStatus {
        git,
        ollama,
        model: settings.model,
        memory,
        validation,
        last_patch,
        last_rollback,
        low_power,
        warning: ollama_diag.message,
    }
}

fn git_status(root: &Path) -> String {
    let out = Command::new("git")
        .args(["status", "--short"])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let txt = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if txt.is_empty() {
                "clean".to_string()
            } else {
                format!("dirty ({})", txt.lines().count())
            }
        }
        Ok(o) => format!("invalid ({})", String::from_utf8_lossy(&o.stderr).trim()),
        Err(e) => format!("error ({e})"),
    }
}

fn latest_patch_id(root: &Path) -> Option<String> {
    let dir = root.join(".pata/patches");
    let mut ids = fs::read_dir(&dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str().map(ToString::to_string))
        .filter(|n| n.ends_with(".diff"))
        .map(|n| n.trim_end_matches(".diff").to_string())
        .collect::<Vec<_>>();
    ids.sort();
    ids.last().cloned()
}
