# Contributing to Pata

Merci de votre intérêt pour Pata 🚀

Ce projet vise une base open source **Rust-first**, modulaire et crédible pour des agents IA spécialisés.

## Ce que nous recherchons

Contributions qui renforcent:

- la **fiabilité** (types clairs, validation explicite, tests)
- la **lisibilité** (architecture simple, docs cohérentes)
- la **spécialisation** (personas utiles et démontrables)
- l’**observabilité** et l’**évaluation** (sans complexité prématurée)

## Avant de commencer

1. Lisez le `README.md`.
2. Lisez `ROADMAP.md` pour comprendre les priorités.
3. Ouvrez une issue si la contribution est significative (nouvelle crate, changement d’API, refactor majeur).

## Setup local

```bash
git clone <repo-url>
cd Pata
cargo fmt --all -- --check
cargo test -p agent-memory
cargo test -p agent-reasoning
cargo test -p agent-core
cargo test -p persona-developer
```

## Règles de contribution

### 1) Garder l’architecture modulaire

- éviter le couplage fort entre crates
- préférer des interfaces explicites
- préserver les responsabilités de chaque crate

### 2) Rester fidèle à l’état réel

- ne pas documenter comme “fait” ce qui n’est pas implémenté
- si une capacité est partielle, le dire clairement
- éviter le marketing trompeur dans docs/PR

### 3) Éviter la complexité prématurée

- pas de framework lourd si une modélisation simple suffit
- prioriser les versions déterministes, testables, explicables

### 4) Ajouter des tests

Tout changement non trivial doit inclure des tests ciblés.

Exemples attendus:

- cas nominal
- cas d’échec/validation
- invariants de structure ou d’ordre

### 5) Documentation obligatoire

Si vous modifiez un comportement visible:

- mettez à jour le `README.md` et/ou les docs associées
- ajoutez un exemple si cela améliore l’adoption

## Conventions de PR

Une bonne PR contient:

1. **Motivation**: pourquoi ce changement
2. **Description**: ce qui change concrètement
3. **Testing**: commandes exécutées + résultats
4. **Limites connues**: ce qui reste à faire / non couvert

## Idées de contributions utiles

- amélioration des exemples de personas
- suites d’évaluation par persona (déterministes)
- amélioration de la persistance mémoire
- amélioration docs (onboarding, diagrammes, scénarios)

## Code of Conduct (light)

- respect, clarté, feedback orienté solution
- critiques techniques oui, attaques personnelles non
