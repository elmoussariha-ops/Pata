use crate::config::{AppConfig, ModelBackend};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn suggest_patch(cfg: &AppConfig, prompt: &str) -> Result<String> {
    match &cfg.model_backend {
        ModelBackend::Disabled => Ok("Model backend disabled in config.".to_string()),
        ModelBackend::Ollama { endpoint } => {
            let client = reqwest::Client::new();
            let body = OllamaRequest {
                model: &cfg.model_name,
                prompt,
                stream: false,
            };
            let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
            let resp = client.post(url).json(&body).send().await?;
            if !resp.status().is_success() {
                return Err(anyhow!(
                    "ollama request failed with status {}",
                    resp.status()
                ));
            }
            let data: OllamaResponse = resp.json().await?;
            Ok(data.response)
        }
    }
}
