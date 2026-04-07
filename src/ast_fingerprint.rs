use crate::fssec;
use crate::lock;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FunctionFingerprint {
    pub path: PathBuf,
    pub name: String,
    pub args: usize,
    pub has_return: bool,
    pub has_result: bool,
    pub has_async: bool,
    pub has_unsafe: bool,
    pub control_score: usize,
    pub shape_hash: String,
}

fn memory_file(root: &Path) -> PathBuf {
    root.join(".pata/memory/function_fingerprints.tsv")
}

fn shape_of(sig: &str, body_preview: &str) -> (usize, bool, bool, bool, bool, usize) {
    let args = sig
        .split('(')
        .nth(1)
        .and_then(|x| x.split(')').next())
        .map(|x| x.split(',').filter(|p| !p.trim().is_empty()).count())
        .unwrap_or(0);
    let has_return = sig.contains("->");
    let has_result = sig.contains("Result<") || body_preview.contains("Result<");
    let has_async = sig.contains("async fn");
    let has_unsafe = sig.contains("unsafe fn") || body_preview.contains("unsafe {");
    let control_score = ["if ", "match ", "for ", "while ", "loop "]
        .iter()
        .map(|k| body_preview.matches(k).count())
        .sum();
    (
        args,
        has_return,
        has_result,
        has_async,
        has_unsafe,
        control_score,
    )
}

fn hash_shape(
    name: &str,
    args: usize,
    has_return: bool,
    has_result: bool,
    control_score: usize,
) -> String {
    let mut h = DefaultHasher::new();
    format!("{name}:{args}:{has_return}:{has_result}:{control_score}").hash(&mut h);
    format!("{:x}", h.finish())
}

pub fn extract_from_file(path: &Path, content: &str) -> Vec<FunctionFingerprint> {
    let mut out = Vec::new();
    let lines = content.lines().collect::<Vec<_>>();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim_start();
        if !(t.starts_with("fn ")
            || t.starts_with("pub fn ")
            || t.starts_with("pub async fn ")
            || t.starts_with("async fn ")
            || t.starts_with("pub unsafe fn ")
            || t.starts_with("unsafe fn "))
        {
            continue;
        }
        let sig = t;
        let name = sig
            .split("fn ")
            .nth(1)
            .and_then(|s| s.split('(').next())
            .unwrap_or("unknown")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let body_preview = lines
            .iter()
            .skip(i)
            .take(40)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        let (args, has_return, has_result, has_async, has_unsafe, control_score) =
            shape_of(sig, &body_preview);
        let hash = hash_shape(&name, args, has_return, has_result, control_score);
        out.push(FunctionFingerprint {
            path: path.to_path_buf(),
            name,
            args,
            has_return,
            has_result,
            has_async,
            has_unsafe,
            control_score,
            shape_hash: hash,
        });
    }
    out
}

pub fn build_index(root: &Path) -> Result<Vec<FunctionFingerprint>, String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(&dir).map_err(|er| er.to_string())? {
            let e = e.map_err(|er| er.to_string())?;
            let p = e.path();
            if p.file_name().and_then(|x| x.to_str()) == Some("target") {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|x| x.to_str()) != Some("rs") {
                continue;
            }
            let rel = p.strip_prefix(root).unwrap_or(&p).to_path_buf();
            let txt = fs::read_to_string(&p).unwrap_or_default();
            out.extend(extract_from_file(&rel, &txt));
        }
    }
    Ok(out)
}

pub fn persist_index(root: &Path, idx: &[FunctionFingerprint]) -> Result<(), String> {
    let _guard = lock::acquire(root, "fingerprint-write")?;
    fs::create_dir_all(root.join(".pata/memory")).map_err(|e| e.to_string())?;
    fssec::set_secure_dir(&root.join(".pata/memory"))?;
    let mut txt = String::new();
    for f in idx {
        txt.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            f.path.display(),
            f.name,
            f.args,
            f.has_return,
            f.has_result,
            f.has_async,
            f.has_unsafe,
            f.control_score,
            f.shape_hash
        ));
    }
    fs::write(memory_file(root), txt).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&memory_file(root))
}

pub fn load_index(root: &Path) -> Vec<FunctionFingerprint> {
    fs::read_to_string(memory_file(root))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let c = l.split('\t').collect::<Vec<_>>();
            if c.len() != 9 {
                return None;
            }
            Some(FunctionFingerprint {
                path: PathBuf::from(c[0]),
                name: c[1].to_string(),
                args: c[2].parse().unwrap_or(0),
                has_return: c[3] == "true",
                has_result: c[4] == "true",
                has_async: c[5] == "true",
                has_unsafe: c[6] == "true",
                control_score: c[7].parse().unwrap_or(0),
                shape_hash: c[8].to_string(),
            })
        })
        .collect()
}

fn distance(a: &FunctionFingerprint, b: &FunctionFingerprint) -> usize {
    let mut d = 0usize;
    d += a.args.abs_diff(b.args);
    d += a.control_score.abs_diff(b.control_score);
    if a.has_return != b.has_return {
        d += 2;
    }
    if a.has_result != b.has_result {
        d += 3;
    }
    if a.has_async != b.has_async {
        d += 2;
    }
    if a.has_unsafe != b.has_unsafe {
        d += 4;
    }
    d
}

pub fn similar_functions(
    root: &Path,
    name: &str,
    top_n: usize,
) -> Vec<(FunctionFingerprint, usize)> {
    let idx = load_index(root);
    let target = idx.iter().find(|f| f.name == name).cloned();
    let Some(t) = target else {
        return Vec::new();
    };
    let mut scored = idx
        .into_iter()
        .filter(|f| f.name != name)
        .map(|f| {
            let d = distance(&t, &f);
            (f, d)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| a.1.cmp(&b.1));
    scored.truncate(top_n);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_extraction_works() {
        let src = "pub fn a(x:i32)->Result<(),String>{if x>0 {Ok(())} else {Err(\"x\".into())}}\nfn b(){for _i in 0..1 {}}\n";
        let fps = extract_from_file(Path::new("src/t.rs"), src);
        assert_eq!(fps.len(), 2);
        assert_eq!(fps[0].name, "a");
        assert!(fps[0].has_result);
    }
}
