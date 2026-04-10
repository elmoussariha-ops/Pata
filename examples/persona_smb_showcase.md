# Showcase — Persona SMB

Ce document montre un scénario vitrine aligné avec l’implémentation actuelle (`agent-core` + `agent-memory` + `agent-reasoning` + `persona-smb`).

## Objectif

Démontrer un flux business petite entreprise avec :

- contexte opérationnel clair,
- objectif court terme concret,
- backlog d’actions priorisé,
- support de décision avec hypothèses,
- suivi via métriques.

## Scénario

**Input**: `"Improve customer retention with limited budget"`

Exécution rapide:

```bash
cargo run -p cli -- --goal "Improve customer retention with limited budget" --persona smb --config config/app.toml
```

## Contrat de sortie attendu (persona-smb)

- `BUSINESS_CONTEXT:`
- `OPERATIONAL_OBJECTIVE:`
- `ACTION_BACKLOG:`
- `DECISION_SUPPORT:`
- `FOLLOW_UP_METRICS:`
- `FINAL_ANSWER:`

## Pourquoi c’est une bonne vitrine

- démontre une spécialisation métier concrète pour petite entreprise,
- conserve le même pipeline mémoire + reasoning + vérification,
- produit des actions business pragmatiques et suivables,
- renforce la crédibilité plateforme multi-persona.
