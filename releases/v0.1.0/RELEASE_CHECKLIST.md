# Release Checklist — v0.1.0

> Objectif: publier une release publique crédible, utile et honnête.

## 1) Préparation produit/documentation
- [ ] Le `README.md` reflète l’état réel du projet (forces + limites).
- [ ] Les personas et leur périmètre sont explicités (`developer`, `teacher`, `personal`, `smb`).
- [ ] Le quickstart fonctionne au moins en mode offline.
- [ ] Le benchmark public est exécutable et ses limites environnementales sont documentées.
- [ ] Les use-case proofs sont accessibles et cohérents avec le benchmark.

## 2) Qualité technique minimale
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] Vérifier que `Cargo.toml` est bien en `version = "0.1.0"` au niveau workspace.

## 3) Artefacts publics de release
- [ ] `CHANGELOG.md` contient l’entrée `0.1.0`.
- [ ] `releases/v0.1.0/RELEASE_NOTES.md` prêt à publier.
- [ ] `releases/v0.1.0/ARTIFACTS.md` liste la structure minimale des artefacts.
- [ ] Créer le tag Git `v0.1.0`.
- [ ] Publier la release GitHub avec les release notes.

## 4) Contrôle “anti-survente”
- [ ] Aucun claim chiffré non traçable n’est publié.
- [ ] Toute métrique affichée provient d’artefacts reproductibles.
- [ ] Les zones non implémentées sont clairement annoncées.

## 5) Post-release immédiat
- [ ] Ouvrir le canal de feedback public (`PUBLIC_FEEDBACK.md`).
- [ ] Suivre les signaux initiaux (`PUBLIC_SIGNAL_TRACKER.md`, `POST_LAUNCH_SIGNAL_LOG.md`).
- [ ] Capturer les premières frictions utilisateur pour priorisation V0.2.
