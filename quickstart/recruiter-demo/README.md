# Demo recruteur startup (≤ 3 minutes)

Objectif: permettre à un recruteur startup de comprendre rapidement la valeur de Pata **sans sur-promesse**.

## Lancer la démo

Depuis la racine du repo:

```bash
./quickstart/recruiter-demo/run.sh
```

Mode offline déterministe:

```bash
./quickstart/recruiter-demo/run.sh --offline
```

Optionnel (scénario + goal custom):

```bash
./quickstart/recruiter-demo/run.sh \
  "Je veux prouver en entretien qu'on a une architecture agentique vérifiable." \
  "Fix rust compile error in ownership handling"
```

## Ce que la démo produit

1. Un output terminal condensé et lisible (signaux clés).
2. Un fichier partageable prêt à envoyer:
   - `quickstart/recruiter-demo/DEMO_OUTPUT.md`

## Trame orale recommandée (3 min)

- **0:00–0:30** — problème: trop de démos IA sont opaques.
- **0:30–2:00** — avant/après: JSON brut vs lecture orientée valeur (raisonnement, vérification, confiance, trace).
- **2:00–3:00** — crédibilité: chiffres non inventés, valeurs issues du runtime réel (ou fixture offline versionnée).

## Contraintes respectées

- Pas de benchmark inventé.
- Pas de chiffres inventés.
- Fidèle à l'implémentation actuelle (pipeline + payload).
