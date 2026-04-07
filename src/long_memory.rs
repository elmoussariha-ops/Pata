use crate::fssec;
use crate::lock;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_TEXT: usize = 600;

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub ts: u64,
    pub day: String,
    pub week: String,
    pub objectives: Vec<String>,
    pub files_touched: Vec<String>,
    pub patches_created: Vec<String>,
    pub patches_applied: Vec<String>,
    pub validations: usize,
    pub errors: Vec<String>,
    pub decisions: Vec<String>,
    pub critical_modules: Vec<String>,
    pub open_tasks: Vec<String>,
    pub recommendations: Vec<String>,
}

fn mem_root(root: &Path) -> PathBuf {
    root.join(".pata/memory")
}

fn day_week_from_epoch(ts: u64) -> (String, String) {
    let days = (ts / 86_400) as i64;
    let (y, m, d) = civil_from_days(days);
    let jan1_days = days_from_civil(y, 1, 1);
    let jan1_weekday_mon0 = (jan1_days + 3).rem_euclid(7);
    let week1_start = jan1_days - jan1_weekday_mon0;
    let week = 1 + ((days - week1_start) / 7);
    (
        format!("{y:04}-{m:02}-{d:02}"),
        format!("{y:04}-W{week:02}"),
    )
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let m = month as i64;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + if m <= 2 { 1 } else { 0 }, m as u32, d as u32)
}

fn sanitize(v: &str) -> String {
    v.chars()
        .filter(|c| !c.is_control() || *c == ' ')
        .collect::<String>()
        .replace(['\n', '\r', '\t'], " ")
        .chars()
        .take(MAX_TEXT)
        .collect::<String>()
}

fn parse_agent_log(line: &str) -> Option<(u64, String, String)> {
    let mut parts = line.splitn(3, '\t');
    let ts = parts.next()?.parse::<u64>().ok()?;
    let event = parts.next()?.to_string();
    let message = parts.next()?.to_string();
    Some((ts, event, message))
}

fn load_session_cutoff(root: &Path) -> u64 {
    let log = fs::read_to_string(mem_root(root).join("sessions.log")).unwrap_or_default();
    log.lines()
        .last()
        .and_then(|l| l.split('\t').next())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

fn load_open_loops(root: &Path) -> Vec<(String, String, String, String)> {
    let p = mem_root(root).join("open_loops.tsv");
    let txt = fs::read_to_string(p).unwrap_or_default();
    txt.lines()
        .filter_map(|l| {
            let cols = l.split('\t').collect::<Vec<_>>();
            if cols.len() == 4 {
                Some((
                    cols[0].to_string(),
                    cols[1].to_string(),
                    cols[2].to_string(),
                    cols[3].to_string(),
                ))
            } else {
                None
            }
        })
        .collect()
}

pub fn add_open_loop(root: &Path, category: &str, text: &str) -> Result<String, String> {
    let _guard = lock::acquire(root, "memory-open-loop")?;
    fs::create_dir_all(mem_root(root)).map_err(|e| e.to_string())?;
    fssec::set_secure_dir(&mem_root(root))?;
    let ts = now_ts()?;
    let id = format!("ol-{ts}");
    let line = format!("{id}\t{}\topen\t{}\n", sanitize(category), sanitize(text));
    let p = mem_root(root).join("open_loops.tsv");
    let mut cur = fs::read_to_string(&p).unwrap_or_default();
    cur.push_str(&line);
    fs::write(&p, cur).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&p)?;
    Ok(id)
}

pub fn resolve_open_loop(root: &Path, id: &str) -> Result<(), String> {
    let _guard = lock::acquire(root, "memory-open-loop")?;
    let rows = load_open_loops(root);
    let mut out = String::new();
    let mut found = false;
    for (rid, category, status, detail) in rows {
        let next_status = if rid == id {
            found = true;
            "closed"
        } else {
            &status
        };
        out.push_str(&format!("{rid}\t{category}\t{next_status}\t{detail}\n"));
    }
    if !found {
        return Err(format!("open loop not found: {id}"));
    }
    let p = mem_root(root).join("open_loops.tsv");
    fs::write(&p, out).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&p)
}

