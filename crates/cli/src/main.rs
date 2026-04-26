use std::{
    fs,
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use agent_core::OrchestratedAgent;
use agent_traits::{Agent, ExecutionContext, Persona};
use anyhow::{Context, Result};
use clap::Parser;
use persona_registry::PersonaRegistry;
use runtime_support::{
    build_tool_registry, ensure_deterministic_mode, validate_goal, DeterministicModelProvider,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Parser)]
#[command(name = "pata-cli", about = "Run Pata personas from CLI")]
struct Cli {
    #[arg(long, help = "Goal to execute")]
    goal: Option<String>,
    #[arg(
        long,
        default_value = "config/app.toml",
        help = "Path to runtime TOML config"
    )]
    config: PathBuf,
    #[arg(
        long,
        help = "Optional persona override (developer|teacher|personal|smb). If omitted, uses config."
    )]
    persona: Option<String>,
    #[arg(long, help = "List available personas and exit")]
    list_personas: bool,
    #[arg(
        long,
        help = "Run a versioned evaluation suite from a JSON file instead of a single goal"
    )]
    eval_suite: Option<PathBuf>,
    #[arg(
        long,
        help = "Optional path to write evaluation results as JSON (used with --eval-suite)"
    )]
    eval_output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    model: ModelConfig,
    persona: Option<PersonaConfig>,
}

#[derive(Debug, Deserialize)]
struct ModelConfig {
    mode: String,
}

#[derive(Debug, Deserialize)]
struct PersonaConfig {
    name: String,
}

#[derive(Debug, Serialize)]
struct RuntimeResponse {
    status: &'static str,
    persona: String,
    goal: String,
    answer: String,
    confidence: f32,
    structured_output: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct EvaluationSuite {
    suite_id: String,
    suite_version: String,
    cases: Vec<EvalCaseSpec>,
}

#[derive(Debug, Deserialize)]
struct EvalCaseSpec {
    case_id: String,
    persona: String,
    goal: String,
    require_accept_verification: bool,
    minimum_confidence: f32,
    required_sections: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EvalCaseResult {
    case_id: String,
    persona: String,
    goal: String,
    passed: bool,
    verification_status: String,
    confidence: f32,
    answer_non_empty: bool,
    sections_present: Vec<String>,
    sections_missing: Vec<String>,
    duration_ms: u128,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct EvalSuiteSummary {
    total_cases: usize,
    passed_cases: usize,
    failed_cases: usize,
    pass_rate: f32,
}

#[derive(Debug, Serialize)]
struct EvalSuiteResult {
    suite_id: String,
    suite_version: String,
    generated_at_epoch_sec: u64,
    summary: EvalSuiteSummary,
    cases: Vec<EvalCaseResult>,
}

fn load_config(path: &PathBuf) -> Result<AppConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("unable to read config at {}", path.display()))?;
    let config: AppConfig = toml::from_str(&raw).context("invalid TOML config")?;

    if config.model.mode.trim().is_empty() {
        anyhow::bail!("config.model.mode must not be empty");
    }

    if let Some(persona) = &config.persona {
        if !PersonaRegistry::exists(persona.name.trim()) {
            anyhow::bail!(
                "invalid config.persona.name '{}': unknown persona",
                persona.name
            );
        }
    }

    Ok(config)
}

fn print_personas() -> Result<()> {
    let payload = json!({
        "status": "ok",
        "personas": PersonaRegistry::list().into_iter().map(|meta| json!({
            "name": meta.name,
            "description": meta.description,
            "objectives": meta.objectives,
            "use_cases": meta.use_cases,
            "guardrails": meta.guardrails,
        })).collect::<Vec<_>>()
    });

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn load_eval_suite(path: &PathBuf) -> Result<EvaluationSuite> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("unable to read eval suite at {}", path.display()))?;
    let suite: EvaluationSuite =
        serde_json::from_str(&raw).context("invalid eval suite JSON format")?;

