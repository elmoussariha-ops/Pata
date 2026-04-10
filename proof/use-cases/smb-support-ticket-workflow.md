# Use Case — SMB Support Ticket Workflow (SMB Persona)

## Context

A small business needs a concrete support/retention workflow with assumptions and follow-up metrics.

## Input

`Plan a weekly SMB support workflow to improve customer retention`

## Reasoning trace

Runtime trace shape is the same orchestrated flow (`ExecutionStarted` → `MemoryRetrieved` → `ReasoningPlanPrepared` → verification → final result).

> In this restricted environment, live scenario execution failed in benchmark runs, so per-run trace events are not available for this scenario.

## Verification result

- `N/A` in current restricted benchmark environment (live runs failed)

## Confidence score

- `N/A` in current restricted benchmark environment (live runs failed)

## Final output (deterministic contract snapshot)

SMB output contract is produced as:
- `BUSINESS_CONTEXT`
- `OPERATIONAL_OBJECTIVE`
- `ACTION_BACKLOG`
- `DECISION_SUPPORT`
- `FOLLOW_UP_METRICS`
- `FINAL_ANSWER`

## Evidence source

- deterministic smb output logic in `crates/cli/src/main.rs`
- benchmark status in `benchmarks/public-comparison/REPORT.md`
