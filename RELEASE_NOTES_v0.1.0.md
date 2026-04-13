# Pata v0.1.0 — Public foundation release

Date: 2026-04-13

## TL;DR
Pata v0.1.0 is a **public, honest foundation** for building specialized AI agents in Rust.

This release focuses on credibility through:
- explicit architecture boundaries,
- reproducible runtime behavior,
- simple and transparent CI quality gates.

## What is included in v0.1.0

### Core platform
- Multi-crate Rust workspace with clear separation between contracts, memory, reasoning, orchestration, personas, CLI, and HTTP server.
- Deterministic orchestration flow: memory -> reasoning -> verification -> evaluation.
- Structured observability with run summaries and traces.

### Personas available
- `developer`
- `teacher`
- `personal`
- `smb`

All personas run through the same engine with persona-specific behavior contracts.

### Runtime surfaces
- CLI runtime for local execution.
- Minimal HTTP server runtime.

### Public assets
- Comparative examples and quickstart artifacts.
- Benchmark/report scaffolding for transparent public sharing.

## Quality bar for this release
- Rust formatting enforced.
- Clippy warnings treated as errors.
- Workspace tests executed in CI.

## Honest scope (what this is not yet)
- Not an enterprise-ready multi-tenant control plane.
- Not a production-grade persistence platform.
- Not claiming external traction metrics in this release.

## Suggested release title
`v0.1.0 — Public foundation for specialized, verifiable AI agents in Rust`

## Suggested GitHub release body (copy/paste)
Pata v0.1.0 is now public.

This release delivers a Rust-first foundation for specialized AI agents with explicit personas, structured reasoning, verification, and observable execution.

Included:
- modular workspace architecture (traits, memory, reasoning, orchestration, personas, runtimes),
- deterministic memory -> reasoning -> verification -> evaluation pipeline,
- four personas (`developer`, `teacher`, `personal`, `smb`),
- CLI + minimal HTTP server runtimes,
- launch/benchmark docs for transparent inspection.

Quality gate:
- cargo fmt --check,
- cargo clippy -D warnings,
- cargo test --workspace.

Scope note:
This is a credible technical base, not a finished enterprise platform yet.
