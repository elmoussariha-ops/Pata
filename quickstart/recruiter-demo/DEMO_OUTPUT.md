# Pata — Démo recruteur startup (≤ 3 min)

_Mode: **OFFLINE**_

## 1) Scénario de pitch (30 sec)

**Contexte:** Un candidat en 3 minutes doit comprendre la valeur: un moteur agentique vérifiable plutôt qu un chatbot opaque.

**Promesse démontrée:** Pata ne renvoie pas juste une réponse; il expose un **raisonnement structuré**, une **vérification explicite** et une **trace observable**.

## 2) Avant / Après lisible (90 sec)

### Avant (JSON brut difficile à pitcher en entretien)

```json
{
  "status": "ok",
  "persona": "developer",
  "goal": "Fix rust compile error in ownership handling",
  "answer": "ANALYSIS: Investigate error scope and constraints.\nHYPOTHESIS: Root cause likely ownership mismatch.\nACTION_PLAN: Apply scoped borrow refactor and run cargo check.\nVALIDATION: Verify no compiler errors and tests remain green.\nDURABLE_RULES_CHECK: No contradiction with project rules.\nFINAL_ANSWER: Apply scoped borrow refactor, re-run cargo test, then review diff.",
  "confiden
...
```

### Après (lecture orientée valeur produit)

| Signal | Ce qu'un recruteur comprend vite |
|---|---|
| Goal | Fix rust compile error in ownership handling |
| Persona | developer (spécialisée, non générique) |
| Reasoning steps | 4 |
| Vérifications locales | 4 |
| Statut global | Accept |
| Niveau de confiance | Medium |
| Score de confiance | 0.88 |

**Contrat developer (résumé):**
- ANALYSIS: Investigate error scope and constraints.
- HYPOTHESIS: Root cause likely ownership mismatch.
- ACTION_PLAN: Apply scoped borrow refactor and run cargo check.
- VALIDATION: Verify no compiler errors and tests remain green.
- DURABLE_RULES_CHECK: No contradiction with project rules.
- FINAL_ANSWER: Apply scoped borrow refactor, re-run cargo test, then review diff.

## 3) Ce qui est fidèle à l'implémentation réelle (45 sec)

- Aucun benchmark inventé ni métrique simulée ajoutée.
- Toutes les valeurs affichées proviennent du JSON runtime (live) ou de la fixture versionnée offline.
- La démo reste alignée avec le pipeline existant: mémoire → reasoning → vérification → résultat.

## 4) Trace observable à montrer à l'oral (30 sec)

- [0] ExecutionStarted: run started for goal: Fix rust compile error in ownership handling
- [1] MemoryRetrieved: retrieved 0 context items and 0 durable rules
- [2] ReasoningPlanPrepared: prepared plan with 4 steps
- [3] GlobalVerificationCompleted: global verification decision=Accept, score=0.88
- [4] FinalResultProduced: final result assembled and interaction stored

---

_Généré automatiquement par quickstart/recruiter-demo/run.sh_
