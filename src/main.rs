mod ast_fingerprint;
mod coder;
mod evals;
mod fssec;
mod history;
mod json;
mod lock;
mod long_memory;
mod memory_engine;
mod model;
mod multiverse;
mod optimizer;
mod patcher;
mod planner;
mod retriever;
mod reviewer;
mod rollback;
mod scanner;
mod state_store;
mod status;
mod tester;
mod types;
mod ui;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const PROTECTED_PATHS: [&str; 4] = [
    "src/main.rs",
    "src/model.rs",
    "src/rollback.rs",
    "AGENTS.md",
];
const MAX_QUERY_LEN: usize = 240;
const MAX_OBJECTIVE_LEN: usize = 500;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn env_low_power() -> bool {
    matches!(
        std::env::var("PATA_LOW_POWER").ok().as_deref(),
        Some("1") | Some("true") | Some("on")
    )
}

fn parse_args() -> (bool, bool, String, Vec<String>) {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let mut low_power = env_low_power();
    let mut verbose = matches!(
        std::env::var("PATA_VERBOSE").ok().as_deref(),
        Some("1") | Some("true") | Some("on")
    );
    args.retain(|a| {
        if a == "--low-power" {
            low_power = true;
            false
        } else if a == "--verbose" {
            verbose = true;
            false
        } else {
            true
        }
    });
    let cmd = args.first().cloned().unwrap_or_else(|| "tui".to_string());
    let rest = if args.is_empty() {
        Vec::new()
    } else {
        args[1..].to_vec()
    };
    (low_power, verbose, cmd, rest)
}

fn run() -> Result<(), String> {
    let (low_power, verbose, cmd, rest) = parse_args();
    if verbose {
        std::env::set_var("PATA_VERBOSE", "1");
    }
    let root = env::current_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join(".pata")).map_err(|e| e.to_string())?;

    match cmd.as_str() {
        "scan" => command_scan(&root),
        "retrieve" => {
            let explain = rest.iter().any(|x| x == "--explain-retrieval");
            let query = rest
                .iter()
                .find(|x| !x.starts_with("--"))
                .cloned()
                .unwrap_or_default();
            command_retrieve(&root, query, low_power, explain)
        }
        "plan" => command_plan(&root, rest.first().cloned().unwrap_or_else(|| "improve rust quality".to_string()), low_power),
        "patch" => command_patch(&root, rest.first().cloned().unwrap_or_else(|| "fix rust".to_string()), low_power),
        "review" => {
            let explain = rest.iter().any(|x| x == "--explain-risk");
            let id = rest.iter().find(|x| !x.starts_with("--")).map(|s| s.as_str());
            command_review(&root, id, explain)
        }
        "approve" => command_approve(&root, rest.first().map(|s| s.as_str()), rest.get(1).map(|s| s.as_str()).unwrap_or("manual-approved")),
        "apply" => command_apply(&root, rest.first().map(|s| s.as_str())),
        "validate" => command_validate(&root),
        "evals" => command_evals(&root),
        "status" => command_status(&root, low_power),
        "end-session" | "daily-summary" => command_end_session(&root),
        "resume-session" => command_resume_session(&root),
        "memory" => command_memory(&root, &rest),
        "watch" => command_watch(&root, rest.first().and_then(|s| s.parse::<u32>().ok()).unwrap_or(30), low_power),
        "multiverse" => command_multiverse(
            &root,
            rest.first().cloned().unwrap_or_else(|| "improve rust reliability".to_string()),
            rest.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(3),
            low_power,
        ),
        "ast-fingerprint" => command_ast_fingerprint(&root, rest.first().map(|s| s.as_str())),
        "doctor" => command_doctor(&root, low_power, verbose),
        "smoke-test" => command_smoke_test(&root, low_power, verbose),
        "low-power-status" => command_low_power_status(low_power),
        "ollama-check" => command_ollama_check(),
        "ollama-status" => command_ollama_status(verbose),
        "model-status" => command_model_status(),
        "demo" => command_demo(&root, low_power),
        "tui" => command_tui(&root, low_power),
        _ => Err("usage: pata [--low-power] [--verbose] [scan|retrieve <q> [--explain-retrieval]|plan <goal>|patch <goal>|review <id> [--explain-risk]|approve <id> [decision]|apply <id>|validate|evals|status|end-session|daily-summary|resume-session|watch [cycles]|multiverse <goal> [2-4]|ast-fingerprint [function_name]|memory <show|recent|open-loops [--priority|--recent]|lessons|daily|weekly|digest|failures|failure-recent|promote-failure <id>|explain-open-loop <id>|similar-functions <fn_name>|add-open-loop <category> <detail> [priority] [module] [impact]|resolve-open-loop <id>|add-lesson <category> <detail>>|doctor|smoke-test|low-power-status|ollama-check|ollama-status|model-status|demo|tui]".to_string()),
    }
}

