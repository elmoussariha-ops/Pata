# Multi-persona Demo — Developer vs Teacher vs Personal vs SMB

Cette démonstration montre, de manière fidèle à la runtime actuelle, comment les 4 personas répondent différemment sur des demandes proches.

## Pourquoi cette démo est importante

Pour un même noyau pipeline (`memory + reasoning + verification + evaluation`), la spécialisation change :

- le **contrat de sortie**,
- le **style de raisonnement final**,
- les **garde-fous** appliqués.

## Comment exécuter rapidement

```bash
# 1) Voir les personas disponibles
cargo run -p cli -- --list-personas

# 2) Persona developer
cargo run -p cli -- \
  --goal "Fix rust compile error in ownership handling" \
  --persona developer \
  --config config/app.toml

# 3) Persona teacher
cargo run -p cli -- \
  --goal "Explain Rust ownership to a beginner" \
  --persona teacher \
  --config config/app.toml

# 4) Persona personal
cargo run -p cli -- \
  --goal "Help me organize my week with limited evening energy" \
  --persona personal \
  --config config/app.toml

# 5) Persona smb
cargo run -p cli -- \
  --goal "Improve customer retention with limited budget" \
  --persona smb \
  --config config/app.toml
```

## Lecture comparative (ce qui change réellement)

### 1) Developer

- **Intention**: diagnostiquer, proposer une action technique, vérifier.
- **Contrat**: `ANALYSIS`, `HYPOTHESIS`, `ACTION_PLAN`, `VALIDATION`, `DURABLE_RULES_CHECK`, `FINAL_ANSWER`.
- **Garde-fous**: pas d’invention de résultats, vérification obligatoire, changements prudents.

Exemple de forme attendue:

```text
ANALYSIS: ...
HYPOTHESIS: ...
ACTION_PLAN: ...
VALIDATION: ...
DURABLE_RULES_CHECK: ...
FINAL_ANSWER: ...
```

### 2) Teacher

- **Intention**: transmettre un concept avec adaptation de niveau.
- **Contrat**: `LEARNING_OBJECTIVE`, `LEVEL_ADAPTATION`, `EXPLANATION`, `GUIDED_PRACTICE`, `UNDERSTANDING_CHECK`, `FINAL_ANSWER`.
- **Garde-fous**: adaptation explicite du niveau (beginner/intermediate/advanced), check de compréhension.

Exemple de forme attendue:

```text
LEARNING_OBJECTIVE: ...
LEVEL_ADAPTATION: beginner ...
EXPLANATION: ...
GUIDED_PRACTICE: ...
UNDERSTANDING_CHECK: ...
FINAL_ANSWER: ...
```

### 3) Personal

- **Intention**: clarifier un objectif personnel et proposer un plan réaliste.
- **Contrat**: `CONTEXT_SUMMARY`, `PRIMARY_OBJECTIVE`, `ACTION_STRUCTURE`, `RISK_CHECK`, `NEXT_STEP`, `FINAL_ANSWER`.
- **Garde-fous**: prudence, contraintes explicites, prochaine action faisable.

Exemple de forme attendue:

```text
CONTEXT_SUMMARY: ...
PRIMARY_OBJECTIVE: ...
ACTION_STRUCTURE: ...
RISK_CHECK: ...
NEXT_STEP: ...
FINAL_ANSWER: ...
```

### 4) SMB

- **Intention**: structurer l’opérationnel d’une petite entreprise et guider des décisions simples.
- **Contrat**: `BUSINESS_CONTEXT`, `OPERATIONAL_OBJECTIVE`, `ACTION_BACKLOG`, `DECISION_SUPPORT`, `FOLLOW_UP_METRICS`, `FINAL_ANSWER`.
- **Garde-fous**: hypothèses explicites, contraintes business prises en compte, prudence sur les certitudes.

Exemple de forme attendue:

```text
BUSINESS_CONTEXT: ...
OPERATIONAL_OBJECTIVE: ...
ACTION_BACKLOG: ...
DECISION_SUPPORT: Assumption: ...
FOLLOW_UP_METRICS: ...
FINAL_ANSWER: ...
```

## Fidélité au pipeline réel

Cette comparaison est alignée avec le projet tel qu’implémenté aujourd’hui :

1. sélection de persona via registry,
2. exécution orchestrée (mémoire + plan de reasoning),
3. vérifications locales/globales,
4. validation du contrat de sortie par persona,
5. réponse finale + confiance + structured output.

En pratique, la valeur plateforme est précisément là : **même moteur, comportements spécialisés différents et vérifiables**.
