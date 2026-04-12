# PRODUCT_OVERVIEW — Pata in 5 Minutes

## One-line thesis

**Pata is a Rust-first platform for specialized AI agents with structured reasoning, explicit verification, and observable execution.**

## What is this project?

Pata is an open-source product foundation that combines:
- modular Rust architecture,
- persona specialization,
- deterministic orchestration,
- runtime surfaces (CLI + HTTP),
- and public, reproducible demos/benchmarks.

It is designed to be understandable, testable, and extensible without rewriting the core every time a new specialized agent is added.

## Why this project exists

Most generic agent wrappers optimize for speed of experimentation, but often under-emphasize:
- explicit output contracts,
- deterministic checks,
- observable execution traces,
- and persona-level specialization quality.

Pata exists to make these concerns first-class from the start.

## Why Rust?

Rust is used for:
- explicit contracts and strong typing,
- predictable runtime behavior,
- maintainable modular boundaries,
- and high-confidence refactoring as the platform grows.

## Why specialized agents (not one generic persona)?

Because different tasks require different quality criteria and guardrails.

Current personas illustrate this directly:
- `developer`: technical diagnosis + action + validation,
- `teacher`: pedagogical adaptation + understanding checks,
- `personal`: personal planning + prudence/constraints,
- `smb`: small-business operational support + assumptions/metrics.

## Why verification + observability?

Pata treats reliability as product behavior, not afterthought:
- structured reasoning steps,
- local/global verification,
- confidence output,
- execution trace + summary,
- deterministic self-evaluation.

This gives users and contributors concrete evidence about how answers were produced.

## Core differentiation

1. **Rust-first architecture** with clear crate boundaries.
2. **Specialized persona contracts** instead of one generic prompt style.
3. **Structured reasoning + verification** baked into orchestration.
4. **Observable execution** through trace and summary artifacts.
5. **Deterministic runtime + benchmark assets** for reproducibility.

## Architecture at a glance

- `agent-traits`: shared contracts/types.
- `agent-memory`: 4-layer memory + retrieval/consolidation.
- `agent-reasoning`: deterministic reasoning + verification flow.
- `agent-core`: orchestration + evaluation + observability.
- `persona-*`: specialization contracts and guardrails.
- `persona-registry`: discovery and runtime persona creation.
- `cli`, `server`: runnable surfaces.

## Personas and use cases

- **Developer**: code review, debugging, safe refactor planning.
- **Teacher**: structured explanations and progression by level.
- **Personal**: goal clarification and practical action structuring.
- **SMB**: operational planning, simple decision support, action follow-up.

## Quickstart entry point

Fastest “see it now” path:

```bash
./quickstart/flagship-demo/run.sh
```

Offline-safe path:

```bash
./quickstart/flagship-demo/run.sh --offline
```

## Benchmark proof

Public comparison benchmark assets:

```bash
python3 benchmarks/public-comparison/run_benchmark.py
```

Generated report:
- `benchmarks/public-comparison/REPORT.md`
- `USE_CASE_PROOFS.md`

Metrics reported from runtime output only:
- execution time,
- confidence,
- verification rate,
- observable trace size,
- output stability.

## Why this is different from generic agent frameworks

Pata does not position itself as a prompt aggregator.
It positions itself as a specialized-agent platform with:
- explicit persona contracts,
- deterministic orchestration,
- inspectable verification behavior,
- and reproducible public evidence.

## Current maturity

Current state is **advanced foundation / early product**:
- architecture is coherent and runnable,
- personas are differentiated and testable,
- public docs, quickstart and benchmarks are present,
- scope limits are explicit (no enterprise control plane claims).

## Next product direction

Near-term direction remains pragmatic:
- richer persona-specific evaluation depth,
- stronger runtime DX,
- improved observability integrations,
- progressive persistence improvements,
- continued public benchmark transparency.

---

If you only read one file before deciding whether to try Pata, this should be it.
