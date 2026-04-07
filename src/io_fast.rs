use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Clone)]
struct CacheEntry {
    mtime: u64,
    len: u64,
    text: String,
}

fn cache() -> &'static Mutex<HashMap<PathBuf, CacheEntry>> {
    static C: OnceLock<Mutex<HashMap<PathBuf, CacheEntry>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clear_cache() {
    if let Ok(mut lock) = cache().lock() {
        lock.clear();
    }
}

fn stat_sig(path: &Path) -> Option<(u64, u64)> {
    let m = std::fs::metadata(path).ok()?;
    let ts = m
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some((ts, m.len()))
}

pub fn read_text(path: &Path) -> String {
    let Some((mtime, len)) = stat_sig(path) else {
        return std::fs::read_to_string(path).unwrap_or_default();
    };

    if let Ok(lock) = cache().lock() {
        if let Some(e) = lock.get(path) {
            if e.mtime == mtime && e.len == len {
                return e.text.clone();
            }
        }
    }

    let text = std::fs::read_to_string(path).unwrap_or_default();

    if let Ok(mut lock) = cache().lock() {
        lock.insert(
            path.to_path_buf(),
            CacheEntry {
                mtime,
                len,
                text: text.clone(),
            },
        );
        if lock.len() > 1200 {
            let keys = lock.keys().take(300).cloned().collect::<Vec<_>>();
            for k in keys {
                lock.remove(&k);
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_read_returns_content() {
        let p = std::env::temp_dir().join("pata-io-fast.txt");
        std::fs::write(&p, "hello").unwrap();
        let a = read_text(&p);
        let b = read_text(&p);
        assert_eq!(a, b);
    }
}
