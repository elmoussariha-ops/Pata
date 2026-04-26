# Proof benchmark

Benchmark minimal et reproductible pour le système agent Pata.

## Contenu

- `tasks.json` : suite standardisée (4 tâches).
- `run_benchmark.sh` : exécution benchmark + génération des sorties.
- `results.json` : résultats machine-readable.
- `report.md` : rapport lisible humainement.

## Exécution

```bash
./proof/benchmark/run_benchmark.sh
```

## Métriques par tâche

Chaque tâche produit :

- `task_id`
- `success`
- `steps_count`
- `tool_calls_count`
- `verification_passed`
- `final_confidence`

## Invariants

- Ne modifie pas le core agent.
- Utilise la config deterministic existante.
- Conserve une structure simple et auditabile.
