# Pata v1.0.0 (Rust local agent for macOS Apple Silicon)

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

## Quickstart release v1.0.0
```bash
cargo run -- scan
cargo run -- retrieve "scanner"
cargo run -- plan "stabilize scanner path handling"
cargo run -- patch "stabilize scanner path handling"
# review + approve + apply gated manually
```

## Commandes principales
```bash
cargo run -- scan
cargo run -- retrieve "scanner"
cargo run -- retrieve "scanner" --explain-retrieval
cargo run -- multiverse "stabiliser scanner" 3
cargo run -- ast-fingerprint
cargo run -- ast-fingerprint command_retrieve
cargo run -- history suspect
cargo run -- history causality
cargo run -- history explain-break
cargo run -- history recommend-rollback
cargo run -- benchmark scanner
cargo run -- export-report digest
cargo run -- export-report causality pata-causality.md
cargo run -- plan "fix scanner"
cargo run -- patch "fix scanner"
cargo run -- review patch-<id>
cargo run -- review patch-<id> --explain-risk
cargo run -- approve patch-<id> "manual-approved"
cargo run -- apply patch-<id>
cargo run -- validate
cargo run -- status
cargo run -- resume-session
cargo run -- end-session
cargo run -- memory show
cargo run -- memory recent
cargo run -- memory open-loops
cargo run -- memory open-loops --priority
cargo run -- memory open-loops --recent
cargo run -- memory lessons
cargo run -- memory digest
cargo run -- memory failures
cargo run -- memory failure-recent
cargo run -- memory promote-failure fm-<ts>
cargo run -- memory explain-open-loop ol-<ts>
cargo run -- memory similar-functions command_retrieve
cargo run -- memory add-open-loop bug "panic scanner sur gros repo" 5 src/scanner.rs critical
cargo run -- memory resolve-open-loop ol-<ts>
cargo run -- memory add-lesson retrieval "booster modules touchés récemment"
cargo run -- --verbose doctor
cargo run -- --verbose smoke-test
cargo run -- --verbose ollama-status
cargo run -- low-power-status
cargo run -- tui
cargo run -- watch 30
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
bash -lc 'command -v ollama || brew install ollama'
ollama serve
ollama pull qwen2.5-coder:7b-instruct-q4_K_M
cargo run -- ollama-check
cargo run -- ollama-status
cargo run -- model-status
cargo run -- doctor
cargo run -- smoke-test
```

## Validation Ollama réelle (Mac)
1. Démarrer daemon: `ollama serve`
2. Vérifier modèle actif: `cargo run -- model-status`
3. Lancer diagnostic: `cargo run -- --verbose doctor`
4. Lancer génération test: `cargo run -- smoke-test`
5. Si OK, lancer pipeline normal (`scan → retrieve → plan → patch`).  

## Troubleshooting court
- `binary-missing` → installer Ollama (`brew install ollama`).
- `daemon-unreachable` → relancer `ollama serve`.
- `model-missing` → `ollama pull <model>`.
- `smoke-test blocked` → vérifier `PATA_OLLAMA_ENDPOINT` et pare-feu local.

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

## Mémoire longue durée locale (.pata/memory)
- `daily/YYYY-MM-DD.txt` : résumés de session/journée append-only
- `weekly/YYYY-Www.txt` : consolidation hebdomadaire légère
- `project_compact.txt` : snapshot compact du projet (open loops, lessons, patchs récents)
- `open_loops.tsv` : backlog explicite des tâches ouvertes/fermées
- `lessons.tsv` : leçons apprises structurées par catégorie
- `sessions.log` : index des sessions résumées pour reprise ciblée
