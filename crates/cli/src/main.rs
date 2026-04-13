use std::{fs, path::PathBuf};

use agent_core::OrchestratedAgent;
use agent_traits::{Agent, ExecutionContext, Persona};
use anyhow::{Context, Result};
use clap::Parser;
use persona_registry::PersonaRegistry;
use runtime_support::{build_tool_registry, ensure_deterministic_mode, DeterministicModelProvider};
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

fn validate_goal(goal: &str) -> Result<()> {
    if goal.trim().is_empty() {
        anyhow::bail!("goal must not be empty")
    }
    if goal.len() > 2_000 {
        anyhow::bail!("goal is too long (max 2000 chars)")
    }
    Ok(())
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    if args.list_personas {
        return print_personas();
    }

    let goal = args
        .goal
        .as_deref()
        .context("missing --goal (or use --list-personas)")?;
    validate_goal(goal).context("invalid --goal input")?;

    let config = load_config(&args.config)?;
    ensure_deterministic_mode(&config.model.mode)?;

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