fn ensure_git_repo_root(root: &Path) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot run git rev-parse: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "invalid git repo: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let reported = std::path::PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    let canon_root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let canon_reported = fs::canonicalize(reported).map_err(|e| e.to_string())?;
    if canon_root != canon_reported {
        return Err("current directory is not the git repository root".to_string());
    }
    Ok(())
}

fn bounded_arg(mut v: String, max: usize) -> Result<String, String> {
    if v.len() > max {
        return Err(format!("argument too large (max {max})"));
    }
    v.truncate(max);
    Ok(v)
}

fn retrieval_limit(low_power: bool, normal: usize) -> usize {
    if low_power {
        normal.min(3)
    } else {
        normal
    }
}

fn latest_patch_id() -> Result<String, String> {
    let dir = PathBuf::from(".pata/patches");
    let mut ids = fs::read_dir(&dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str().map(ToString::to_string))
        .filter(|n| n.ends_with(".diff"))
        .map(|n| n.trim_end_matches(".diff").to_string())
        .collect::<Vec<_>>();
    ids.sort();
    ids.last()
        .cloned()
        .ok_or_else(|| "no patch found in .pata/patches".to_string())
}

fn command_scan(root: &Path) -> Result<(), String> {
    let idx = scanner::scan_repo(root)?;
    memory_engine::write_file_summaries(root, &idx.file_summaries)?;
    history::log(
        root,
        "scan",
        &format!(
            "workspace={} packages={} files={} bins={} libs={} examples={} tests={}",
            idx.workspace_root.display(),
            idx.packages.len(),
            idx.file_summaries.len(),
            idx.target_stats.bins,
            idx.target_stats.libs,
            idx.target_stats.examples,
            idx.target_stats.tests
        ),
    )?;
    println!(
        "scan ok: workspace={} crates={} manifests={} files={}",
        idx.workspace_root.display(),
        idx.packages.len(),
        idx.manifests.len(),
        idx.file_summaries.len(),
    );
    Ok(())
}

