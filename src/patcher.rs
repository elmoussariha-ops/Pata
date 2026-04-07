use crate::fssec;
use crate::lock;
use crate::types::{PatchProposal, PatchReview};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_PATCH_BYTES: usize = 2_000_000;
const MAX_META_BYTES: usize = 200_000;

pub fn create(objective: &str, diff: String, files: Vec<PathBuf>) -> Result<PatchProposal, String> {
    if diff.len() > MAX_PATCH_BYTES {
        return Err(format!("patch too large: {} bytes", diff.len()));
    }
    let _guard = lock::acquire(&current_root()?, "patch-write")?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let id = format!("patch-{ts}");
    validate_patch_id(&id)?;

    let p = PatchProposal {
        id: id.clone(),
        objective: objective.to_string(),
        diff,
        files,
    };

    let patch_dir = ensure_safe_dir(Path::new(".pata/patches"))?;
    let diff_path = patch_dir.join(format!("{id}.diff"));
    fs::write(&diff_path, &p.diff).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&diff_path)?;

    let checksum = checksum_str(&p.diff);
    let meta_path = patch_dir.join(format!("{id}.meta"));
    fs::write(
        &meta_path,
        format!(
            "id={id}\nobjective={}\nfiles={:?}\nchecksum={checksum}\n",
            sanitize_line(&p.objective),
            p.files
        ),
    )
    .map_err(|e| e.to_string())?;
    fssec::set_secure_file(&meta_path)?;
    Ok(p)
}

pub fn load(id: &str) -> Result<PatchProposal, String> {
    validate_patch_id(id)?;
    let patch_dir = ensure_safe_dir(Path::new(".pata/patches"))?;
    let diff_path = patch_dir.join(format!("{id}.diff"));
    let meta_path = patch_dir.join(format!("{id}.meta"));

    check_regular_file_size(&diff_path, MAX_PATCH_BYTES)?;
    check_regular_file_size(&meta_path, MAX_META_BYTES)?;

    let diff = fs::read_to_string(&diff_path)
        .map_err(|e| format!("cannot read patch diff {}: {e}", diff_path.display()))?;
    let meta = fs::read_to_string(&meta_path)
        .map_err(|e| format!("cannot read patch metadata {}: {e}", meta_path.display()))?;

    let id_meta = meta.lines().find_map(|l| l.strip_prefix("id="));
    if id_meta != Some(id) {
        return Err("meta id mismatch".to_string());
    }

    let objective = meta
        .lines()
        .find_map(|l| l.strip_prefix("objective="))
        .unwrap_or("unknown objective")
        .to_string();

    let checksum_meta = meta
        .lines()
        .find_map(|l| l.strip_prefix("checksum="))
        .ok_or_else(|| "meta checksum missing".to_string())?;
    let checksum_real = checksum_str(&diff);
    if checksum_meta != checksum_real {
        return Err("meta checksum mismatch".to_string());
    }

    Ok(PatchProposal {
        id: id.to_string(),
        objective,
        diff,
        files: vec![],
    })
}

pub fn save_review(id: &str, review: &PatchReview) -> Result<(), String> {
    validate_patch_id(id)?;
    let _guard = lock::acquire(&current_root()?, "patch-write")?;
    let patch_dir = ensure_safe_dir(Path::new(".pata/patches"))?;
    let payload = format!(
        "summary={}\nhunks={}\nrisk_score={}\nallowed={}\nreasons={}\ncritical={}\nrecommendation={}\n",
        sanitize_line(&review.summary),
        review.hunk_count,
        review.risk.score,
        review.risk.allowed,
        sanitize_line(&review.risk.reasons.join(" | ")),
        sanitize_line(&review.risk.critical_files.join(" | ")),
        sanitize_line(&review.risk.recommendation)
    );
    let checksum = checksum_str(&payload);
    let review_path = patch_dir.join(format!("{id}.review"));
    fs::write(&review_path, format!("{payload}checksum={checksum}\n"))
        .map_err(|e| e.to_string())?;
    fssec::set_secure_file(&review_path)?;
    Ok(())
}

pub fn load_review(id: &str) -> Result<String, String> {
    validate_patch_id(id)?;
    let patch_dir = ensure_safe_dir(Path::new(".pata/patches"))?;
    let review_path = patch_dir.join(format!("{id}.review"));
    check_regular_file_size(&review_path, MAX_META_BYTES)?;
    let text = fs::read_to_string(&review_path).map_err(|e| format!("cannot read review: {e}"))?;
    let mut lines = text.lines().collect::<Vec<_>>();
    let checksum_line = lines.pop().ok_or_else(|| "review is empty".to_string())?;
    let checksum_saved = checksum_line
        .strip_prefix("checksum=")
        .ok_or_else(|| "review checksum missing".to_string())?;
    let mut payload = lines.join("\n");
    payload.push('\n');
    let checksum_real = checksum_str(&payload);
    if checksum_real != checksum_saved {
        return Err("review checksum mismatch".to_string());
    }
    Ok(payload)
}

pub fn approval_file(id: &str) -> PathBuf {
    let _ = validate_patch_id(id);
    PathBuf::from(format!(".pata/approvals/{id}.ok"))
}

