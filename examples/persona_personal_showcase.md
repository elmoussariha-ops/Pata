# Showcase — Persona Personal

Ce document montre un scénario vitrine aligné avec l’implémentation actuelle (`agent-core` + `agent-memory` + `agent-reasoning` + `persona-personal`).

## Objectif

Démontrer un flux d’assistant personnel structuré avec :

- clarification du contexte personnel,
- objectif prioritaire explicite,
- plan d’actions réaliste,
- vérification des risques et contraintes,
- prochaine action immédiate.

## Scénario

**Input**: `"Help me organize my week"`

Exécution rapide:

```bash
cargo run -p cli -- --goal "Help me organize my week" --persona personal --config config/app.toml
```

## Contrat de sortie attendu (persona-personal)

- `CONTEXT_SUMMARY:`
- `PRIMARY_OBJECTIVE:`
- `ACTION_STRUCTURE:`
- `RISK_CHECK:`
- `NEXT_STEP:`
- `FINAL_ANSWER:`

## Pourquoi c’est une bonne vitrine

- démontre une spécialisation non-technique et non-pédagogique,
- réutilise le même pipeline mémoire + raisonnement + vérification,
- produit des réponses actionnables et prudentes,
- confirme l’extensibilité du registry multi-persona.
