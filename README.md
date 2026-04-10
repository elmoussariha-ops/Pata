# Pata — Rust AI Agents

![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Status](https://img.shields.io/badge/Status-V2%20Launch%20Prep-6f42c1)
![Personas](https://img.shields.io/badge/Personas-4-success)

**Tagline:** _Rust-first platform for specialized, verifiable AI agents._

Plateforme **Rust-first** pour construire des agents IA spécialisés, fiables et observables.

Pata is an open-source Rust platform that combines:
- persona specialization,
- structured reasoning,
- explicit verification,
- execution observability,
- and deterministic evaluation.

It is designed for teams who want **more than generic agent wrappers**: a modular base for building credible specialized agents.

> 📌 Pour contribuer ou suivre la direction du projet:
> - [Understand the project in 5 minutes](PRODUCT_OVERVIEW.md)
> - [Visual architecture overview](VISUAL_OVERVIEW.md)
> - [Guide de contribution](CONTRIBUTING.md)
> - [Roadmap publique](ROADMAP.md)
> - [Index docs](docs/README.md)
> - [Launch note V2](docs/launch_v2.md)
> - [Public launch pack](docs/public_launch_pack.md)
> - [Demo comparative multi-persona](examples/multi_persona_demo.md)
> - [Quickstart flagship demo](quickstart/flagship-demo/QUICKSTART.md)
> - [Public comparison benchmarks](benchmarks/public-comparison/BENCHMARKS.md)
> - [Help shape V3 (public feedback)](PUBLIC_FEEDBACK.md)
> - [Use case proofs](USE_CASE_PROOFS.md)
> - [Launch visibility pack](LAUNCH_VISIBILITY.md)

**Statut du dépôt**: _V2 product phase_ (plateforme multi-personas spécialisée, runtime encore minimal).

## New here? Start with the 5-minute product overview

👉 [`PRODUCT_OVERVIEW.md`](PRODUCT_OVERVIEW.md)

## Prefer diagrams first? Use the visual overview

👉 [`VISUAL_OVERVIEW.md`](VISUAL_OVERVIEW.md)

## Launch snapshot (V2)

Pata is currently a **serious launch-ready foundation**, not a finished enterprise product:
- ✅ Rust-first architecture with clear crate boundaries.
- ✅ Multi-persona platform (`developer`, `teacher`, `personal`, `smb`) via registry.
- ✅ Deterministic runtime surfaces (CLI + HTTP server).
- ✅ Observable orchestrated pipeline (memory → reasoning → verification → evaluation).
- ⚠️ Deliberately minimal runtime integrations (no enterprise multi-tenant/persistence stack yet).

## Why this project / Pourquoi ce projet est différent

Pata ne vise pas un simple “wrapper LLM”.
Le dépôt est structuré pour combiner, dès les premières briques :

- **spécialisation par persona** (ex: `persona-developer`, `persona-teacher`, `persona-personal`, `persona-smb`)
- **registry de personas** pour découverte et sélection runtime propres
- **raisonnement structuré et vérifiable** (single-branch, déterministe)
- **mémoire multi-couches** (court terme, résumés, apprentissage, permanent)
- **observabilité native** (trace d’exécution + résumé)
- **évaluation qualité déterministe** (sans juge externe)

Objectif: devenir une base open source crédible pour des agents IA spécialisés en Rust.

## Demo comparative (à voir en premier)

👉 **Si vous découvrez le projet, commencez par la démo comparative** :
[`examples/multi_persona_demo.md`](examples/multi_persona_demo.md)

Elle montre, sur des scénarios proches, la différence concrète entre `developer`, `teacher`, `personal` et `smb`, tout en restant fidèle au pipeline réel (mémoire, reasoning, vérification, confiance finale).

## Quickstart flagship (3 minutes)

Pour un effet “wow” immédiat avec la persona `developer` :

```bash
./quickstart/flagship-demo/run.sh
# or fully offline:
./quickstart/flagship-demo/run.sh --offline
```

Ce one-liner affiche directement input, raisonnement structuré, vérification, score de confiance, trace observable et output final.

## Public comparison benchmark

To generate a shareable benchmark report for GitHub/HN/communities:

```bash
python3 benchmarks/public-comparison/run_benchmark.py
```

This produces `benchmarks/public-comparison/REPORT.md` with runtime-derived metrics only (time, confidence, verification, trace, stability), without inventing values.

## Use case proofs

For practical end-to-end value evidence across real scenarios:

👉 [`USE_CASE_PROOFS.md`](USE_CASE_PROOFS.md)

## Help shape V3 — feedback wanted

We are now in a public validation sprint and we actively want high-quality external feedback.

Before posting feedback, please run:

```bash
./quickstart/flagship-demo/run.sh --offline
python3 benchmarks/public-comparison/run_benchmark.py
```

Then use:
- feedback guidance: [`PUBLIC_FEEDBACK.md`](PUBLIC_FEEDBACK.md)
- issue templates: `.github/ISSUE_TEMPLATE/*`
- discussion templates: `.github/DISCUSSION_TEMPLATE/*`

## Launch visibility pack

Ready-to-publish launch copy for GitHub, Hacker News, Reddit, and X/LinkedIn:
👉 [`LAUNCH_VISIBILITY.md`](LAUNCH_VISIBILITY.md)

---

## Vision produit

Créer un noyau d’orchestration commun où chaque persona peut exploiter :

1. du contexte mémoire pertinent,
2. un plan de raisonnement clair,
3. des vérifications locales + globales,
4. une décision finale traçable,
5. une évaluation qualité exploitable.

Le tout avec une architecture modulaire, extensible et lisible.

### Ce qui est en place vs ce qui vient ensuite

- ✅ **Déjà en place**: mémoire 4 couches, reasoning structuré, orchestration, observabilité, évaluation déterministe, persona developer vitrine.
- 🔜 **Prévu ensuite**: evals par persona plus riches, persistance mémoire externe, amélioration DX, API server.
- 🧪 **Non implémenté à ce stade**: ToT multi-branches, DAG complexe, judge model externe, dashboard complet.

---

## Architecture générale

```text
crates/
├─ agent-traits/        # contrats et types partagés
├─ agent-memory/        # mémoire 4 couches + retrieval + consolidation
├─ agent-reasoning/     # raisonnement structuré + vérification locale/globale
├─ agent-core/          # orchestration end-to-end + observabilité + évaluation
├─ cli/                 # CLI stable pour lancer une persona spécialisée
├─ server/              # API minimale HTTP pour exécuter un run agentique
├─ persona-developer/   # persona orientée dev/Rust-first
├─ persona-personal/    # persona orientée assistant personnel structuré
├─ persona-smb/         # persona orientée opérations petite entreprise
├─ persona-registry/    # registre léger + métadonnées + factory runtime
└─ persona-teacher/     # persona orientée pédagogie structurée
```

### Briques principales

- **`agent-traits`**
  - interfaces communes: `Agent`, `Persona`, `Tool`, `ModelProvider`
  - types de base: `ExecutionContext`, `AgentResult`, etc.

- **`agent-memory`**
  - mémoire court terme
  - fiches de résumé d’interaction
  - mémoire d’apprentissage (erreurs/corrections)
  - mémoire permanente (source de vérité durable)
  - retrieval et consolidation déterministes

- **`agent-reasoning`**
  - pipeline: `Analyze -> Hypothesis -> ActionOrTest -> Validation`
  - vérification locale avec décisions de correction
  - vérification globale légère (score, confiance, décision)

- **`agent-core`**
  - `SimpleAgent` (minimal)
  - `OrchestratedAgent` (pipeline complet)
  - trace structurée (`ExecutionTrace`) + résumé (`ExecutionSummary`)
  - évaluation qualité (`DeterministicPipelineEvaluator`)

- **`persona-developer`**
  - objectifs, comportements, critères qualité, garde-fous explicites
  - contrat de sortie structuré orienté usage développeur

- **`persona-teacher`**
  - objectifs pédagogiques explicites (clarté, adaptation niveau, cohérence)
  - contrat structuré pour l’enseignement (objectif, adaptation, explication, pratique guidée, check compréhension)

- **`persona-personal`**
  - objectifs d’assistance personnelle (priorisation, organisation, actionnabilité)
  - contrat structuré orienté plan personnel prudent

- **`persona-smb`**
  - objectifs business SMB (opérations, décisions simples, suivi d’actions)
  - contrat structuré orienté pilotage opérationnel pragmatique

- **`persona-registry`**
  - enregistrement centralisé des personas disponibles
  - exposition de métadonnées structurées (description, objectifs, cas d’usage, garde-fous)
  - factory de sélection runtime sans logique conditionnelle dispersée

---

## Personas spécialisées (vitrine)

### Persona Developer

`persona-developer` est la première démonstration forte du projet.

Elle impose un format de réponse clair:

1. `ANALYSIS:`
2. `HYPOTHESIS:`
3. `ACTION_PLAN:`
4. `VALIDATION:`
5. `DURABLE_RULES_CHECK:`
6. `FINAL_ANSWER:`

Pourquoi c’est utile:

- réponses plus auditables
- meilleure compatibilité avec le pipeline de vérification
- meilleure réutilisation en contexte PR/debug/refactor

### Persona Teacher

`persona-teacher` démontre que la plateforme gère une spécialisation non-dev tout en réutilisant le même pipeline mémoire + raisonnement + vérification + évaluation.

Contrat de sortie:

1. `LEARNING_OBJECTIVE:`
2. `LEVEL_ADAPTATION:`
3. `EXPLANATION:`
4. `GUIDED_PRACTICE:`
5. `UNDERSTANDING_CHECK:`
6. `FINAL_ANSWER:`

### Persona Personal

`persona-personal` démontre un usage assistant personnel structuré (organisation, clarification d’objectifs, plan d’action prudent).

Contrat de sortie:

1. `CONTEXT_SUMMARY:`
2. `PRIMARY_OBJECTIVE:`
3. `ACTION_STRUCTURE:`
4. `RISK_CHECK:`
5. `NEXT_STEP:`
6. `FINAL_ANSWER:`

### Persona SMB

`persona-smb` démontre une spécialisation business pour petite entreprise (organisation opérationnelle, décisions simples, suivi concret).

Contrat de sortie:

1. `BUSINESS_CONTEXT:`
2. `OPERATIONAL_OBJECTIVE:`
3. `ACTION_BACKLOG:`
4. `DECISION_SUPPORT:`
5. `FOLLOW_UP_METRICS:`
6. `FINAL_ANSWER:`

---

## Quickstart

### 1) Prérequis

- Rust stable (`rustup` + `cargo`)
- Environnement capable d’accéder à crates.io

### 2) Cloner le dépôt

```bash
git clone <repo-url>
cd Pata
```

### 3) Vérifier le format

```bash
cargo fmt --all -- --check
```

### 4) Lancer les tests des briques principales

```bash
cargo test -p agent-memory
cargo test -p agent-reasoning
cargo test -p agent-core
cargo test -p persona-developer
cargo test -p persona-teacher
cargo test -p persona-personal
cargo test -p persona-smb
```

### 5) Lancer la CLI (persona au choix)

```bash
cargo run -p cli -- --list-personas
cargo run -p cli -- --goal "Fix rust compile error" --config config/app.toml
cargo run -p cli -- --goal "Explain Rust ownership to a beginner" --persona teacher --config config/app.toml
cargo run -p cli -- --goal "Help me organize my week" --persona personal --config config/app.toml
cargo run -p cli -- --goal "Improve customer retention with limited budget" --persona smb --config config/app.toml
```

### 6) Lancer le serveur API minimal

```bash
cargo run -p server
```

Puis appeler l’endpoint:

```bash
curl -X POST http://127.0.0.1:8080/run \
  -H "Content-Type: application/json" \
  -d '{"goal":"Fix rust compile error"}'

curl http://127.0.0.1:8080/personas
```

La persona active pour le serveur est configurée dans `config/app.toml`:

```toml
[persona]
name = "developer" # or "teacher" or "personal" or "smb"
```

### 7) Explorer le flux orchestré dans le code

Point d’entrée pipeline: `OrchestratedAgent` dans `crates/agent-core/src/lib.rs`.

---

## Exemple vitrine — flux développeur (persona + raisonnement + vérification + confiance)

Voir aussi: [`examples/persona_developer_showcase.md`](examples/persona_developer_showcase.md).
Comparatif rapide multi-persona: [`examples/persona_comparison.md`](examples/persona_comparison.md).
Demo guidée multi-persona: [`examples/multi_persona_demo.md`](examples/multi_persona_demo.md).

### Flux attendu

1. objectif développeur reçu (ex: bug compile Rust)
2. récupération de contexte mémoire pertinent
3. plan de raisonnement structuré
4. exécution étape par étape + vérification locale
5. vérification globale
6. résultat final avec confiance + trace + évaluation

### Extrait d’usage (conceptuel)

```rust
// construit un OrchestratedAgent avec DeveloperPersona + ModelProvider
let result = agent.run("Fix rust compile error", ExecutionContext::default()).await?;

println!("Answer: {}", result.answer);
println!("Confidence: {}", result.confidence);
println!("Verification: {}", result.structured_output["verification_status"]);
println!("Trace events: {}", result.structured_output["execution_trace"]["events"].as_array().unwrap().len());
println!("Eval score: {}", result.structured_output["evaluation"]["overall_score"]);
```

Ce flux est déjà couvert par les tests `agent-core` (orchestration nominale + présence/ordre d’événements).

## Configuration simple (TOML)

Configuration par défaut: `config/app.toml`.

```toml
[model]
mode = "deterministic"

[server]
host = "127.0.0.1"
port = 8080
```

Cette V2 initiale supporte volontairement un mode déterministe local pour faciliter les tests externes rapides.

---

## Philosophie technique

- **Rust-first**: sûreté mémoire + clarté des contrats
- **Deterministic-first**: comportement explicable avant sophistication
- **Specialization-first**: personas fortes plutôt que prompts génériques
- **Observability-first**: trace et résumé dès le début
- **Quality-first**: première couche d’évaluation sans dépendances lourdes

---

## Roadmap courte (réaliste)

- enrichir les suites d’evals par persona
- brancher l’observabilité sur un backend externe (optionnel)
- améliorer la persistance mémoire (fichier/DB)
- ajouter de nouvelles personas spécialisées sur les mêmes abstractions

Roadmap détaillée: [ROADMAP.md](ROADMAP.md).

---

## État actuel

Le projet est en phase de fondation avancée: les briques clés sont en place et testables.
L’objectif des prochaines étapes est d’augmenter la démontrabilité (exemples), la robustesse (evals), et l’adoption développeur.

## Contribuer

Le projet est ouvert aux contributions orientées architecture propre, testabilité et documentation.

- Process complet: [CONTRIBUTING.md](CONTRIBUTING.md)
- Point d’entrée docs: [docs/README.md](docs/README.md)
