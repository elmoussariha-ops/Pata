use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct ScopedLock {
    path: PathBuf,
}

impl Drop for ScopedLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn acquire(root: &Path, name: &str) -> Result<ScopedLock, String> {
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("invalid lock name".to_string());
    }
    let lock_dir = root.join(".pata/locks");
    fs::create_dir_all(&lock_dir).map_err(|e| e.to_string())?;
    let path = lock_dir.join(format!("{name}.lock"));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|_| format!("operation is locked: {}", path.display()))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let pid = std::process::id();
    let _ = writeln!(f, "pid={pid}\nts={ts}");
    Ok(ScopedLock { path })
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
        let dir = std::env::temp_dir().join(format!("pata-lock-{ts}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn lock_conflict_is_reported() {
        let root = temp_root();
        let _a = acquire(&root, "apply").unwrap();
        let err = acquire(&root, "apply").unwrap_err();
        assert!(err.contains("locked"));
    }
}
