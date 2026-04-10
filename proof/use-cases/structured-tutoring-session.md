# Use Case — Structured Tutoring Session (Teacher Persona)

## Context

A beginner asks for a structured explanation of Rust ownership with progressive pedagogy.

## Input

`Explain Rust ownership to a beginner with one guided exercise`

## Reasoning trace

Runtime trace shape is the same orchestrated flow (`ExecutionStarted` → `MemoryRetrieved` → `ReasoningPlanPrepared` → verification → final result).

> In this restricted environment, live scenario execution failed in benchmark runs, so per-run trace events are not available for this scenario.

## Verification result

- `N/A` in current restricted benchmark environment (live runs failed)

## Confidence score

- `N/A` in current restricted benchmark environment (live runs failed)

## Final output (deterministic contract snapshot)

Teacher output contract is produced as:
- `LEARNING_OBJECTIVE`
- `LEVEL_ADAPTATION`
- `EXPLANATION`
- `GUIDED_PRACTICE`
- `UNDERSTANDING_CHECK`
- `FINAL_ANSWER`

## Evidence source

- deterministic teacher output logic in `crates/cli/src/main.rs`
- benchmark status in `benchmarks/public-comparison/REPORT.md`
