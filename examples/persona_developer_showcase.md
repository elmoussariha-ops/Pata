# Showcase — Persona Developer

Ce document montre un scénario vitrine aligné avec l’implémentation actuelle (`agent-core` + `agent-memory` + `agent-reasoning` + `persona-developer`).

> Pour la vitrine pédagogique, voir aussi `examples/persona_teacher_showcase.md`.

## Objectif

Démontrer un flux développeur crédible avec :

- raisonnement structuré,
- vérification locale et globale,
- confiance finale,
- trace d’exécution,
- évaluation qualité.

## Scénario

**Input**: `"Fix rust compile error"`

Exécution rapide:

```bash
cargo run -p cli -- --goal "Fix rust compile error" --config config/app.toml
```

## Étapes du pipeline

1. `OrchestratedAgent` reçoit l’objectif.
2. Le moteur récupère du contexte via `MemoryRetriever`.
3. Il construit un `ReasoningPlan` (`Analyze -> Hypothesis -> ActionOrTest -> Validation`).
4. Chaque étape produit un `StepResult` puis passe par `verify_and_push`.
5. Le moteur exécute `verify_global`.
6. Il produit un `AgentResult` contenant:
   - `answer`
   - `confidence`
   - `structured_output.verification_status`
   - `structured_output.execution_trace`
   - `structured_output.evaluation`

## Contrat de sortie attendu (persona-developer)

- `ANALYSIS:`
- `HYPOTHESIS:`
- `ACTION_PLAN:`
- `VALIDATION:`
- `DURABLE_RULES_CHECK:`
- `FINAL_ANSWER:`

## Lecture rapide du résultat

Points utiles à inspecter dans `structured_output`:

- `verification_status`
- `reasoning_steps_executed`
- `execution_trace.events`
- `evaluation.overall_score`
- `evaluation.dimension_scores`

## Pourquoi c’est une bonne vitrine

- montre un pipeline complet et observable,
- reste déterministe et testable,
- illustre la valeur d’une persona spécialisée,
- prépare naturellement des extensions (autres personas, evals plus riches, backend d’observabilité).
