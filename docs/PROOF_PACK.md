# Pata Proof Pack (2-minute viral demo)

Le **Proof Pack** montre en quelques minutes pourquoi Pata est différent des agents “black-box” :

- raisonnement structuré lisible (format ReAct)
- mémoire et récupération de contexte visibles
- vérification explicite avant réponse finale
- traces d’exécution auditables

## TL;DR

Depuis la racine du repo :

```bash
./proof/run_demo.sh
```

Puis ouvrir :

- `proof/traces/latest/developer_code_review.trace.md`
- `proof/traces/latest/smb_support_ticket.trace.md`
- `proof/traces/latest/teacher_structured_tutoring.trace.md`

## Ce que la démo prouve immédiatement

1. **Comment l’agent pense**
   - Sections `ANALYSIS`, `HYPOTHESIS`, `ACTION_PLAN`, `VALIDATION`, `DURABLE_RULES_CHECK`, `FINAL_ANSWER`.
2. **Comment l’agent agit**
   - Le plan est transformé en résultat final orienté action.
3. **Comment l’agent se vérifie**
   - `verification_status`, `confidence_level`, `global_failures` sont exposés.
4. **Pourquoi c’est différent**
   - Les événements d’exécution (`ExecutionStarted`, `MemoryRetrieved`, `GlobalVerificationCompleted`, etc.) rendent le pipeline observable.

## Modes d’exécution

### Mode par défaut (offline déterministe)

```bash
./proof/run_demo.sh
```

- Aucun setup complexe.
- Utilise `proof/fixtures/*.json` pour garantir une démo stable et rapide.

### Mode live (optionnel)

```bash
./proof/run_demo.sh --live
```

- Lance la CLI réelle (`cargo run -p cli ...`).
- Si l’exécution live échoue sur un cas, fallback automatique vers le fixture correspondant.

## Structure du dossier `/proof`

- `proof/run_demo.sh` : orchestrateur principal de la démonstration
- `proof/inputs/*.goal` : prompts réalistes de démo
- `proof/fixtures/*.json` : sorties déterministes versionnées
- `proof/traces/latest/*.trace.md` : traces lisibles générées à chaque run
- `proof/traces/latest/*.raw.json` : payloads bruts

## Promesses respectées

- ✅ zéro changement de logique core agent
- ✅ aucun refactor architectural profond
- ✅ pas de dépendance externe inutile
- ✅ setup et exécution en moins de 2 minutes (mode offline)
- ✅ output orienté humain (markdown + sections explicites)

## Script reproductible pour reviewers

```bash
./proof/run_demo.sh && sed -n '1,140p' proof/traces/latest/developer_code_review.trace.md
```

Ce one-liner suffit pour valider le **Proof Pack** sur n’importe quel environnement standard avec bash + python3.