pub fn add_lesson(root: &Path, category: &str, text: &str) -> Result<(), String> {
    let _guard = lock::acquire(root, "memory-lesson")?;
    fs::create_dir_all(mem_root(root)).map_err(|e| e.to_string())?;
    fssec::set_secure_dir(&mem_root(root))?;
    let ts = now_ts()?;
    let p = mem_root(root).join("lessons.tsv");
    let mut cur = fs::read_to_string(&p).unwrap_or_default();
    cur.push_str(&format!(
        "{ts}\t{}\t{}\n",
        sanitize(category),
        sanitize(text)
    ));
    fs::write(&p, cur).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&p)
}

fn now_ts() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())
        .map(|d| d.as_secs())
}

pub fn summarize_session(root: &Path) -> Result<SessionSummary, String> {
    let ts = now_ts()?;
    let cutoff = load_session_cutoff(root);
    let (day, week) = day_week_from_epoch(ts);
    let log = fs::read_to_string(root.join(".pata/logs/agent.log")).unwrap_or_default();

    let mut objectives = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut patches_created = BTreeSet::new();
    let mut patches_applied = BTreeSet::new();
    let mut errors = Vec::new();
    let mut decisions = Vec::new();
    let mut critical = BTreeSet::new();
    let mut validations = 0usize;

    for line in log.lines() {
        let Some((lts, event, message)) = parse_agent_log(line) else {
            continue;
        };
        if lts <= cutoff {
            continue;
        }
        match event.as_str() {
            "plan" => {
                objectives.insert(sanitize(&message));
            }
            "patch" => {
                if let Some(pid) = message
                    .split_whitespace()
                    .find_map(|w| w.strip_prefix("id="))
                {
                    patches_created.insert(pid.to_string());
                }
            }
            "apply" => {
                if let Some(pid) = message
                    .split_whitespace()
                    .find_map(|w| w.strip_prefix("patch="))
                {
                    patches_applied.insert(pid.to_string());
                }
            }
            "validate" => validations += 1,
            "rollback" => errors.push(format!("rollback: {}", sanitize(&message))),
            "approve" => decisions.push(format!("approval: {}", sanitize(&message))),
            _ => {}
        }
    }

    let patch_dir = root.join(".pata/patches");
    if patch_dir.exists() {
        for pid in &patches_created {
            let review =
                fs::read_to_string(patch_dir.join(format!("{pid}.review"))).unwrap_or_default();
            for l in review.lines() {
                if let Some(v) = l.strip_prefix("critical=") {
                    for x in v.split(" | ") {
                        if !x.trim().is_empty() {
                            critical.insert(x.trim().to_string());
                            files.insert(x.trim().to_string());
                        }
                    }
                }
            }
            let meta =
                fs::read_to_string(patch_dir.join(format!("{pid}.meta"))).unwrap_or_default();
            if let Some(fline) = meta.lines().find_map(|l| l.strip_prefix("files=")) {
                for token in fline.split(['[', ']', ',', '"']) {
                    let t = token.trim();
                    if !t.is_empty() && (t.contains('/') || t.ends_with(".rs")) {
                        files.insert(t.to_string());
                    }
                }
            }
        }
    }

    let open_tasks = load_open_loops(root)
        .into_iter()
        .filter(|(_, _, status, _)| status == "open")
        .map(|(id, cat, _, detail)| format!("{id}:{cat}:{detail}"))
        .collect::<Vec<_>>();

    let mut recommendations = Vec::new();
    if !errors.is_empty() {
        recommendations.push(
            "Rejouer validate après correction des erreurs de rollback/validation".to_string(),
        );
    }
    if !open_tasks.is_empty() {
        recommendations
            .push("Traiter les open loops prioritaires avant un nouveau patch large".to_string());
    }
    if validations == 0 {
        recommendations.push("Lancer validate pour figer un état fiable de la session".to_string());
    }
    if recommendations.is_empty() {
        recommendations
            .push("Session stable: reprendre avec resume-session puis patch ciblé".to_string());
    }

    if root.join(".pata/memory/cargo_errors_latest.log").exists() {
        let e = fs::read_to_string(root.join(".pata/memory/cargo_errors_latest.log"))
            .unwrap_or_default();
        if !e.trim().is_empty() {
            errors.push(format!("validate-errors: {} lignes", e.lines().count()));
        }
    }

    Ok(SessionSummary {
        ts,
        day,
        week,
        objectives: objectives.into_iter().collect(),
        files_touched: files.into_iter().take(25).collect(),
        patches_created: patches_created.into_iter().collect(),
        patches_applied: patches_applied.into_iter().collect(),
        validations,
        errors,
        decisions,
        critical_modules: critical.into_iter().collect(),
        open_tasks,
        recommendations,
    })
}

