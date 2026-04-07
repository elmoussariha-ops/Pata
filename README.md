# Pata MVP+ (Rust local agent for macOS Apple Silicon)

Pipeline sécurisé réel:

`scan → retrieve → plan → patch → review → approve → apply → validate → memory → optimize`

## Installation macOS (Apple Silicon)
```bash
xcode-select --install || true
brew install rustup-init ollama
rustup-init -y
source "$HOME/.cargo/env"
```

## Build / Validation
```bash
cargo fmt --all
cargo check --offline || cargo check
cargo clippy --offline -- -D warnings || cargo clippy -- -D warnings
cargo test --offline || cargo test
```

## Commandes principales
```bash
cargo run -- scan
cargo run -- retrieve "scanner"
cargo run -- plan "fix scanner"
cargo run -- patch "fix scanner"
cargo run -- review patch-<id>
cargo run -- approve patch-<id> "manual-approved"
cargo run -- apply patch-<id>
cargo run -- validate
cargo run -- status
cargo run -- --verbose doctor
cargo run -- --verbose smoke-test
cargo run -- --verbose ollama-status
cargo run -- low-power-status
cargo run -- tui
```

## Mode low-power (MacBook Air M4)
Activation:
```bash
cargo run -- --low-power status
# ou
export PATA_LOW_POWER=1
cargo run -- status
```

Effets:
- retrieval top-N réduit
- timeout Ollama plafonné
- retries Ollama réduits
- max tokens réduit
- petite pause TUI pour limiter charge idle

## Diagnostics Ollama
```bash
cargo run -- ollama-check
cargo run -- ollama-status
cargo run -- model-status
cargo run -- doctor
cargo run -- smoke-test
```

## Variables d'environnement
```bash
export PATA_MODEL="qwen2.5-coder:7b-instruct-q4_K_M"
export PATA_OLLAMA_ENDPOINT="http://127.0.0.1:11434"
export PATA_OLLAMA_TIMEOUT_SEC=45
export PATA_OLLAMA_RETRIES=2
export PATA_TEMPERATURE=0.1
export PATA_MAX_TOKENS=1200
export PATA_LOW_POWER=0
```

## Mapping modèle conseillé
- 24+ GB RAM → `qwen2.5-coder:14b-instruct-q4_K_M`
- 16+ GB RAM → `qwen2.5-coder:7b-instruct-q4_K_M`
- <16 GB RAM → `deepseek-coder:6.7b-instruct-q4_K_M`

## Workflow recommandé (Mac)
```bash
cargo run -- scan
cargo run -- retrieve "error scanner"
cargo run -- plan "fix scanner error handling"
cargo run -- patch "fix scanner error handling"
# récupérer patch-<id> dans la sortie
cargo run -- review patch-<id>
cargo run -- approve patch-<id> "approved-after-review"
cargo run -- apply patch-<id>
cargo run -- validate
cargo run -- status
```

## Erreurs fréquentes
1. `doctor` => `binary-missing`
   - `brew install ollama`
2. `doctor` => `daemon-unreachable`
   - `ollama serve`
3. `doctor` => `model-missing`
   - `ollama pull <model>`
4. `apply` refusé
   - lancer `approve` pour créer `.pata/approvals/<id>.ok`
5. `apply` rollback après validation
   - corriger patch puis re-run pipeline


## Fichiers d'état rapide (.pata/state)
- `last_validate.txt`
- `last_ollama_diagnostic.txt`
- `last_status.txt`
- `last_warning.txt`
