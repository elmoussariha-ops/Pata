# Changelog

## v0.1.0 - 2026-04-13

Première release crédible orientée preuve technique.

### Added
- commande `evals` (alias explicite de validation + évaluation)
- persistance d'un rapport d'évaluation versionné dans `.pata/evals/runs/evals.2026-04-13.v1.txt`
- base d'évaluations versionnée dans `evals/cases.v1.tsv`
- quickstart exécutable minimal (`QUICKSTART.md`)

### Changed
- alignement README ↔ commandes réellement supportées
- CI: étape explicite `cargo run -- evals`
- version du crate définie à `0.1.0`

### Quality
- tests unitaires ajoutés pour la persistance des evals versionnées
