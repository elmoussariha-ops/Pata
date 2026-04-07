use crate::types::{OllamaDiagnostic, OllamaSettings, OllamaStatus};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn detect_ram_gb_macos() -> Option<u64> {
    let out = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let bytes: u64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    Some(bytes / 1024 / 1024 / 1024)
}

pub fn choose_model(ram_gb: u64) -> &'static str {
    if ram_gb >= 24 {
        "qwen2.5-coder:14b-instruct-q4_K_M"
    } else if ram_gb >= 16 {
        "qwen2.5-coder:7b-instruct-q4_K_M"
    } else {
        "deepseek-coder:6.7b-instruct-q4_K_M"
    }
}

fn low_power_enabled() -> bool {
    matches!(
        std::env::var("PATA_LOW_POWER").ok().as_deref(),
        Some("1") | Some("true") | Some("on")
    )
}

fn verbose_enabled() -> bool {
    matches!(
        std::env::var("PATA_VERBOSE").ok().as_deref(),
        Some("1") | Some("true") | Some("on")
    )
}

fn bounded_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(min, max))
        .unwrap_or(default)
}

fn bounded_u32(name: &str, default: u32, min: u32, max: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .map(|v| v.clamp(min, max))
        .unwrap_or(default)
}

fn bounded_f32(name: &str, default: f32, min: f32, max: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| v.clamp(min, max))
        .unwrap_or(default)
}

pub fn settings_from_env() -> OllamaSettings {
    let ram = detect_ram_gb_macos().unwrap_or(16);
    let default_model = choose_model(ram).to_string();
    let mut model = std::env::var("PATA_MODEL").unwrap_or(default_model);
    if model.len() > 120
        || !model
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '.' || c == '-' || c == '_')
    {
        model = choose_model(ram).to_string();
    }
    let endpoint = std::env::var("PATA_OLLAMA_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
    let mut timeout_sec = bounded_u64("PATA_OLLAMA_TIMEOUT_SEC", 45, 5, 300);
    let mut retries = bounded_u32("PATA_OLLAMA_RETRIES", 1, 1, 5);
    let temperature = bounded_f32("PATA_TEMPERATURE", 0.1, 0.0, 2.0);
    let mut max_tokens = bounded_u32("PATA_MAX_TOKENS", 1200, 64, 8192);

    if low_power_enabled() {
        timeout_sec = timeout_sec.min(20);
        retries = retries.min(1);
        max_tokens = max_tokens.min(400);
    }

    OllamaSettings {
        endpoint,
        model,
        timeout_sec,
        retries,
        temperature,
        max_tokens,
    }
}

pub fn exceeded_timeout(start: Instant, timeout: Duration) -> bool {
    start.elapsed() > timeout
}

pub fn ask_ollama(settings: &OllamaSettings, prompt: &str) -> Result<String, String> {
    let mut last_err = String::new();
    let timeout = Duration::from_secs(settings.timeout_sec);
    for attempt in 1..=settings.retries.max(1) {
        let prompt_with_config = format!(
            "[temperature={}] [max_tokens={}]\n{}",
            settings.temperature, settings.max_tokens, prompt
        );
        if verbose_enabled() {
            eprintln!(
                "[pata][ollama] attempt {attempt}/{} model={} timeout={}s",
                settings.retries.max(1),
                settings.model,
                settings.timeout_sec
            );
        }
        match ask_once(&settings.model, &prompt_with_config, timeout) {
            Ok(v) => return Ok(v),
            Err(e) => {
                if verbose_enabled() {
                    eprintln!("[pata][ollama] attempt failed: {e}");
                }
                last_err = format!("attempt {attempt}/{}: {e}", settings.retries.max(1));
            }
        }
    }
    Err(last_err)
}

pub fn smoke_generate(settings: &OllamaSettings) -> Result<String, String> {
    ask_ollama(settings, "Return only: OK")
}

