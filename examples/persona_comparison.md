# Persona Comparison — Developer vs Teacher vs Personal vs SMB

Ce document compare les quatre personas actuellement enregistrées dans le registry.

| Dimension | `developer` | `teacher` | `personal` | `smb` |
|---|---|---|---|---|
| Mission principale | Résoudre des problèmes techniques et proposer des actions sûres | Expliquer des concepts de manière pédagogique | Aider à organiser des objectifs personnels avec un plan prudent | Aider une petite entreprise à structurer opérations et décisions |
| Structure attendue | `ANALYSIS`, `HYPOTHESIS`, `ACTION_PLAN`, `VALIDATION`, `DURABLE_RULES_CHECK`, `FINAL_ANSWER` | `LEARNING_OBJECTIVE`, `LEVEL_ADAPTATION`, `EXPLANATION`, `GUIDED_PRACTICE`, `UNDERSTANDING_CHECK`, `FINAL_ANSWER` | `CONTEXT_SUMMARY`, `PRIMARY_OBJECTIVE`, `ACTION_STRUCTURE`, `RISK_CHECK`, `NEXT_STEP`, `FINAL_ANSWER` | `BUSINESS_CONTEXT`, `OPERATIONAL_OBJECTIVE`, `ACTION_BACKLOG`, `DECISION_SUPPORT`, `FOLLOW_UP_METRICS`, `FINAL_ANSWER` |
| Cas d’usage typiques | Debug, refactor, vérification avant patch | Onboarding, explication graduelle, contrôle de compréhension | Organisation hebdomadaire, clarification d’objectifs, plan d’action personnel | Pilotage opérationnel, priorisation d’actions, suivi business |
| Garde-fous clés | Pas d’invention de résultats, vérification obligatoire | Adaptation de niveau obligatoire, check de compréhension obligatoire | Vérification explicite des contraintes/risques et plan réaliste | Hypothèses explicites, contraintes business, pas de certitudes légales/financières |

## Découverte runtime

CLI:

```bash
cargo run -p cli -- --list-personas
```

Serveur:

```bash
curl http://127.0.0.1:8080/personas
```
