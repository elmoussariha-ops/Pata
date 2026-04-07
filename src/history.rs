use crate::fssec;
use crate::lock;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log(root: &Path, event: &str, message: &str) -> Result<(), String> {
    let _guard = lock::acquire(root, "log-write")?;
    let log_dir = root.join(".pata/logs");
    fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    fssec::set_secure_dir(&log_dir)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let line = format!(
        "{ts}\t{}\t{}\n",
        sanitize_field(event),
        sanitize_field(message)
    );
    let path = log_dir.join("agent.log");
    let mut cur = fs::read_to_string(&path).unwrap_or_default();
    if cur.len() > 2_000_000 {
        cur = cur
            .lines()
            .rev()
            .take(5000)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        cur.push('\n');
    }
    cur.push_str(&line);
    fs::write(&path, cur).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)
}

fn sanitize_field(v: &str) -> String {
    v.replace(['\t', '\n', '\r'], " ")
}
