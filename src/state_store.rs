use crate::fssec;
use crate::lock;
use crate::status::GlobalStatus;
use crate::types::{OllamaDiagnostic, ValidationResult};
use std::fs;
use std::path::{Path, PathBuf};

fn state_dir(root: &Path) -> PathBuf {
    root.join(".pata/state")
}

fn ensure_state_dir(root: &Path) -> Result<PathBuf, String> {
    let dir = state_dir(root);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let meta = fs::symlink_metadata(&dir).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("state dir cannot be symlink".to_string());
    }
    fssec::set_secure_dir(&dir)?;
    Ok(dir)
}

pub fn write_last_validate(root: &Path, report: &ValidationResult) -> Result<(), String> {
    let _guard = lock::acquire(root, "state-write")?;
    let dir = ensure_state_dir(root)?;
    let path = dir.join("last_validate.txt");
    fs::write(
        &path,
        format!(
            "ok={}\ncheck={}\nclippy={}\ntest={}\n",
            report.ok(),
            report.check_ok,
            report.clippy_ok,
            report.test_ok
        ),
    )
    .map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)
}

pub fn write_last_ollama_diag(root: &Path, diag: &OllamaDiagnostic) -> Result<(), String> {
    let _guard = lock::acquire(root, "state-write")?;
    let dir = ensure_state_dir(root)?;
    let path = dir.join("last_ollama_diagnostic.txt");
    fs::write(
        &path,
        format!(
            "state={}\nmessage={}\nhint={}\n",
            sanitize(&diag.state),
            sanitize(&diag.message),
            sanitize(&diag.hint)
        ),
    )
    .map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)
}

pub fn write_last_status(root: &Path, s: &GlobalStatus) -> Result<(), String> {
    let _guard = lock::acquire(root, "state-write")?;
    let dir = ensure_state_dir(root)?;
    let path = dir.join("last_status.txt");
    fs::write(
        &path,
        format!(
            "git={}\nollama={}\nmodel={}\nmemory={}\nvalidation={}\nlast_patch={}\nlast_rollback={}\nlow_power={}\nwarning={}\n",
            sanitize(&s.git),
            sanitize(&s.ollama),
            sanitize(&s.model),
            sanitize(&s.memory),
            sanitize(&s.validation),
            sanitize(&s.last_patch),
            sanitize(&s.last_rollback),
            s.low_power,
            sanitize(&s.warning)
        ),
    )
    .map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)
}

pub fn write_last_warning(root: &Path, msg: &str) -> Result<(), String> {
    let _guard = lock::acquire(root, "state-write")?;
    let dir = ensure_state_dir(root)?;
    let path = dir.join("last_warning.txt");
    fs::write(&path, format!("{}\n", sanitize(msg))).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)
}

pub fn read_state_file(root: &Path, name: &str) -> String {
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return "invalid-state-name".to_string();
    }
    let path = state_dir(root).join(name);
    let meta = match fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(_) => return "unavailable".to_string(),
    };
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > 200_000 {
        return "unavailable".to_string();
    }
    fs::read_to_string(path)
        .unwrap_or_else(|_| "unavailable".to_string())
        .chars()
        .take(4000)
        .collect()
}

fn sanitize(v: &str) -> String {
    v.replace(['\n', '\r', '\t'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pata-state-{ts}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn invalid_state_name_is_rejected() {
        let root = temp_root();
        let v = read_state_file(&root, "../bad");
        assert_eq!(v, "invalid-state-name");
    }

    #[test]
    fn oversized_state_file_is_ignored() {
        let root = temp_root();
        let dir = root.join(".pata/state");
        fs::create_dir_all(&dir).unwrap();
        let data = "x".repeat(220_000);
        fs::write(dir.join("last_status.txt"), data).unwrap();
        let v = read_state_file(&root, "last_status.txt");
        assert_eq!(v, "unavailable");
    }
}
