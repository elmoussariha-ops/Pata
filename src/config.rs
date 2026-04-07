use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspace: PathBuf,
    pub model_endpoint: String,
    pub model_name: String,
    pub optimization_interval_sec: u64,
    pub allow_auto_apply: bool,
    pub protected_paths: Vec<PathBuf>,
    pub max_loaded_files: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        let ram_gb = detect_ram_gb().unwrap_or(16);
        let model_name = select_model_for_mac_m4(ram_gb);
        Self {
            workspace: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            model_endpoint: "http://127.0.0.1:11434/api/generate".to_string(),
            model_name,
            optimization_interval_sec: 600,
            allow_auto_apply: false,
            protected_paths: vec![PathBuf::from("src/core"), PathBuf::from("src/engine")],
            max_loaded_files: 8,
        }
    }
}

impl AppConfig {
    pub fn path() -> PathBuf {
        PathBuf::from(".pata/config.json")
    }

    pub fn load_or_create() -> Result<Self> {
        let path = Self::path();
        if path.exists() {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("unable to read {}", path.display()))?;
            Ok(serde_json::from_str(&text).context("invalid config json")?)
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let cfg = Self::default();
            fs::write(&path, serde_json::to_string_pretty(&cfg)?)?;
            Ok(cfg)
        }
    }
}

pub fn detect_ram_gb() -> Option<u64> {
    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes: u64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()?;
    Some(bytes / 1024 / 1024 / 1024)
}

pub fn select_model_for_mac_m4(ram_gb: u64) -> String {
    if ram_gb >= 24 {
        "qwen2.5-coder:14b-instruct-q4_K_M".to_string()
    } else if ram_gb >= 16 {
        "qwen2.5-coder:7b-instruct-q4_K_M".to_string()
    } else {
        "deepseek-coder:6.7b-instruct-q4_K_M".to_string()
    }
}