fn summary_to_text(s: &SessionSummary) -> String {
    [
        format!("ts={}", s.ts),
        format!("day={}", s.day),
        format!("week={}", s.week),
        format!("objectives={}", s.objectives.join(" | ")),
        format!("files_touched={}", s.files_touched.join(" | ")),
        format!("patches_created={}", s.patches_created.join(" | ")),
        format!("patches_applied={}", s.patches_applied.join(" | ")),
        format!("validations={}", s.validations),
        format!("errors={}", s.errors.join(" | ")),
        format!("decisions={}", s.decisions.join(" | ")),
        format!("critical_modules={}", s.critical_modules.join(" | ")),
        format!("open_tasks={}", s.open_tasks.join(" | ")),
        format!("recommendations={}", s.recommendations.join(" | ")),
    ]
    .join("\n")
        + "\n"
}

pub fn persist_summary(root: &Path, summary: &SessionSummary) -> Result<(), String> {
    let _guard = lock::acquire(root, "memory-summary")?;
    fs::create_dir_all(mem_root(root).join("daily")).map_err(|e| e.to_string())?;
    fs::create_dir_all(mem_root(root).join("weekly")).map_err(|e| e.to_string())?;
    fssec::set_secure_dir(&mem_root(root))?;
    fssec::set_secure_dir(&mem_root(root).join("daily"))?;
    fssec::set_secure_dir(&mem_root(root).join("weekly"))?;

    let text = summary_to_text(summary);
    let daily = mem_root(root)
        .join("daily")
        .join(format!("{}.txt", summary.day));
    let weekly = mem_root(root)
        .join("weekly")
        .join(format!("{}.txt", summary.week));
    let mut dcur = fs::read_to_string(&daily).unwrap_or_default();
    dcur.push_str("---\n");
    dcur.push_str(&text);
    fs::write(&daily, dcur).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&daily)?;

    let mut wcur = fs::read_to_string(&weekly).unwrap_or_default();
    wcur.push_str("---\n");
    wcur.push_str(&text);
    fs::write(&weekly, wcur).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&weekly)?;

    let project_compact = build_project_compact(root)?;
    let compact_path = mem_root(root).join("project_compact.txt");
    fs::write(&compact_path, project_compact).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&compact_path)?;

    let sessions = mem_root(root).join("sessions.log");
    let mut scur = fs::read_to_string(&sessions).unwrap_or_default();
    scur.push_str(&format!(
        "{}\t{}\t{}\n",
        summary.ts, summary.day, summary.week
    ));
    if scur.lines().count() > 300 {
        scur = scur
            .lines()
            .rev()
            .take(200)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        scur.push('\n');
    }
    fs::write(&sessions, scur).map_err(|e| e.to_string())?;
    fssec::set_secure_file(&sessions)
}

