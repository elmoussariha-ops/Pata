# Use Case — Real Code Review Flow (Developer Persona)

## Context

A Rust developer wants a quick, structured review for ownership/safety regressions before merging a patch.

## Input

`Fix rust compile error in ownership handling`

## Reasoning trace (observed)

- `ExecutionStarted`
- `MemoryRetrieved`
- `ReasoningPlanPrepared`
- `GlobalVerificationCompleted`
- `FinalResultProduced`

## Verification result

- `verification_status: Accept`
- `global_failures: 0`

## Confidence score

- `0.88`

## Final output

- `Apply scoped borrow refactor, re-run cargo test, then review diff.`

## Evidence source

- `quickstart/flagship-demo/fixture_developer_output.json`
