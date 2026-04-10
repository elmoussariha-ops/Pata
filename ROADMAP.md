# Pata Roadmap

Cette roadmap reflète l’état **réel** actuel du dépôt (avril 2026) et les prochaines priorités.

## État actuel (déjà en place)

- Workspace Rust modulaire (`agent-traits`, `agent-memory`, `agent-reasoning`, `agent-core`, `persona-developer`)
- Mémoire 4 couches avec retrieval + consolidation déterministes
- Raisonnement structuré single-branch avec vérification locale et globale
- Orchestration bout-en-bout légère dans `agent-core`
- Observabilité initiale (trace + résumé d’exécution)
- Évaluation qualité déterministe de base
- Persona developer structurée (contrat de sortie + garde-fous)

## Portée V1 (foundation)

V1 vise une base crédible, modulaire et démontrable — pas encore une plateforme “production enterprise” complète.

## Next (prochaines étapes prioritaires)

1. **Stabilisation API inter-crates**
   - clarifier les surfaces publiques
   - réduire les zones expérimentales

2. **Evals par persona**
   - suites déterministes dédiées
   - critères explicites par usage

3. **Persistance mémoire**
   - backend fichier/DB minimal pour mémoire permanente
   - migration simple depuis l’in-memory backend

4. **Amélioration DX**
   - exemples exécutable plus complets
   - onboarding contributeur renforcé

## Plus tard (ambitions moyen terme)

- serveur API dédié
- authentification et multi-tenant (niveau entreprise)
- connecteurs d’outils supplémentaires
- dashboards d’observabilité
- chaines de vérification plus riches

## Hors scope actuel (non implémenté)

- Tree of Thoughts multi-branches
- DAG de tâches complexe
- parallélisation avancée de reasoning
- judge model externe pour evals
- benchmark performance massif/distribué

## Principes de pilotage

- Rust-first, modulaire, typé
- déterminisme et testabilité avant sophistication
- pas de surpromesse fonctionnelle
- docs alignées avec le code réellement présent
