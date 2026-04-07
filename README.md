# Pata — Assistant agentique Rust local (macOS Apple Silicon)

Pata est un assistant de code **100% local**, orienté Rust, inspiré du workflow des assistants modernes de terminal.

## Plan général

1. Scanner un projet Rust (multi-fichiers) et extraire des signaux de complexité.
2. Exécuter `cargo check`, `cargo test`, `cargo clippy` pour diagnostiquer.
3. Construire un plan d'actions priorisé.
4. Proposer des patches (assistés par modèle local via Ollama).
5. Contrôler les changements (garde-fous + rollback git).
6. Exécuter une boucle d'optimisation toutes les 10 minutes.
7. Journaliser toutes les actions (`.pata/history.jsonl`, `.pata/logs`).

## Architecture

- `src/main.rs`: orchestration CLI + REPL.
- `src/cli.rs`: commandes terminal (`analyze`, `diagnose`, etc.).
- `src/analyzer.rs`: analyse statique rapide multi-fichiers.
- `src/diagnostics.rs`: exécution outillée Cargo.
- `src/planner.rs`: planification d'actions.
- `src/model.rs`: connexion modèle local (Ollama).
- `src/patcher.rs`: génération/stockage de propositions de patch.
- `src/optimizer.rs`: cycle contrôlé toutes les 10 minutes.
- `src/rollback.rs`: checkpoints + rollback automatique.
- `src/history.rs`: historique JSONL complet.
- `src/config.rs`: configuration runtime.

## Choix techniques (MacBook Air M4)

- Langage coeur: **Rust** (performance, mémoire maîtrisée).
- Runtime async: **Tokio**.
- Modèle local: **Ollama** (Metal sur Apple Silicon).
- Reco modèle principal: `qwen2.5-coder:7b-instruct-q4_K_M`.
- Alternatives réalistes:
  - `deepseek-coder:6.7b-instruct-q4_K_M`
  - `codestral:22b` (plus lourd, seulement si mémoire suffisante)

## Boucle d'optimisation (10 minutes)

- Mesure des temps `cargo check/test`.
- Détection des points faibles.
- Suggestion de patch.
- Exécution tests.
- Application uniquement si autorisé (`allow_auto_apply=true`) et garde-fous validés.
- Rollback automatique via checkpoint git en cas d'échec.

## Démarrage

```bash
cargo build
cargo run -- repl
```

Commandes rapides:

```bash
cargo run -- analyze
cargo run -- diagnose
cargo run -- plan
cargo run -- optimize-once
cargo run -- history 20
```

## Configuration

Le premier lancement crée `.pata/config.json`.

Paramètres clés:

- `workspace`: chemin du dépôt Rust cible
- `model_backend`: Ollama (endpoint local)
- `model_name`: modèle local choisi
- `optimization_interval_seconds`: 600 (10 min)
- `allow_auto_apply`: `false` par défaut (sécurité)
- `protected_paths`: chemins interdits à la modification auto

## Limites connues

- L'application d'un patch est volontairement conservatrice.
- Le moteur ne modifie rien automatiquement tant que l'auto-apply n'est pas explicitement activé.
