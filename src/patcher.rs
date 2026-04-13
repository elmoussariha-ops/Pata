use crate::fssec;
use crate::lock;
use crate::types::{PatchProposal, PatchReview};
use std::fs;
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
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let mut msg = v.as_bytes().to_vec();
    let bit_len = (msg.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let mut w = [0u32; 64];
    for chunk in msg.chunks_exact(64) {
        for (i, word) in chunk.chunks_exact(4).take(16).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = String::with_capacity(64);
    for part in h {
        out.push_str(&format!("{part:08x}"));
    }
    out
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
    fn checksum_is_sha256_hex() {
        let checksum = checksum_str("abc");
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
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
