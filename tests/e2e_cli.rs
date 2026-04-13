use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_repo() -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("pata-cli-e2e-{ts}"));
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n",
    )
    .unwrap();
    fs::write(dir.join("src/main.rs"), "fn main(){println!(\"hello\");}\n").unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "tester"])
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(&dir)
        .output()
        .unwrap();
    dir
}

fn run_ok(repo: &PathBuf, args: &[&str], envs: &[(&str, &str)]) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pata"));
    cmd.current_dir(repo).args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    if !out.status.success() {
        panic!(
            "command failed: {:?}\nstdout={}\nstderr={}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn full_cli_pipeline_happy_path_with_mock_patch() {
    let repo = temp_repo();
    let fake_diff = "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-fn main(){println!(\"hello\");}\n+fn main(){println!(\"hello2\");}\n";
    let envs = [("PATA_FAKE_PATCH", fake_diff), ("PATA_LOW_POWER", "1")];

    run_ok(&repo, &["scan"], &envs);
    run_ok(&repo, &["retrieve", "main"], &envs);
    run_ok(&repo, &["plan", "update main"], &envs);
    let patch_out = run_ok(&repo, &["patch", "update main"], &envs);
    let patch_id_line = patch_out
        .lines()
        .find(|l| l.starts_with("patch proposal:"))
        .expect("missing patch id line");
    let patch_id = patch_id_line.replace("patch proposal: ", "");

    run_ok(&repo, &["review", &patch_id], &envs);
    run_ok(&repo, &["approve", &patch_id, "ok"], &envs);
    run_ok(&repo, &["apply", &patch_id], &envs);
    run_ok(&repo, &["validate"], &envs);
    let eval_out = run_ok(&repo, &["evals"], &envs);
    assert!(eval_out.contains("evals: suite=evals.2026-04-13.v1"));
    assert!(repo
        .join(".pata/evals/runs/evals.2026-04-13.v1.txt")
        .exists());
    let status_out = run_ok(&repo, &["status"], &envs);
    assert!(status_out.contains("low_power=true"));

    let src = fs::read_to_string(repo.join("src/main.rs")).unwrap();
    assert!(src.contains("hello2"));
}
