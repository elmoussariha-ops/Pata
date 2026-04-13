# Quickstart (réel, minimal)

## 1) Préparer l'environnement
```bash
cargo build
```

## 2) Boucle minimale vérifiable
```bash
cargo run -- scan
cargo run -- retrieve "scanner"
cargo run -- plan "stabiliser scanner"
cargo run -- patch "stabiliser scanner"
# récupérer patch-<id>
cargo run -- review patch-<id>
cargo run -- approve patch-<id> "manual-approved"
cargo run -- apply patch-<id>
```

## 3) Validation + evals versionnées
```bash
cargo run -- evals
# écrit .pata/evals/runs/evals.2026-04-13.v1.txt
```

## 4) Vérification d'état
```bash
cargo run -- status
```