pub fn approve(id: &str, decision: &str) -> Result<PathBuf, String> {
    validate_patch_id(id)?;
    if decision.len() > 300 {
        return Err("approval decision too long".to_string());
    }
    let _guard = lock::acquire(&current_root()?, "approve")?;
    let approval_dir = ensure_safe_dir(Path::new(".pata/approvals"))?;
    let path = approval_dir.join(format!("{id}.ok"));
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    fs::write(
        &path,
        format!(
            "approved=true\ndecision={}\nts={ts}\n",
            sanitize_line(decision)
        ),
    )
    .map_err(|e| e.to_string())?;
    fssec::set_secure_file(&path)?;
    Ok(path)
}

pub fn ensure_approved(id: &str) -> Result<PathBuf, String> {
    validate_patch_id(id)?;
    let approval_dir = ensure_safe_dir(Path::new(".pata/approvals"))?;
    let path = approval_dir.join(format!("{id}.ok"));
    if !path.exists() {
        return Err(format!(
            "patch '{id}' not approved. create file {} first",
            path.display()
        ));
    }
    check_regular_file_size(&path, 20_000)?;
    let text = fs::read_to_string(&path).map_err(|e| format!("cannot read approval: {e}"))?;
    let approved = text.lines().any(|l| l.trim() == "approved=true");
    if !approved {
        return Err(format!("approval file {} is invalid", path.display()));
    }
    Ok(path)
}

pub fn apply(root: &Path, p: &PatchProposal) -> Result<(), String> {
    let _guard = lock::acquire(root, "apply")?;
    ensure_approved(&p.id)?;
    validate_patch_id(&p.id)?;
    let file = root.join(".pata/patches").join(format!("{}.diff", p.id));
    check_regular_file_size(&file, MAX_PATCH_BYTES)?;
    let out = std::process::Command::new("git")
        .arg("apply")
        .arg(&file)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git apply failed: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git apply failed (code={}): {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

fn validate_patch_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 80 {
        return Err("invalid patch id length".to_string());
    }
    if id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        Ok(())
    } else {
        Err("invalid patch id characters".to_string())
    }
}

fn ensure_safe_dir(path: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(path).map_err(|e| format!("cannot create dir {}: {e}", path.display()))?;
    let canon = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let root = current_root()?.canonicalize().map_err(|e| e.to_string())?;
    if !canon.starts_with(root) {
        return Err(format!("path escapes repo: {}", canon.display()));
    }
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err(format!("refusing symlink directory: {}", path.display()));
    }
    if !meta.is_dir() {
        return Err(format!("not a directory: {}", path.display()));
    }
    fssec::set_secure_dir(path)?;
    Ok(path.to_path_buf())
}

fn check_regular_file_size(path: &Path, max: usize) -> Result<(), String> {
    let meta =
        fs::symlink_metadata(path).map_err(|e| format!("cannot stat {}: {e}", path.display()))?;
    if meta.file_type().is_symlink() {
        return Err(format!("refusing symlink file: {}", path.display()));
    }
    if !meta.is_file() {
        return Err(format!("not a regular file: {}", path.display()));
    }
    if meta.len() > max as u64 {
        return Err(format!("file too large: {}", path.display()));
    }
    Ok(())
}

fn sanitize_line(s: &str) -> String {
    s.replace(['\n', '\r', '\t'], " ")
}

fn checksum_str(v: &str) -> String {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    format!("{:x}", h.finish())
}

fn current_root() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pata-test-{ts}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn approval_file_is_required() {
        let dir = temp_dir();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let err = ensure_approved("abc").unwrap_err();
        assert!(err.contains("not approved"));
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn approve_creates_file() {
        let dir = temp_dir();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let path = approve("abc", "ok").unwrap();
        assert!(path.exists());
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn invalid_patch_id_is_rejected() {
        let err = validate_patch_id("../evil").unwrap_err();
        assert!(err.contains("invalid"));
    }

    #[test]
    fn corrupted_approval_is_rejected() {
        let dir = temp_dir();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        std::fs::create_dir_all(".pata/approvals").unwrap();
        std::fs::write(".pata/approvals/a1.ok", "bad=true\n").unwrap();
        let err = ensure_approved("a1").unwrap_err();
        assert!(err.contains("invalid"));
        std::env::set_current_dir(old).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_permissions_applied_on_approval() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let path = approve("perm1", "ok").unwrap();
        let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn corrupted_review_is_rejected() {
        let dir = temp_dir();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        std::fs::create_dir_all(".pata/patches").unwrap();
        std::fs::write(
            ".pata/patches/p1.review",
            "summary=x
checksum=bad
",
        )
        .unwrap();
        let err = load_review("p1").unwrap_err();
        assert!(err.contains("checksum") || err.contains("missing"));
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn corrupted_meta_is_rejected() {
        let dir = temp_dir();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        std::fs::create_dir_all(".pata/patches").unwrap();
        std::fs::write(".pata/patches/p1.diff", "abc").unwrap();
        std::fs::write(".pata/patches/p1.meta", "id=p1\nchecksum=wrong\n").unwrap();
        let err = load("p1").unwrap_err();
        assert!(err.contains("checksum") || err.contains("mismatch"));
        std::env::set_current_dir(old).unwrap();
    }
}
