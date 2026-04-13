# Pata v0.1.0 — Première release publique

## TL;DR
`v0.1.0` publie une base produit **sérieuse mais encore early-stage**: architecture claire, personas spécialisées, quickstart, benchmark reproductible et preuves de cas d’usage.

## Ce qui est solide dans cette release
- **Architecture lisible et modulaire** en crates Rust, pensée pour évoluer sans casser le noyau.
- **Personas orientées usage réel**: `developer`, `teacher`, `personal`, `smb`.
- **Pipeline explicite et observable**: raisonnement structuré, vérification, trace et résumé.
- **Entrée rapide** via quickstart flagship.
- **Preuves publiques** via benchmark et use-case proofs versionnés.

## Honnêteté sur l’état actuel
- Ce n’est **pas** une plateforme enterprise complète.
- Les intégrations runtime sont volontairement minimales.
- Le benchmark peut produire des échecs réseau dans certains environnements restreints, avec métriques `N/A`.

## Pour évaluer rapidement le projet
```bash
./quickstart/flagship-demo/run.sh --offline
python3 benchmarks/public-comparison/run_benchmark.py
```

## Liens utiles
- Architecture & état du projet: `README.md`
- Vue produit 5 minutes: `PRODUCT_OVERVIEW.md`
- Quickstart: `quickstart/flagship-demo/QUICKSTART.md`
- Benchmark public: `benchmarks/public-comparison/BENCHMARKS.md`
- Proofs: `USE_CASE_PROOFS.md`
