# Showcase — Persona Teacher

Ce document montre un scénario vitrine aligné avec l’implémentation actuelle (`agent-core` + `agent-memory` + `agent-reasoning` + `persona-teacher`).

## Objectif

Démontrer un flux enseignant crédible avec :

- explication claire,
- adaptation de niveau,
- structure pédagogique,
- vérification de cohérence,
- réponse finale orientée apprentissage.

## Scénario

**Input**: `"Explain Rust ownership to a beginner"`

Exécution rapide:

```bash
cargo run -p cli -- --goal "Explain Rust ownership to a beginner" --persona teacher --config config/app.toml
```

## Contrat de sortie attendu (persona-teacher)

- `LEARNING_OBJECTIVE:`
- `LEVEL_ADAPTATION:`
- `EXPLANATION:`
- `GUIDED_PRACTICE:`
- `UNDERSTANDING_CHECK:`
- `FINAL_ANSWER:`

## Pourquoi c’est une bonne vitrine

- prouve qu’on n’est pas limité au domaine développeur,
- réutilise exactement le même pipeline core,
- reste déterministe et testable,
- prépare l’ajout de nouvelles personas spécialisées.
