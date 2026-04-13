# Evals (versionnées)

Ce dossier convertit les showcases en **suite d'évaluation reproductible**.

## Fichiers

- `suite.v1.json`: cas d'évaluation versionnés (persona + goal + sections minimales attendues).

## Lancer les evals

```bash
cargo run -p cli -- --config config/app.toml --eval-suite evals/suite.v1.json --eval-output /tmp/pata-evals.json
```

- Le runner échoue (`exit 1`) si au moins un cas échoue.
- Le JSON `/tmp/pata-evals.json` peut être comparé entre runs dans CI ou local.

## Philosophie

- pas de benchmark artificiel,
- checks minimaux orientés contrats persona,
- stable et lisible pour review technique.