fn ask_once(model: &str, prompt: &str, timeout: Duration) -> Result<String, String> {
    let mut child = Command::new("ollama")
        .args(["run", model])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot launch ollama: {e}"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| format!("cannot write prompt: {e}"))?;
    }

    let start = Instant::now();
    loop {
        if exceeded_timeout(start, timeout) {
            let _ = child.kill();
            if verbose_enabled() {
                eprintln!("[pata][ollama] timeout reached");
            }
            return Err("ollama timeout".to_string());
        }
        if let Ok(Some(_)) = child.try_wait() {
            let out = child
                .wait_with_output()
                .map_err(|e| format!("cannot read ollama output: {e}"))?;
            if out.status.success() {
                return Ok(String::from_utf8_lossy(&out.stdout).to_string());
            }
            return Err(String::from_utf8_lossy(&out.stderr).to_string());
        }
        std::thread::sleep(Duration::from_millis(30));
    }
}

pub fn ensure_model_available(settings: &OllamaSettings) -> Result<(), String> {
    let status = ollama_status(settings);
    if !status.reachable {
        return Err(status.message);
    }
    if status
        .installed_models
        .iter()
        .any(|m| m.contains(&settings.model))
    {
        Ok(())
    } else {
        Err(format!(
            "model '{}' not installed. run: ollama pull {}",
            settings.model, settings.model
        ))
    }
}

pub fn diagnose_ollama(settings: &OllamaSettings) -> OllamaDiagnostic {
    let version = Command::new("ollama").arg("--version").output();
    match version {
        Err(_) => {
            return OllamaDiagnostic {
                state: "binary-missing".to_string(),
                message: "ollama binary not found in PATH".to_string(),
                hint: "install with: brew install ollama".to_string(),
            }
        }
        Ok(v) if !v.status.success() => {
            return OllamaDiagnostic {
                state: "binary-error".to_string(),
                message: String::from_utf8_lossy(&v.stderr).to_string(),
                hint: "reinstall or check shell PATH".to_string(),
            }
        }
        Ok(_) => {}
    }

    let status = ollama_status(settings);
    if !status.reachable {
        return OllamaDiagnostic {
            state: "daemon-unreachable".to_string(),
            message: status.message,
            hint: "start daemon with: ollama serve".to_string(),
        };
    }

    if !status
        .installed_models
        .iter()
        .any(|m| m.contains(&settings.model))
    {
        return OllamaDiagnostic {
            state: "model-missing".to_string(),
            message: format!("model '{}' not listed", settings.model),
            hint: format!("install with: ollama pull {}", settings.model),
        };
    }

    OllamaDiagnostic {
        state: "ok".to_string(),
        message: format!("ready model={}", settings.model),
        hint: "ready".to_string(),
    }
}

pub fn ollama_status(settings: &OllamaSettings) -> OllamaStatus {
    let out = Command::new("ollama").arg("list").output();
    match out {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let models = stdout
                .lines()
                .skip(1)
                .filter_map(|l| l.split_whitespace().next())
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            OllamaStatus {
                reachable: true,
                installed_models: models,
                selected_model: settings.model.clone(),
                message: format!("Ollama reachable via CLI, endpoint {}", settings.endpoint),
            }
        }
        Ok(o) => OllamaStatus {
            reachable: false,
            installed_models: vec![],
            selected_model: settings.model.clone(),
            message: format!(
                "Ollama unavailable (code={}): {}",
                o.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&o.stderr)
            ),
        },
        Err(e) => OllamaStatus {
            reachable: false,
            installed_models: vec![],
            selected_model: settings.model.clone(),
            message: format!("Cannot execute ollama CLI: {e}"),
        },
    }
}

pub fn diagnostic_summary(diag: &OllamaDiagnostic) -> String {
    format!(
        "state={} | message={} | hint={}",
        diag.state, diag.message, diag.hint
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_helper_works() {
        let start = Instant::now() - Duration::from_secs(2);
        assert!(exceeded_timeout(start, Duration::from_secs(1)));
    }

    #[test]
    fn abnormal_env_values_are_bounded() {
        std::env::set_var("PATA_OLLAMA_TIMEOUT_SEC", "999999");
        std::env::set_var("PATA_OLLAMA_RETRIES", "0");
        std::env::set_var("PATA_MAX_TOKENS", "999999");
        std::env::set_var("PATA_MODEL", "../../bad model");
        let cfg = settings_from_env();
        assert!(cfg.timeout_sec <= 300);
        assert!(cfg.retries >= 1);
        assert!(cfg.max_tokens <= 8192);
        assert!(!cfg.model.contains(".."));
    }
}
