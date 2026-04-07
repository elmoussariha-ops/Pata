use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspace: PathBuf,
    pub model_backend: ModelBackend,
    pub model_name: String,
    pub optimization_interval_seconds: u64,
    pub allow_auto_apply: bool,
    pub protected_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum ModelBackend {
    Ollama { endpoint: String },
    Disabled,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workspace: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            model_backend: ModelBackend::Ollama {
                endpoint: "http://127.0.0.1:11434".to_string(),
            },
            model_name: "qwen2.5-coder:7b-instruct-q4_K_M".to_string(),
            optimization_interval_seconds: 600,
            allow_auto_apply: false,
            protected_paths: vec![PathBuf::from("src/core"), PathBuf::from("src/engine")],
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        PathBuf::from(".pata/config.json")
    }

    pub fn load_or_create() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed reading {}", path.display()))?;
            let cfg = serde_json::from_str(&content).context("failed parsing config json")?;
            Ok(cfg)
        } else {
            let cfg = Self::default();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, serde_json::to_string_pretty(&cfg)?)?;
            Ok(cfg)
        }
    }
}