fn build_project_compact(root: &Path) -> Result<String, String> {
    let loops = load_open_loops(root);
    let lessons = fs::read_to_string(mem_root(root).join("lessons.tsv")).unwrap_or_default();
    let patch_history =
        fs::read_to_string(mem_root(root).join("patch_history.tsv")).unwrap_or_default();
    let recent_patches = patch_history
        .lines()
        .rev()
        .take(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for (_, cat, status, _) in &loops {
        if status == "open" {
            *by_category.entry(cat.clone()).or_insert(0) += 1;
        }
    }

    let recent_lessons = lessons
        .lines()
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let mut txt = String::new();
    txt.push_str("project_memory_compact=1\n");
    txt.push_str(&format!(
        "open_loops_total={}\n",
        loops.iter().filter(|x| x.2 == "open").count()
    ));
    txt.push_str(&format!(
        "open_loops_by_category={}\n",
        by_category
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(" | ")
    ));
    txt.push_str(&format!("recent_patches={}\n", recent_patches.join(" | ")));
    txt.push_str(&format!("recent_lessons={}\n", recent_lessons.join(" | ")));
    Ok(txt)
}

pub fn render_recent(root: &Path) -> String {
    let mut out = String::new();
    let sessions = fs::read_to_string(mem_root(root).join("sessions.log")).unwrap_or_default();
    let last_session = sessions.lines().last().unwrap_or("none");
    out.push_str(&format!("last_session={last_session}\n"));

    let open = load_open_loops(root)
        .into_iter()
        .filter(|(_, _, status, _)| status == "open")
        .take(8)
        .map(|(id, cat, _, d)| format!("{id} [{cat}] {d}"))
        .collect::<Vec<_>>();
    out.push_str(&format!("open_loops={}\n", open.join(" | ")));

    let lessons = fs::read_to_string(mem_root(root).join("lessons.tsv")).unwrap_or_default();
    let recent_lessons = lessons
        .lines()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    out.push_str(&format!("lessons={}\n", recent_lessons.join(" | ")));

    let patch_hist =
        fs::read_to_string(mem_root(root).join("patch_history.tsv")).unwrap_or_default();
    let recent_patch = patch_hist
        .lines()
        .rev()
        .take(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    out.push_str(&format!("patches={}\n", recent_patch.join(" | ")));

    let validate_err =
        fs::read_to_string(mem_root(root).join("cargo_errors_latest.log")).unwrap_or_default();
    let err_lines = validate_err.lines().take(5).collect::<Vec<_>>();
    out.push_str(&format!("last_validate_errors={}\n", err_lines.join(" | ")));

    let mut touched = BTreeSet::new();
    let daily_dir = mem_root(root).join("daily");
    if let Ok(entries) = fs::read_dir(daily_dir) {
        let mut files = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect::<Vec<_>>();
        files.sort();
        if let Some(last) = files.last() {
            let txt = fs::read_to_string(last).unwrap_or_default();
            for l in txt.lines() {
                if let Some(v) = l.strip_prefix("files_touched=") {
                    for p in v.split(" | ") {
                        if !p.trim().is_empty() {
                            touched.insert(p.trim().to_string());
                        }
                    }
                }
            }
        }
    }
    out.push_str(&format!(
        "recent_modules={}\n",
        touched.into_iter().take(12).collect::<Vec<_>>().join(" | ")
    ));
    out
}

pub fn render_view(root: &Path, view: &str) -> String {
    match view {
        "show" => fs::read_to_string(mem_root(root).join("project_compact.txt"))
            .unwrap_or_else(|_| "memory unavailable".to_string()),
        "recent" => render_recent(root),
        "open-loops" => fs::read_to_string(mem_root(root).join("open_loops.tsv"))
            .unwrap_or_else(|_| "open loops empty".to_string()),
        "lessons" => fs::read_to_string(mem_root(root).join("lessons.tsv"))
            .unwrap_or_else(|_| "lessons empty".to_string()),
        "daily" => {
            let ts = now_ts().unwrap_or(0);
            let (day, _) = day_week_from_epoch(ts);
            fs::read_to_string(mem_root(root).join("daily").join(format!("{day}.txt")))
                .unwrap_or_else(|_| format!("daily summary missing for {day}"))
        }
        "weekly" => {
            let ts = now_ts().unwrap_or(0);
            let (_, week) = day_week_from_epoch(ts);
            fs::read_to_string(mem_root(root).join("weekly").join(format!("{week}.txt")))
                .unwrap_or_else(|_| format!("weekly summary missing for {week}"))
        }
        _ => "unknown memory view".to_string(),
    }
}

pub fn recent_modules(root: &Path, n: usize) -> Vec<String> {
    let txt = render_recent(root);
    txt.lines()
        .find_map(|l| l.strip_prefix("recent_modules="))
        .map(|v| {
            v.split(" | ")
                .filter(|x| !x.trim().is_empty())
                .map(ToString::to_string)
                .take(n)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_conversion_stable() {
        let (day, week) = day_week_from_epoch(1_704_067_200); // 2024-01-01
        assert_eq!(day, "2024-01-01");
        assert!(week.starts_with("2024-W"));
    }

    #[test]
    fn open_loop_roundtrip() {
        let root = std::env::temp_dir().join(format!(
            "pata-long-memory-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let id = add_open_loop(&root, "bug", "corriger panic scanner").unwrap();
        let txt = render_view(&root, "open-loops");
        assert!(txt.contains("corriger panic scanner"));
        resolve_open_loop(&root, &id).unwrap();
        let txt2 = render_view(&root, "open-loops");
        assert!(txt2.contains("closed"));
    }
}
