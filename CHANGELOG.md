# Changelog

Toutes les évolutions notables de ce projet seront documentées ici.

Ce format s'inspire de Keep a Changelog et respecte le versioning sémantique.

## [0.1.0] - 2026-04-13

Première release publique crédible de **Pata**.

### Added
- Architecture Rust modulaire organisée en crates (`agent-traits`, `agent-memory`, `agent-reasoning`, `agent-core`, `persona-*`, `cli`, `server`).
- Registre de personas spécialisées (`developer`, `teacher`, `personal`, `smb`) avec contrats de sortie explicites.
- Pipeline agentique structuré: mémoire -> raisonnement -> vérification locale/globale -> évaluation déterministe.
- Surfaces d’exécution minimales: CLI et serveur HTTP.
- Parcours quickstart publiable (`quickstart/flagship-demo`) pour démonstration rapide.
- Matériel public de benchmark reproductible (`benchmarks/public-comparison`).
- Dossiers de preuves de cas d’usage (`proof/use-cases/*`, `USE_CASE_PROOFS.md`).

### Documentation
- Positionnement produit, limites et maturité explicités (`README.md`, `PRODUCT_OVERVIEW.md`).
- Documentation de contribution et de feedback public.
- Pack de lancement public et supports de visibilité.

### Known limitations (important)
- Le benchmark public peut échouer en environnement restreint (ex: tunnel réseau/proxy), avec métriques `N/A`.
- Intégrations runtime volontairement minimales à ce stade (pas de stack enterprise multi-tenant complète).
- Les preuves de use-cases `teacher` et `smb` restent partiellement contraintes par l’environnement d’exécution.

[0.1.0]: https://github.com/elmoussariha-ops/Pata/releases/tag/v0.1.0
