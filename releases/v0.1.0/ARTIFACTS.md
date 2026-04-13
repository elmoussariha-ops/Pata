# Structure minimale des artefacts publics — v0.1.0

Cette structure vise le minimum utile pour une release publique crédible.

```text
releases/
└── v0.1.0/
    ├── RELEASE_NOTES.md
    ├── RELEASE_CHECKLIST.md
    └── ARTIFACTS.md

racine repo/
├── CHANGELOG.md
├── README.md
├── PRODUCT_OVERVIEW.md
├── quickstart/flagship-demo/QUICKSTART.md
├── benchmarks/public-comparison/BENCHMARKS.md
├── benchmarks/public-comparison/REPORT.md
├── USE_CASE_PROOFS.md
└── proof/use-cases/
    ├── code-review-flow.md
    ├── structured-tutoring-session.md
    └── smb-support-ticket-workflow.md
```

## Rôle de chaque artefact
- `CHANGELOG.md`: historique versionné, honnête, orienté faits.
- `RELEASE_NOTES.md`: message court et publiable pour GitHub Release.
- `RELEASE_CHECKLIST.md`: garde-fou opérationnel avant publication.
- `README.md` + `PRODUCT_OVERVIEW.md`: proposition de valeur + limites.
- `QUICKSTART.md`: première expérience reproductible.
- `BENCHMARKS.md` + `REPORT.md`: méthode + résultats observables.
- `USE_CASE_PROOFS.md` + `proof/use-cases/*`: preuves narratives reliées aux artefacts.

## Optionnel après v0.1.0 (pas bloquant)
- archive des sorties CLI de démonstration,
- badge de release + SHA du tag,
- template de “known issues” par release.