    if suite.suite_id.trim().is_empty() || suite.suite_version.trim().is_empty() {
        anyhow::bail!("eval suite must include non-empty suite_id and suite_version");
    }

    if suite.cases.is_empty() {
        anyhow::bail!("eval suite must include at least one case");
    }

    let mut seen_case_ids = std::collections::HashSet::new();
    for case in &suite.cases {
        if !seen_case_ids.insert(case.case_id.clone()) {
            anyhow::bail!("duplicate case_id found in eval suite: '{}'", case.case_id);
        }
        validate_goal(&case.goal)
            .with_context(|| format!("invalid goal in case {}", case.case_id))?;
        if !PersonaRegistry::exists(case.persona.trim()) {
            anyhow::bail!(
                "invalid persona '{}' in case '{}': unknown persona",
                case.persona,
                case.case_id
            );
        }
        if !(0.0..=1.0).contains(&case.minimum_confidence) {
            anyhow::bail!(
                "invalid minimum_confidence in case '{}': expected value in [0, 1]",
                case.case_id
            );
        }
    }

    Ok(suite)
}

async fn run_eval_suite(config: &AppConfig, suite: EvaluationSuite) -> Result<EvalSuiteResult> {
    let mut results = Vec::new();

    for case in suite.cases {
        let started = Instant::now();
        let case_result = async {
            let persona = PersonaRegistry::create(&case.persona)
                .with_context(|| format!("unable to create persona '{}'", case.persona))?;
            let agent =
                OrchestratedAgent::new(persona, DeterministicModelProvider, build_tool_registry())
                    .context("unable to initialize orchestrated agent")?;
            let run_result = agent
                .run(&case.goal, ExecutionContext::default())
                .await
                .context("agent execution failed")?;

            let structured = run_result.structured_output.unwrap_or_default();
            let verification_status = structured["verification_status"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string();
            let raw_response = structured["raw_response"].as_str().unwrap_or("");
            let sections_present = case
                .required_sections
                .iter()
                .filter(|section| raw_response.contains(section.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let sections_missing = case
                .required_sections
                .iter()
                .filter(|section| !raw_response.contains(section.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let answer_non_empty = !run_result.answer.trim().is_empty();
            let verification_ok =
                !case.require_accept_verification || verification_status == "Accept";
            let confidence_ok = run_result.confidence >= case.minimum_confidence;
            let passed =
                answer_non_empty && verification_ok && confidence_ok && sections_missing.is_empty();

            Ok::<EvalCaseResult, anyhow::Error>(EvalCaseResult {
                case_id: case.case_id.clone(),
                persona: case.persona.clone(),
                goal: case.goal.clone(),
                passed,
                verification_status,
                confidence: run_result.confidence,
                answer_non_empty,
                sections_present,
                sections_missing,
                duration_ms: started.elapsed().as_millis(),
                error: None,
            })
        }
        .await;

        match case_result {
            Ok(result) => results.push(result),
            Err(err) => results.push(EvalCaseResult {
                case_id: case.case_id,
                persona: case.persona,
                goal: case.goal,
                passed: false,
                verification_status: "ExecutionError".to_string(),
                confidence: 0.0,
                answer_non_empty: false,
                sections_present: Vec::new(),
                sections_missing: case.required_sections,
                duration_ms: started.elapsed().as_millis(),
                error: Some(err.to_string()),
            }),
        }
    }

    let total_cases = results.len();
    let passed_cases = results.iter().filter(|case| case.passed).count();
    let failed_cases = total_cases.saturating_sub(passed_cases);
    let summary = EvalSuiteSummary {
        total_cases,
        passed_cases,
        failed_cases,
        pass_rate: if total_cases == 0 {
            0.0
        } else {
            passed_cases as f32 / total_cases as f32
        },
    };

    let generated_at_epoch_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let report = EvalSuiteResult {
        suite_id: suite.suite_id,
        suite_version: suite.suite_version,
        generated_at_epoch_sec,
        summary,
        cases: results,
    };

    if !config.model.mode.eq("deterministic") {
        anyhow::bail!("eval suite only supports deterministic mode");
    }

    Ok(report)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    if args.list_personas {
        return print_personas();
    }

    let config = load_config(&args.config)?;
    ensure_deterministic_mode(&config.model.mode)?;

    if let Some(suite_path) = &args.eval_suite {
        let suite = load_eval_suite(suite_path)?;
        let report = run_eval_suite(&config, suite).await?;

        let report_json = serde_json::to_string_pretty(&report)?;
        println!("{report_json}");

        if let Some(output_path) = &args.eval_output {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "unable to create eval output directory {}",
                        parent.display()
                    )
                })?;
            }
            fs::write(output_path, &report_json).with_context(|| {
                format!("unable to write eval output at {}", output_path.display())
            })?;
        }

        if report.summary.failed_cases > 0 {
            anyhow::bail!(
                "evaluation suite failed: {} / {} cases passed",
                report.summary.passed_cases,
                report.summary.total_cases
            );
        }
        return Ok(());
    }

    let goal = args
        .goal
        .as_deref()
        .context("missing --goal (or use --list-personas or --eval-suite)")?;
    validate_goal(goal).context("invalid --goal input")?;

    let persona_name = args
        .persona
        .as_deref()
        .or(config.persona.as_ref().map(|p| p.name.as_str()))
        .unwrap_or("developer");

    let persona = PersonaRegistry::create(persona_name).context("invalid persona selection")?;
    let persona_name = persona.name().to_string();

    let agent = OrchestratedAgent::new(persona, DeterministicModelProvider, build_tool_registry())
        .context("unable to initialize orchestrated agent")?;

    let result = agent
        .run(goal, ExecutionContext::default())
        .await
        .context("agent execution failed")?;

    let payload = RuntimeResponse {
        status: "ok",
        persona: persona_name,
        goal: goal.to_string(),
        answer: result.answer,
        confidence: result.confidence,
        structured_output: result.structured_output,
    };

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_traits::ModelProvider;

    #[test]
    fn reject_empty_goal() {
        assert!(validate_goal("   ").is_err());
    }

    #[test]
    fn reject_unknown_persona_in_config() {
        let path = PathBuf::from("/tmp/pata_cli_test.toml");
        fs::write(
            &path,
            "[model]\nmode = \"deterministic\"\n[persona]\nname = \"unknown\"\n",
        )
        .expect("write temp config");
        let result = load_config(&path);
        let _ = fs::remove_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn print_personas_lists_teacher_and_developer() {
        let list = PersonaRegistry::list();
        assert!(list.iter().any(|p| p.name == "developer"));
        assert!(list.iter().any(|p| p.name == "teacher"));
        assert!(list.iter().any(|p| p.name == "personal"));
        assert!(list.iter().any(|p| p.name == "smb"));
    }

    #[test]
    fn reject_eval_suite_with_duplicate_case_ids() {
        let path = PathBuf::from("/tmp/pata_cli_eval_suite_test.json");
        fs::write(
            &path,
            r#"{
  "suite_id": "suite",
  "suite_version": "1",
  "cases": [
    {
      "case_id": "dup",
      "persona": "developer",
      "goal": "Fix compile error",
      "require_accept_verification": false,
      "minimum_confidence": 0.1,
      "required_sections": []
    },
    {
      "case_id": "dup",
      "persona": "teacher",
      "goal": "Explain ownership",
      "require_accept_verification": false,
      "minimum_confidence": 0.1,
      "required_sections": []
    }
  ]
}"#,
        )
        .expect("write temp suite");

        let result = load_eval_suite(&path);
        let _ = fs::remove_file(&path);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn smb_provider_output_matches_persona_contract() {
        let persona = PersonaRegistry::create("smb").expect("smb persona");
        let provider = DeterministicModelProvider;

        let draft = provider
            .complete(
                &persona.system_prompt(),
                "Step: Validation",
                &ExecutionContext::default(),
            )
            .await
            .expect("provider should return a draft");

        assert!(persona.validate(&draft).is_ok());
    }
}
