# USE_CASE_PROOFS — Public End-to-End Value Evidence

This document consolidates practical use-case proofs for Pata.

It is intentionally factual:
- no synthetic performance claims,
- no hidden scoring,
- only runtime artifacts or explicit restricted-environment notes.

## Snapshot table

| Use case | Persona | Input type | Reasoning trace | Verification | Confidence | Output |
|---|---|---|---|---|---|---|
| Real code review flow | `developer` | compile/safety review | Observed (fixture-backed) | Accept | 0.88 | Structured final recommendation |
| Structured tutoring session | `teacher` | beginner learning request | Orchestrated shape documented | N/A (restricted env) | N/A (restricted env) | Structured pedagogical contract |
| SMB support ticket workflow | `smb` | operational support planning | Orchestrated shape documented | N/A (restricted env) | N/A (restricted env) | Structured SMB action contract |

## Scenario proofs

- [Real code review flow](proof/use-cases/code-review-flow.md)
- [Structured tutoring session](proof/use-cases/structured-tutoring-session.md)
- [SMB support ticket workflow](proof/use-cases/smb-support-ticket-workflow.md)

## How to reproduce

1. Run flagship demo:

```bash
./quickstart/flagship-demo/run.sh --offline
```

2. Run public benchmark:

```bash
python3 benchmarks/public-comparison/run_benchmark.py
```

3. Cross-check evidence sources referenced in each scenario file.

## Interpretation guidance

- `developer` proof includes concrete observed verification/confidence from versioned fixture.
- `teacher` and `smb` proofs are contract-faithful and benchmark-referenced, but live per-scenario confidence/verification are currently `N/A` in this restricted environment.
- This reflects environment constraints, not hidden or fabricated numbers.
