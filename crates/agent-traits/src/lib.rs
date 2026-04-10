use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub run_id: Uuid,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            run_id: Uuid::new_v4(),
            user_id: None,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub timeout_ms: u64,
    pub required_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    PlanCreated { steps: Vec<String> },
    ToolCalled { tool: String },
    ToolReturned { tool: String },
    ValidationPassed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub answer: String,
    pub confidence: f32,
    pub structured_output: Option<Value>,
    pub events: Vec<AgentEvent>,
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("provider error: {0}")]
    Provider(String),
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        context: &ExecutionContext,
    ) -> Result<String>;
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;

    async fn run(&self, input: Value, context: &ExecutionContext) -> Result<ToolOutput>;
}

pub trait Persona: Send + Sync {
    fn name(&self) -> &'static str;
    fn system_prompt(&self) -> String;
    fn allowed_tools(&self) -> Vec<String>;
    fn validate(&self, draft: &str) -> std::result::Result<(), AgentError>;
}

impl<T> Persona for Box<T>
where
    T: Persona + ?Sized,
{
    fn name(&self) -> &'static str {
        (**self).name()
    }

    fn system_prompt(&self) -> String {
        (**self).system_prompt()
    }

    fn allowed_tools(&self) -> Vec<String> {
        (**self).allowed_tools()
    }

    fn validate(&self, draft: &str) -> std::result::Result<(), AgentError> {
        (**self).validate(draft)
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn run(&self, goal: &str, context: ExecutionContext) -> Result<AgentResult>;
}
