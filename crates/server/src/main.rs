use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};

use agent_core::{OrchestratedAgent, ToolRegistry};
use agent_traits::{Agent, ExecutionContext, ModelProvider, Tool, ToolOutput, ToolSpec};
use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use persona_registry::PersonaRegistry;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct AppConfig {
    model: ModelConfig,
    server: ServerConfig,
    persona: Option<PersonaConfig>,
}

#[derive(Debug, Deserialize)]
struct ModelConfig {
    mode: String,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct PersonaConfig {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RunRequest {
    goal: String,
}

#[derive(Debug, Serialize)]
struct RunResponse {
    status: &'static str,
    persona: String,
    data: RunData,
}

#[derive(Debug, Serialize)]
struct RunData {
    answer: String,
    confidence: f32,
    structured_output: Option<Value>,
}

#[derive(Debug, Serialize)]
struct PersonasResponse {
    status: &'static str,
    personas: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: &'static str,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                status: "error",
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
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

fn validate_goal(goal: &str) -> Result<(), ApiError> {
    if goal.trim().is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "INVALID_GOAL",
            message: "goal must not be empty".to_string(),
        });
    }
    if goal.len() > 2_000 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            code: "GOAL_TOO_LONG",
            message: "goal is too long (max 2000 chars)".to_string(),
        });
    }
    Ok(())
}

fn load_config(path: &PathBuf) -> Result<AppConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("unable to read config at {}", path.display()))?;
    let config: AppConfig = toml::from_str(&raw).context("invalid TOML config")?;

    if config.server.host.trim().is_empty() {
        anyhow::bail!("config.server.host must not be empty");
    }
    if config.server.port == 0 {
        anyhow::bail!("config.server.port must be > 0");
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

#[derive(Debug)]
struct AppState {
    persona_name: String,
    agent: Arc<OrchestratedAgent<Box<dyn agent_traits::Persona>, DeterministicModelProvider>>,
}

async fn health_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn personas_handler() -> Json<PersonasResponse> {
    let personas = PersonaRegistry::list()
        .into_iter()
        .map(|meta| {
            json!({
                "name": meta.name,
                "description": meta.description,
                "objectives": meta.objectives,
                "use_cases": meta.use_cases,
                "guardrails": meta.guardrails,
            })
        })
        .collect::<Vec<_>>();

    Json(PersonasResponse {
        status: "ok",
        personas,
    })
}

async fn run_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RunRequest>,
) -> Result<Json<RunResponse>, ApiError> {
    validate_goal(&payload.goal)?;

    let result = state
        .agent
        .run(&payload.goal, ExecutionContext::default())
        .await
        .map_err(|err| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "RUN_FAILED",
            message: err.to_string(),
        })?;

    Ok(Json(RunResponse {
        status: "ok",
        persona: state.persona_name.clone(),
        data: RunData {
            answer: result.answer,
            confidence: result.confidence,
            structured_output: result.structured_output,
        },
    }))
}

fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/personas", get(personas_handler))
        .route("/run", post(run_handler))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = PathBuf::from("config/app.toml");
    let config = load_config(&config_path)?;

    if config.model.mode != "deterministic" {
        anyhow::bail!(
            "unsupported model mode '{}': only deterministic is available in V2 runtime",
            config.model.mode
        );
    }

    let persona_name = config
        .persona
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("developer");

    let persona = PersonaRegistry::create(persona_name).context("invalid persona selection")?;

    let agent = Arc::new(OrchestratedAgent::new(
        persona,
        DeterministicModelProvider,
        build_registry(),
    )?);

    let state = Arc::new(AppState {
        persona_name: persona_name.to_string(),
        agent,
    });
    let app = build_app(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("invalid host/port in config")?;

    println!("pata-server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn make_state(persona_name: &str) -> Arc<AppState> {
        let persona = PersonaRegistry::create(persona_name).expect("valid persona");
        Arc::new(AppState {
            persona_name: persona_name.to_string(),
            agent: Arc::new(
                OrchestratedAgent::new(persona, DeterministicModelProvider, build_registry())
                    .expect("agent init"),
            ),
        })
    }

    #[tokio::test]
    async fn health_endpoint_is_ok() {
        let response = build_app(make_state("developer"))
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn personas_endpoint_is_ok() {
        let response = build_app(make_state("developer"))
            .oneshot(
                Request::builder()
                    .uri("/personas")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn run_endpoint_rejects_empty_goal() {
        let response = build_app(make_state("developer"))
            .oneshot(
                Request::builder()
                    .uri("/run")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"goal":"   "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn run_endpoint_supports_personal_persona() {
        let response = build_app(make_state("personal"))
            .oneshot(
                Request::builder()
                    .uri("/run")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"goal":"Organize my week"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn run_endpoint_supports_smb_persona() {
        let response = build_app(make_state("smb"))
            .oneshot(
                Request::builder()
                    .uri("/run")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"goal":"Improve customer retention"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