fn command_retrieve(
    root: &Path,
    query: String,
    low_power: bool,
    explain: bool,
) -> Result<(), String> {
    let query = bounded_arg(query, MAX_QUERY_LEN)?;
    let idx = scanner::scan_repo(root)?;
    let hits = retriever::top_n(
        &idx,
        &query,
        retrieval_limit(low_power, 6),
        &idx.workspace_root,
    );
    let mut hits = hits;
    let retrieval_boosts = long_memory::rerank_retrieval(root, &query, &mut hits);
    let recent_modules = long_memory::recent_modules(root, 8);
    memory_engine::write_retrieval_snapshot(root, &query, &hits)?;
    history::log(
        root,
        "retrieve",
        &format!("query='{query}' hits={} low_power={low_power}", hits.len()),
    )?;
    println!("retrieve query='{query}' hits={}", hits.len());
    if !recent_modules.is_empty() {
        println!("memory_recent_modules={}", recent_modules.join(" | "));
    }
    if !retrieval_boosts.is_empty() {
        println!("memory_boosts={}", retrieval_boosts.join(" | "));
    }
    if explain {
        println!(
            "explain_retrieval=score_base+boost_recent+boost_open_loops+boost_lessons_fragile"
        );
    }
    if query.split_whitespace().count() == 1 {
        let similar = ast_fingerprint::similar_functions(root, &query, 3);
        if !similar.is_empty() {
            println!(
                "memory_similar_functions={}",
                similar
                    .iter()
                    .map(|(f, d)| format!("{}::{}(d={})", f.path.display(), f.name, d))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
    }
    for h in hits {
        println!("- {} (score={})", h.path.display(), h.score);
    }
    Ok(())
}

fn command_plan(root: &Path, objective: String, low_power: bool) -> Result<(), String> {
    let objective = bounded_arg(objective, MAX_OBJECTIVE_LEN)?;
    let idx = scanner::scan_repo(root)?;
    let hits = retriever::top_n(
        &idx,
        &objective,
        retrieval_limit(low_power, 5),
        &idx.workspace_root,
    );
    let plan = planner::build_plan(&objective, &hits);
    println!("plan objective='{objective}'");
    for step in &plan {
        println!("- {step}");
    }
    for hint in long_memory::planner_hints(root, &objective) {
        println!("- {hint}");
    }
    memory_engine::append_task_event(root, "plan", &plan.join(" | "))?;
    history::log(root, "plan", &objective)?;
    Ok(())
}

fn command_patch(root: &Path, objective: String, low_power: bool) -> Result<(), String> {
    let objective = bounded_arg(objective, MAX_OBJECTIVE_LEN)?;
    let idx = scanner::scan_repo(root)?;
    let hits = retriever::top_n(
        &idx,
        &objective,
        retrieval_limit(low_power, 5),
        &idx.workspace_root,
    );
    let plan = planner::build_plan(&objective, &hits);
    memory_engine::append_task_event(root, "plan", &plan.join(" | "))?;

    let (diff, model_used) = coder::generate_patch(&objective, &hits)?;
    history::log(root, "patch_model", &model_used)?;

    let proposal = patcher::create(
        &objective,
        diff,
        hits.iter().map(|h| h.path.clone()).collect(),
    )?;

    let review = reviewer::review(&proposal, &PROTECTED_PATHS);
    let mut review = review;
    long_memory::adjust_review_with_memory(root, &mut review);
    patcher::save_review(&proposal.id, &review)?;
    memory_engine::append_patch_history(root, &proposal.id, review.risk.score)?;

    fs::create_dir_all(root.join(".pata/approvals")).map_err(|e| e.to_string())?;
    history::log(
        root,
        "patch",
        &format!(
            "id={} risk={} allowed={} low_power={low_power}",
            proposal.id, review.risk.score, review.risk.allowed
        ),
    )?;

    println!("patch proposal: {}", proposal.id);
    println!("summary: {}", review.summary);
    println!("files touched: {}", review.files.join(", "));
    println!("lines: +{} -{}", review.added_lines, review.removed_lines);
    println!("risk score: {}", review.risk.score);
    println!("reasons: {}", review.risk.reasons.join(" | "));
    println!(
        "sensitive zones: {}",
        review.risk.sensitive_zones.join(" | ")
    );
    println!("critical files: {}", review.risk.critical_files.join(" | "));
    println!("recommendation: {}", review.risk.recommendation);
    println!(
        "approval file required: {}",
        patcher::approval_file(&proposal.id).display()
    );
    Ok(())
}

fn command_review(root: &Path, id: Option<&str>, explain: bool) -> Result<(), String> {
    let patch_id = id.map(ToString::to_string).unwrap_or(latest_patch_id()?);
    let proposal = patcher::load(&patch_id)?;
    let review = reviewer::review(&proposal, &PROTECTED_PATHS);
    let mut review = review;
    long_memory::adjust_review_with_memory(root, &mut review);
    patcher::save_review(&patch_id, &review)?;
    println!("review id={patch_id}");
    println!("summary: {}", review.summary);
    println!("risk={} allowed={}", review.risk.score, review.risk.allowed);
    println!("recommendation={}", review.risk.recommendation);
    if explain {
        println!("explain_risk=base_reviewer_risk + memory_fragile_modules + memory_validation_priority + recurring_error_penalty");
    }
    Ok(())
}

fn command_approve(root: &Path, id: Option<&str>, decision: &str) -> Result<(), String> {
    let patch_id = id.map(ToString::to_string).unwrap_or(latest_patch_id()?);
    let approval_path = patcher::approve(&patch_id, decision)?;
    history::log(
        root,
        "approve",
        &format!("patch={patch_id} decision={decision}"),
    )?;
    println!("approved patch {patch_id}");
    println!("approval file: {}", approval_path.display());
    Ok(())
}

fn command_end_session(root: &Path) -> Result<(), String> {
    let summary = long_memory::summarize_session(root)?;
    long_memory::persist_summary(root, &summary)?;
    println!("end-session summary for {} ({})", summary.day, summary.week);
    println!("objectives={}", summary.objectives.join(" | "));
    println!("files_touched={}", summary.files_touched.join(" | "));
    println!("patches_created={}", summary.patches_created.join(" | "));
    println!("patches_applied={}", summary.patches_applied.join(" | "));
    println!("validations={}", summary.validations);
    println!("errors={}", summary.errors.join(" | "));
    println!("decisions={}", summary.decisions.join(" | "));
    println!("critical_modules={}", summary.critical_modules.join(" | "));
    println!("open_tasks={}", summary.open_tasks.join(" | "));
    println!("recommendations={}", summary.recommendations.join(" | "));
    history::log(
        root,
        "end_session",
        &format!("day={} week={}", summary.day, summary.week),
    )?;
    Ok(())
}

fn command_resume_session(root: &Path) -> Result<(), String> {
    let text = long_memory::render_recent(root);
    println!("{text}");
    history::log(root, "resume_session", "loaded compact memory context")
}

fn command_memory(root: &Path, rest: &[String]) -> Result<(), String> {
    let sub = rest.first().map(String::as_str).unwrap_or("show");
    match sub {
        "add-open-loop" => {
            let category = rest.get(1).ok_or_else(|| "missing category".to_string())?;
            let detail = rest.get(2).ok_or_else(|| "missing detail".to_string())?;
            let priority = rest.get(3).and_then(|v| v.parse::<u8>().ok());
            let module = rest.get(4).map(String::as_str);
            let impact = rest.get(5).map(String::as_str);
            let id = long_memory::add_open_loop(root, category, detail, priority, module, impact)?;
            println!("open loop added: {id}");
            history::log(
                root,
                "open_loop_add",
                &format!("id={id} category={category}"),
            )?;
            Ok(())
        }
        "resolve-open-loop" => {
            let id = rest.get(1).ok_or_else(|| "missing id".to_string())?;
            long_memory::resolve_open_loop(root, id)?;
            println!("open loop resolved: {id}");
            history::log(root, "open_loop_resolve", &format!("id={id}"))?;
            Ok(())
        }
        "add-lesson" => {
            let category = rest.get(1).ok_or_else(|| "missing category".to_string())?;
            let detail = rest.get(2).ok_or_else(|| "missing detail".to_string())?;
            long_memory::add_lesson(root, category, detail)?;
            println!("lesson added: {category}");
            history::log(root, "lesson_add", &format!("category={category}"))?;
            Ok(())
        }
        "promote-failure" => {
            let id = rest
                .get(1)
                .ok_or_else(|| "missing failure id".to_string())?;
            long_memory::promote_failure(root, id)?;
            println!("failure promoted: {id}");
            Ok(())
        }
        "explain-open-loop" => {
            let id = rest
                .get(1)
                .ok_or_else(|| "missing open-loop id".to_string())?;
            println!("{}", long_memory::explain_open_loop_priority(root, id));
            Ok(())
        }
        "similar-functions" => {
            let name = rest
                .get(1)
                .ok_or_else(|| "missing function name".to_string())?;
            let sims = ast_fingerprint::similar_functions(root, name, 8);
            if sims.is_empty() {
                println!("similar-functions empty for {name}");
            } else {
                for (f, d) in sims {
                    println!(
                        "{}\t{}\targs={}\tcontrol={}\tdistance={}",
                        f.path.display(),
                        f.name,
                        f.args,
                        f.control_score,
                        d
                    );
                }
            }
            Ok(())
        }
        "open-loops" => {
            let mode = if rest.iter().any(|x| x == "--priority") {
                "priority"
            } else if rest.iter().any(|x| x == "--recent") {
                "recent"
            } else {
                "default"
            };
            println!("{}", long_memory::render_open_loops(root, mode));
            Ok(())
        }
        "show" | "recent" | "lessons" | "daily" | "weekly" | "digest" | "failures"
        | "failure-recent" => {
            println!("{}", long_memory::render_view(root, sub));
            Ok(())
        }
        _ => Err("unknown memory command".to_string()),
    }
}

fn command_watch(root: &Path, cycles: u32, low_power: bool) -> Result<(), String> {
    let mut known = std::collections::BTreeMap::<PathBuf, u64>::new();
    let mut sleep_ms: u64 = if low_power { 2200 } else { 1200 };
    if cycles > 120 {
        sleep_ms = sleep_ms.saturating_add(800);
    }
    println!("watch start: cycles={cycles} low_power={low_power}");
    for i in 0..cycles {
        let idx = scanner::scan_repo(root)?;
        let mut changed = Vec::new();
        for f in &idx.file_summaries {
            if !f.path.to_string_lossy().ends_with(".rs") {
                continue;
            }
            let mtime = memory_engine::file_mtime_sec(&root.join(&f.path)).unwrap_or(0);
            let old = known.insert(f.path.clone(), mtime);
            if let Some(prev) = old {
                if mtime > prev {
                    changed.push(f.path.clone());
                }
            }
        }
        if !changed.is_empty() {
            if let Ok(fp) = ast_fingerprint::build_index(root) {
                let _ = ast_fingerprint::persist_index(root, &fp);
                println!("watch ast_fingerprint_functions={}", fp.len());
            }
            let focus = changed
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let mut hits = retriever::top_n(
                &idx,
                &focus,
                retrieval_limit(low_power, 4),
                &idx.workspace_root,
            );
            let boosts = long_memory::rerank_retrieval(root, &focus, &mut hits);
            println!(
                "watch change-detected cycle={} files={}",
                i + 1,
                changed.len()
            );
            println!(
                "watch changed={}",
                changed
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
            if let Some(top) = hits.first() {
                println!(
                    "watch retrieval_top={} score={}",
                    top.path.display(),
                    top.score
                );
            }
            if !boosts.is_empty() {
                println!("watch memory_boosts={}", boosts.join(" | "));
            }
            let hints = long_memory::planner_hints(root, &focus);
            if !hints.is_empty() {
                println!("watch suggestion_ready={}", hints.join(" | "));
            } else {
                println!("watch suggestion_ready=scan/retrieve refreshed");
            }
            history::log(
                root,
                "watch_change",
                &format!("changed={} top_hit={}", changed.len(), hits.len()),
            )?;
        }
        std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
    }
    println!("watch complete");
    Ok(())
}

fn command_multiverse(
    root: &Path,
    objective: String,
    branches: usize,
    low_power: bool,
) -> Result<(), String> {
    let objective = bounded_arg(objective, MAX_OBJECTIVE_LEN)?;
    let results = multiverse::run_multiverse(root, &objective, low_power, branches)?;
    println!(
        "multiverse objective='{objective}' branches={} (sorted by score)",
        results.len()
    );
    for r in &results {
        println!(
            "- {}: patch={} apply_check={} check={} test={} risk={} diff_lines={} score={} note={}",
            r.strategy,
            r.patch_id,
            r.apply_check_ok,
            r.check_ok,
            r.test_ok,
            r.risk,
            r.diff_lines,
            r.score,
            r.note
        );
    }
    if let Some(best) = results.first() {
        println!(
            "survivante choisie: {} ({}) score={}",
            best.strategy, best.patch_id, best.score
        );
    }
    history::log(
        root,
        "multiverse",
        &format!("objective='{objective}' branches={}", results.len()),
    )?;
    Ok(())
}

fn command_ast_fingerprint(root: &Path, function_name: Option<&str>) -> Result<(), String> {
    let idx = ast_fingerprint::build_index(root)?;
    ast_fingerprint::persist_index(root, &idx)?;
    println!("ast-fingerprint indexed_functions={}", idx.len());
    if let Some(name) = function_name {
        let sims = ast_fingerprint::similar_functions(root, name, 8);
        if sims.is_empty() {
            println!("similar-functions none for '{name}'");
        } else {
            println!("similar-functions for '{name}':");
            for (f, d) in sims {
                println!(
                    "- {}::{} args={} control={} distance={}",
                    f.path.display(),
                    f.name,
                    f.args,
                    f.control_score,
                    d
                );
            }
        }
    }
    Ok(())
}

fn command_apply(root: &Path, id: Option<&str>) -> Result<(), String> {
    ensure_git_repo_root(root)?;
    let patch_id = id.map(ToString::to_string).unwrap_or(latest_patch_id()?);
    let proposal = patcher::load(&patch_id)?;
    patcher::ensure_approved(&patch_id)?;

    rollback::checkpoint(root, &format!("pata-pre-apply-{patch_id}"))?;
    if let Err(e) = patcher::apply(root, &proposal) {
        let _ = long_memory::record_failure(
            root,
            &proposal.objective,
            "apply",
            "apply_failed",
            &patch_id,
            &e,
        );
        rollback::rollback(root, "HEAD~1")?;
        history::log(
            root,
            "rollback",
            &format!("apply failure for {patch_id}: {e}"),
        )?;
        return Err(format!("apply failed and rollback applied: {e}"));
    }

    let report = tester::validate(root);
    memory_engine::cache_validation_errors(root, &report)?;
    if !report.ok() {
        let _ = long_memory::record_failure(
            root,
            &proposal.objective,
            "validate-after-apply",
            "validation_failed_after_apply",
            &patch_id,
            &report.logs.join(" | "),
        );
        rollback::rollback(root, "HEAD~1")?;
        history::log(
            root,
            "rollback",
            &format!("validation failure for {patch_id}"),
        )?;
        return Err("validation failed after apply; rollback executed".to_string());
    }

    memory_engine::append_patch_history(root, &patch_id, 0)?;
    memory_engine::append_task_event(root, "apply", &format!("patch={patch_id}"))?;
    optimizer::optimization_tick(root)?;
    history::log(root, "apply", &format!("patch={patch_id} success"))?;
    println!("apply complete: {patch_id}");
    Ok(())
}

fn command_validate(root: &Path) -> Result<(), String> {
    let report = tester::validate(root);
    let eval_run = evals::run(root, &report);
    let eval_path = evals::persist(root, &report, &eval_run)?;
    memory_engine::cache_validation_errors(root, &report)?;
    state_store::write_last_validate(root, &report, &eval_run)?;
    println!(
        "validate: check={} clippy={} test={} ok={} eval_version={} total_ms={} regressions={}",
        report.check_ok,
        report.clippy_ok,
        report.test_ok,
        report.ok(),
        report.eval_version,
        report.total_duration_ms,
        report.regression_alerts.len()
    );
    println!(
        "evals: suite={} score={} blocked={}",
        eval_run.suite_version, eval_run.score_pct, eval_run.blocked
    );
    println!("evals_report={eval_path}");
    for case in &eval_run.cases {
        println!(
            "eval_case={} pass={} detail={}",
            case.name, case.passed, case.detail
        );
    }
    history::log(root, "validate", &format!("ok={}", report.ok()))?;
    if !report.ok() || eval_run.blocked {
        let _ = long_memory::record_failure(
            root,
            "manual validate",
            "validate",
            "validation_failed",
            "workspace",
            &report.logs.join(" | "),
        );
        let _ = state_store::write_last_warning(root, "validate failed");
        return Err("validate blocked by pipeline failure or eval regression".to_string());
    }
    Ok(())
}

fn command_evals(root: &Path) -> Result<(), String> {
    command_validate(root)
}

fn command_status(root: &Path, low_power: bool) -> Result<(), String> {
    let s = status::gather(root, low_power);
    let _ = state_store::write_last_status(root, &s);
    println!("git={}", s.git);
    println!("ollama={}", s.ollama);
    println!("model={}", s.model);
    println!("memory={}", s.memory);
    println!("validation={}", s.validation);
    println!("last_patch={}", s.last_patch);
    println!("last_rollback={}", s.last_rollback);
    println!("low_power={}", s.low_power);
    println!("warning={}", s.warning);
    if s.git.starts_with("error") {
        let _ = state_store::write_last_warning(root, "status check failed");
        Err("status check failed".to_string())
    } else {
        Ok(())
    }
}

fn command_doctor(root: &Path, low_power: bool, verbose: bool) -> Result<(), String> {
    let s = status::gather(root, low_power);
    println!("doctor: git={}", s.git);
    let settings = model::settings_from_env();
    let diag = model::diagnose_ollama(&settings);
    println!("doctor: ollama_state={}", diag.state);
    println!("doctor: message={}", diag.message);
    println!("doctor: hint={}", diag.hint);
    if verbose {
        let settings = model::settings_from_env();
        println!(
            "doctor: model={} timeout={} retries={} endpoint={}",
            settings.model, settings.timeout_sec, settings.retries, settings.endpoint
        );
    }
    println!("doctor: summary={}", model::diagnostic_summary(&diag));
    let _ = state_store::write_last_ollama_diag(root, &diag);
    if diag.state == "ok" {
        Ok(())
    } else {
        let _ = state_store::write_last_warning(root, &format!("doctor failed: {}", diag.state));
        Err(format!("doctor failed: {}", diag.state))
    }
}

fn command_smoke_test(root: &Path, low_power: bool, verbose: bool) -> Result<(), String> {
    command_scan(root)?;
    command_status(root, low_power)?;
    let settings = model::settings_from_env();
    let diag = model::diagnose_ollama(&settings);
    let _ = state_store::write_last_ollama_diag(root, &diag);
    if verbose {
        println!("smoke: summary={}", model::diagnostic_summary(&diag));
    }
    if diag.state != "ok" {
        let _ =
            state_store::write_last_warning(root, &format!("smoke-test blocked: {}", diag.state));
        return Err(format!(
            "smoke-test blocked: {} ({})",
            diag.state, diag.hint
        ));
    }
    let resp = model::smoke_generate(&settings)?;
    println!("smoke_response={}", resp.trim());
    Ok(())
}

fn command_low_power_status(low_power: bool) -> Result<(), String> {
    let settings = model::settings_from_env();
    println!("low_power={low_power}");
    println!(
        "retrieval_limit={} (for normal 6)",
        retrieval_limit(low_power, 6)
    );
    println!("ollama_timeout_sec={}", settings.timeout_sec);
    println!("ollama_retries={}", settings.retries);
    println!("ollama_max_tokens={}", settings.max_tokens);
    if settings.timeout_sec == 0 {
        Err("invalid timeout".to_string())
    } else {
        Ok(())
    }
}

fn command_ollama_check() -> Result<(), String> {
    let settings = model::settings_from_env();
    let status = model::ollama_status(&settings);
    println!("reachable={}", status.reachable);
    println!("message={}", status.message);
    if !status.reachable {
        return Err("ollama-check failed".to_string());
    }
    Ok(())
}

fn command_ollama_status(verbose: bool) -> Result<(), String> {
    let settings = model::settings_from_env();
    let status = model::ollama_status(&settings);
    let diag = model::diagnose_ollama(&settings);
    println!("endpoint={}", settings.endpoint);
    println!("selected_model={}", status.selected_model);
    println!("reachable={}", status.reachable);
    println!("installed_models={}", status.installed_models.join(", "));
    println!("diagnostic_state={}", diag.state);
    println!("diagnostic_hint={}", diag.hint);
    if verbose {
        println!("diagnostic_summary={}", model::diagnostic_summary(&diag));
        println!("diagnostic_message={}", diag.message);
    }
    if status.reachable {
        Ok(())
    } else {
        Err(status.message)
    }
}

fn command_model_status() -> Result<(), String> {
    let ram = model::detect_ram_gb_macos().unwrap_or(16);
    let auto_model = model::choose_model(ram);
    let settings = model::settings_from_env();
    println!("ram_gb={ram}");
    println!("auto_model={auto_model}");
    println!("active_model={}", settings.model);
    println!("temperature={}", settings.temperature);
    println!("max_tokens={}", settings.max_tokens);
    println!(
        "timeout_sec={} retries={}",
        settings.timeout_sec, settings.retries
    );
    if settings.model.is_empty() {
        Err("active model cannot be empty".to_string())
    } else {
        Ok(())
    }
}

fn command_demo(root: &Path, low_power: bool) -> Result<(), String> {
    let objective = "refactor scanner";
    command_scan(root)?;
    command_retrieve(root, objective.to_string(), low_power, false)?;
    command_plan(root, objective.to_string(), low_power)?;

    let idx = scanner::scan_repo(root)?;
    let hits = retriever::top_n(
        &idx,
        objective,
        retrieval_limit(low_power, 4),
        &idx.workspace_root,
    );
    let diff = "diff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-# Pata\n+# Pata MVP\n".to_string();
    let proposal = patcher::create(
        objective,
        diff,
        hits.iter().map(|h| h.path.clone()).collect(),
    )?;
    let review = reviewer::review(&proposal, &PROTECTED_PATHS);
    patcher::save_review(&proposal.id, &review)?;
    println!(
        "demo review risk={} allowed={}",
        review.risk.score, review.risk.allowed
    );

    if patcher::approval_file(&proposal.id).exists() {
        command_apply(root, Some(&proposal.id))?;
    } else {
        println!(
            "demo paused for approval: create {} then run cargo run -- apply {}",
            patcher::approval_file(&proposal.id).display(),
            proposal.id
        );
    }

    optimizer::optimization_tick(root)?;
    history::log(
        root,
        "demo",
        "pipeline complete with review/approve/apply gate",
    )?;
    println!(
        "demo complete: scan→retrieve→plan→patch→review→approve→apply→validate→memory→optimize"
    );
    Ok(())
}

fn command_tui(root: &Path, low_power: bool) -> Result<(), String> {
    let mut state = ui::UiState::new();
    let idx = scanner::scan_repo(root)?;
    state.files = idx
        .file_summaries
        .iter()
        .take(if low_power { 5 } else { 8 })
        .map(|f| f.path.display().to_string())
        .collect();

    state.memory = vec![
        format!("indexed_files={}", idx.file_summaries.len()),
        format!(
            "memory_file={}",
            root.join(".pata/memory/task_memory.jsonl").display()
        ),
        format!("low_power={low_power}"),
        format!(
            "last_validate={}",
            state_store::read_state_file(root, "last_validate.txt").replace("\n", " | ")
        ),
        format!(
            "last_ollama_diag={}",
            state_store::read_state_file(root, "last_ollama_diagnostic.txt").replace("\n", " | ")
        ),
    ];

    let s = status::gather(root, low_power);
    state.status = vec![
        format!("git={}", s.git),
        format!("ollama={}", s.ollama),
        format!("active_model={}", s.model),
        format!("last_patch={}", s.last_patch),
        format!("last_rollback={}", s.last_rollback),
        format!("validation={}", s.validation),
        format!("memory={}", s.memory),
        format!("warning={}", s.warning),
        format!(
            "last_warning={}",
            state_store::read_state_file(root, "last_warning.txt").trim()
        ),
    ];

    if let Ok(id) = latest_patch_id() {
        let review_txt =
            patcher::load_review(&id).unwrap_or_else(|e| format!("review unavailable: {e}"));
        state.patch_review = format!("patch={id}\n{}", review_txt);
    }

    loop {
        ui::render(&state);
        match ui::read_key() {
            Some('q') => break,
            Some('f') => state.active_view = "files",
            Some('c') => state.active_view = "chat",
            Some('t') => state.active_view = "terminal",
            Some('l') => state.active_view = "logs",
            Some('p') => state.active_view = "patch",
            Some('h') => {
                let log_path = PathBuf::from(".pata/logs/agent.log");
                let txt = fs::read_to_string(log_path).unwrap_or_else(|_| "no history".to_string());
                state
                    .chat
                    .push(txt.lines().last().unwrap_or("empty").to_string());
                state.active_view = "history";
            }
            Some('m') => state.active_view = "memory",
            Some('s') => state.active_view = "status",
            Some(other) => state.terminal.push(format!("unknown key {other}")),
            None => break,
        }
        if low_power {
            std::thread::sleep(std::time::Duration::from_millis(80));
        }
    }
    Ok(())
}

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use std::process::Command;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo(name: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pata-e2e-{name}-{ts}"));
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n",
        )
        .unwrap();
        fs::write(dir.join("src/main.rs"), "fn main(){println!(\"ok\");}\n").unwrap();
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

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sha256_hex(v: &str) -> String {
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
            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
                (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
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

    fn write_patch(dir: &Path, id: &str, diff: &str) {
        fs::create_dir_all(dir.join(".pata/patches")).unwrap();
        fs::write(dir.join(format!(".pata/patches/{id}.diff")), diff).unwrap();
        let checksum = sha256_hex(diff);
        fs::write(
            dir.join(format!(".pata/patches/{id}.meta")),
            format!("id={id}\nobjective=test\nchecksum={checksum}\n"),
        )
        .unwrap();
    }
    #[test]
    fn apply_refused_without_approval() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let repo = temp_repo("no-approval");
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo).unwrap();
        write_patch(&repo, "p1", "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-fn main(){println!(\"ok\");}\n+fn main(){println!(\"ok2\");}\n");
        let err = command_apply(&repo, Some("p1")).unwrap_err();
        assert!(err.contains("not approved"));
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn apply_success_with_approval() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let repo = temp_repo("apply-ok");
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo).unwrap();
        write_patch(&repo, "p2", "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-fn main(){println!(\"ok\");}\n+fn main(){println!(\"ok2\");}\n");
        patcher::approve("p2", "ok").unwrap();
        command_apply(&repo, Some("p2")).unwrap();
        let src = fs::read_to_string(repo.join("src/main.rs")).unwrap();
        assert!(src.contains("ok2"));
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn rollback_trace_written_on_apply_failure() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let repo = temp_repo("apply-fail");
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo).unwrap();
        write_patch(&repo, "p3", "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -99 +99 @@\n-nope\n+stillnope\n");
        patcher::approve("p3", "ok").unwrap();
        let err = command_apply(&repo, Some("p3")).unwrap_err();
        assert!(err.contains("rollback"));
        assert!(repo.join(".pata/last_rollback.txt").exists());
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn rollback_on_validation_failure() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let repo = temp_repo("validate-fail");
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo).unwrap();
        write_patch(&repo, "p4", "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-fn main(){println!(\"ok\");}\n+fn main(){ let x = ; }\n");
        patcher::approve("p4", "ok").unwrap();
        let err = command_apply(&repo, Some("p4")).unwrap_err();
        assert!(err.contains("validation failed"));
        let src = fs::read_to_string(repo.join("src/main.rs")).unwrap();
        assert!(src.contains("ok"));
        assert!(repo.join(".pata/last_rollback.txt").exists());
        std::env::set_current_dir(old).unwrap();
    }

    #[test]
    fn invalid_git_repo_is_reported() {
        let _guard = test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pata-e2e-nogit-{ts}"));
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(dir.join(".pata/patches")).unwrap();
        fs::write(dir.join(".pata/patches/p5.diff"), "").unwrap();
        fs::write(dir.join(".pata/patches/p5.meta"), "objective=x\n").unwrap();
        fs::create_dir_all(dir.join(".pata/approvals")).unwrap();
        fs::write(dir.join(".pata/approvals/p5.ok"), "ok\n").unwrap();
        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let err = command_apply(&dir, Some("p5")).unwrap_err();
        assert!(err.contains("git") || err.contains("not a git repository"));
        std::env::set_current_dir(old).unwrap();
    }
}
