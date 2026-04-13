# Release v0.1.0

## Title
v0.1.0 — Public foundation for verifiable Rust AI agents

## Release notes
Pata v0.1.0 is the first public baseline: a Rust-first, persona-based agent platform focused on clarity, verification, and reproducibility.

### Highlights
- Modular Rust workspace: memory, reasoning, orchestration, personas, CLI, and HTTP server.
- Persona-first design with four public personas (`developer`, `teacher`, `personal`, `smb`).
- Structured reasoning and verification flow with observable traces and confidence outputs.
- Reproducible quickstart and benchmark artifacts to evaluate behavior on concrete scenarios.
- CI gate on `main` for formatting, linting, and tests.

### Honest scope (important)
This release is a solid technical foundation, not a complete enterprise platform yet.
Runtime integrations remain intentionally minimal at this stage.

### Upgrade / usage
```bash
cargo run -p cli -- --persona developer --goal "Explain Rust ownership with a practical fix"
cargo run -p server
```

## Initial changelog
See `CHANGELOG.md` for the initial project changelog entry for `0.1.0`.
