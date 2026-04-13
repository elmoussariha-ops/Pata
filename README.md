# Pata (agent Rust local)

Pipeline actuel:

`scan → retrieve → plan → patch → review → approve → apply → validate → memory → optimize`

Garanties de sécurité des artefacts patch:
- checksums **SHA-256** sur patch/meta/review (détection de tampering local)

## Démarrage rapide
Le quickstart exécutable et maintenu est ici: [`QUICKSTART.md`](./QUICKSTART.md).

## Build / validation locale
```bash
cargo fmt --all
cargo check --all-targets --offline || cargo check --all-targets
cargo clippy --all-targets --offline -- -D warnings || cargo clippy --all-targets -- -D warnings
cargo test --all-targets --offline || cargo test --all-targets
cargo run -- evals
```

## Commandes principales supportées
```bash
cargo run -- scan
cargo run -- retrieve "scanner"
cargo run -- retrieve "scanner" --explain-retrieval
cargo run -- multiverse "stabiliser scanner" 3
cargo run -- ast-fingerprint
cargo run -- ast-fingerprint command_retrieve
cargo run -- plan "fix scanner"
cargo run -- patch "fix scanner"
cargo run -- review patch-<id>
cargo run -- review patch-<id> --explain-risk
cargo run -- approve patch-<id> "manual-approved"
cargo run -- apply patch-<id>
cargo run -- validate
cargo run -- evals
cargo run -- status
cargo run -- resume-session
cargo run -- end-session
cargo run -- memory show
cargo run -- doctor
cargo run -- smoke-test
cargo run -- ollama-check
cargo run -- ollama-status
cargo run -- model-status
cargo run -- low-power-status
cargo run -- tui
cargo run -- watch 30
```

## Evals versionnées
- `evals/cases.v1.tsv` : définition des checks de la suite `evals.2026-04-13.v1`
- `.pata/evals/validation_baseline.v1` : baseline runtime locale (créée après pipeline green)
- `.pata/evals/runs/evals.2026-04-13.v1.txt` : rapport d'exécution de référence

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
