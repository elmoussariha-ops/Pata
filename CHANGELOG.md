# Changelog

## [1.0.0] - 2026-04-07

### Added
- `benchmark` / `perf` command for lightweight cold/warm timings (`scan`, `retrieve`, `ast-fingerprint`) plus watch-refresh estimate.
- richer causal history commands: `history causality`, `history explain-break`, `history recommend-rollback`.
- markdown export helper: `export-report <digest|causality|suspect> [path]`.
- `io_fast` incremental read cache (`path+mtime+size`) used by scanner/retriever/ast fingerprint.
- compact patch causality persistence and suspect scoring integrated into memory digest.
- `LICENSE` file (MIT).

### Changed
- `history suspect` now complemented by dedicated causality/explanation/rollback recommendation views.
- `patch/apply` flow now records causality events (`delta`, `validate`, `failure`).
