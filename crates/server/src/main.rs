use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};

use agent_core::OrchestratedAgent;
use agent_traits::{Agent, ExecutionContext};
use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use persona_registry::PersonaRegistry;
use runtime_support::{
    build_tool_registry, ensure_deterministic_mode, validate_goal, DeterministicModelProvider,
};
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

fn validate_goal_request(goal: &str) -> Result<(), ApiError> {
    validate_goal(goal).map_err(|err| {
        let message = err.to_string();
        let (status, code) = if message.contains("too long") {
            (StatusCode::BAD_REQUEST, "GOAL_TOO_LONG")
        } else {
            (StatusCode::BAD_REQUEST, "INVALID_GOAL")
        };

        ApiError {
            status,
            code,
            message,
        }
    })
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
    validate_goal_request(&payload.goal)?;

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

    ensure_deterministic_mode(&config.model.mode)?;

    let persona_name = config
        .persona
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("developer");

    let persona = PersonaRegistry::create(persona_name).context("invalid persona selection")?;
    let agent = Arc::new(OrchestratedAgent::new(
        persona,
        DeterministicModelProvider,
        build_tool_registry(),
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
                OrchestratedAgent::new(persona, DeterministicModelProvider, build_tool_registry())
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
