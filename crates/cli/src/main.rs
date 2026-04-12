use std::{fs, path::PathBuf};

use agent_core::{OrchestratedAgent, ToolRegistry};
use agent_traits::{Agent, ExecutionContext, ModelProvider, Persona, Tool, ToolOutput, ToolSpec};
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Parser;
use persona_registry::PersonaRegistry;
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

#[derive(Debug, Clone)]
struct DeterministicModelProvider;

#[async_trait]
impl ModelProvider for DeterministicModelProvider {
    fn name(&self) -> &'static str {
        "deterministic-model"
    }

    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _context: &ExecutionContext,
    ) -> Result<String> {
        let is_teacher = system_prompt.contains("pedagogical AI teacher");
        let is_personal = system_prompt.contains("structured personal productivity assistant");
        let is_smb = system_prompt.contains("SMB operations copilot");

        if user_prompt.contains("Analyze") {
            return if is_teacher {
                Ok("LEARNING_OBJECTIVE: Identify what the learner should understand from the topic."
                    .to_string())
            } else if is_personal {
                Ok(
                    "CONTEXT_SUMMARY: Identify personal context, priorities and constraints."
                        .to_string(),
                )
            } else if is_smb {
                Ok(
                    "BUSINESS_CONTEXT: Identify business operating context and key constraints."
                        .to_string(),
                )
            } else {
                Ok("ANALYSIS: Investigate error scope and constraints".to_string())
            };
        }
        if user_prompt.contains("Hypothesis") {
            return if is_teacher {
                Ok("LEVEL_ADAPTATION: beginner level, use plain language and one concept at a time."
                    .to_string())
            } else if is_personal {
                Ok(
                    "PRIMARY_OBJECTIVE: Define one realistic priority objective for this period."
                        .to_string(),
                )
            } else if is_smb {
                Ok(
                    "OPERATIONAL_OBJECTIVE: Define a concrete short-term business objective."
                        .to_string(),
                )
            } else {
                Ok("HYPOTHESIS: Root cause likely ownership mismatch".to_string())
            };
        }
        if user_prompt.contains("ActionOrTest") {
            return if is_teacher {
                Ok("EXPLANATION: Explain core concept with one concrete example before adding nuance."
                    .to_string())
            } else if is_personal {
                Ok("ACTION_STRUCTURE: Break the objective into concrete, feasible and time-bound steps."
                    .to_string())
            } else if is_smb {
                Ok("ACTION_BACKLOG: Build a prioritized list of operational actions.".to_string())
            } else {
                Ok(
                    "ACTION_PLAN: Apply scoped borrow refactor. VALIDATION: cargo check"
                        .to_string(),
                )
            };
        }
        if user_prompt.contains("Validation") {
            return if is_teacher {
                Ok("LEARNING_OBJECTIVE: Understand Rust ownership for function arguments.\n\
LEVEL_ADAPTATION: beginner audience, short sentences and minimal jargon.\n\
EXPLANATION: In Rust, each value has one owner; moving a value transfers ownership.\n\
GUIDED_PRACTICE: Compare a function taking String by value versus &str by reference.\n\
UNDERSTANDING_CHECK: Why can you still use a variable after borrowing but not after move?\n\
FINAL_ANSWER: Start with ownership, then borrowing, then lifetimes using one compiled example each."
                    .to_string())
            } else if is_personal {
                Ok("CONTEXT_SUMMARY: You want better weekly organization with limited time and energy.\n\
PRIMARY_OBJECTIVE: Stabilize personal planning and focus on essential priorities.\n\
ACTION_STRUCTURE: Plan weekly priorities on Sunday, set daily top-3 tasks, and review each evening.\n\
RISK_CHECK: Main constraint is limited evening energy; keep tasks small and review trade-offs.\n\
NEXT_STEP: Write tomorrow's top-3 tasks now and block 20 minutes for planning.\n\
FINAL_ANSWER: Use a lightweight routine, adjust weekly, and keep the plan realistic with constraints."
                    .to_string())
            } else if is_smb {
                Ok("BUSINESS_CONTEXT: Small business with limited staff capacity and marketing budget.\n\
OPERATIONAL_OBJECTIVE: Increase repeat customer visits in the next 30 days.\n\
ACTION_BACKLOG: 1) Daily follow-up messages, 2) Weekly offer bundle, 3) Inventory focus on top sellers.\n\
DECISION_SUPPORT: Assumption: retention actions will outperform broad paid ads under current budget.\n\
FOLLOW_UP_METRICS: Repeat visits/week, offer conversion rate, average basket size.\n\
FINAL_ANSWER: Execute low-cost retention actions first, track weekly metrics, and iterate quickly."
                    .to_string())
            } else {
                Ok("ANALYSIS: Investigate error scope and constraints.\n\
HYPOTHESIS: Root cause likely ownership mismatch.\n\
ACTION_PLAN: Apply scoped borrow refactor and run cargo check.\n\
VALIDATION: Verify no compiler errors and tests remain green.\n\
DURABLE_RULES_CHECK: No contradiction with project rules.\n\
FINAL_ANSWER: Apply scoped borrow refactor, re-run cargo test, then review diff."
                    .to_string())
            };
        }

        Ok("FINAL_ANSWER: deterministic fallback response.".to_string())
    }
}

#[derive(Debug, Clone)]
struct NoopTool {
    name: String,
}

#[async_trait]
impl Tool for NoopTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name.clone(),
            description: "No-op placeholder tool for local runtime".to_string(),
            input_schema: json!({"type": "object"}),
            output_schema: json!({"type": "object"}),
            timeout_ms: 1_000,
            required_permissions: Vec::new(),
        }
    }

    async fn run(&self, _input: Value, _context: &ExecutionContext) -> Result<ToolOutput> {
        Ok(ToolOutput {
            value: json!({"ok": true}),
        })
    }
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

fn build_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    for name in [
        "filesystem.read",
        "filesystem.write",
        "cargo.check",
        "cargo.test",
        "git.diff",
    ] {
        registry.register(NoopTool {
            name: name.to_string(),
        });
    }
    registry
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
    if config.model.mode != "deterministic" {
        anyhow::bail!(
            "unsupported model mode '{}': only deterministic is available in V2 runtime",
            config.model.mode
        );
    }

    let persona_name = args
        .persona
        .as_deref()
        .or(config.persona.as_ref().map(|p| p.name.as_str()))
        .unwrap_or("developer");

    let persona = PersonaRegistry::create(persona_name).context("invalid persona selection")?;
    let persona_name = persona.name().to_string();

    let agent = OrchestratedAgent::new(persona, DeterministicModelProvider, build_registry())
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
