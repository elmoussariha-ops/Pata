# VISUAL_OVERVIEW — Pata at a Glance

> **Visual architecture overview** for fast project comprehension.

## 1) Architecture flow

```text
┌───────────────────────────────────────────────────────────────────────────┐
│                                PATA PLATFORM                             │
└───────────────────────────────────────────────────────────────────────────┘
           │
           ├── Runtime surfaces
           │     ├─ CLI (`cargo run -p cli ...`)
           │     └─ HTTP Server (`/health`, `/personas`, `/run`)
           │
           ├── Persona routing
           │     └─ persona-registry → developer | teacher | personal | smb
           │
           └── Orchestrated core
                 ├─ agent-memory      (4-layer memory)
                 ├─ agent-reasoning   (structured deterministic steps)
                 ├─ verification      (local + global checks)
                 ├─ observability     (trace + summary)
                 └─ evaluation        (deterministic quality scoring)
```

## 2) Request lifecycle

```text
User goal
  │
  ▼
Runtime input validation
  │
  ▼
Persona selection (registry)
  │
  ▼
OrchestratedAgent::run
  │
  ├─ Retrieve memory context
  ├─ Build reasoning plan
  ├─ Execute steps (Analyze → Hypothesis → ActionOrTest → Validation)
  ├─ Run local/global verification
  └─ Build final answer + structured_output
  │
  ▼
JSON response
  ├─ answer
  ├─ confidence
  └─ structured_output (verification, trace, evaluation)
```

## 3) Reasoning → Verification → Observability

```text
Reasoning step output
   │
   ▼
Local verification (per step)
   │ pass / revise
   ▼
Global verification (final decision + score)
   │
   ▼
Execution summary
   ├─ verification_status
   ├─ confidence_level
   ├─ global_failures
   └─ evaluation
   │
   ▼
Execution trace
   ├─ run_id
   └─ ordered pipeline events
```

## 4) Persona routing (visual)

```text
              persona-registry
                    │
      ┌─────────────┼─────────────┬─────────────┐
      ▼             ▼             ▼             ▼
  developer      teacher       personal        smb
  (code)         (pedagogy)    (planning)      (small business)
      │             │             │             │
      └─────────────┴─────────────┴─────────────┘
                    │
                    ▼
         same orchestrated deterministic engine
```

## 5) Visual persona comparison

| Persona | Primary intent | Output contract style | Guardrail emphasis |
|---|---|---|---|
| `developer` | technical diagnosis + safe actions | analysis/hypothesis/action/validation | avoid invented command outcomes |
| `teacher` | explain + adapt by learner level | objective/adaptation/explanation/check | explicit level adaptation + understanding check |
| `personal` | practical personal planning | context/objective/actions/risk/next step | constraints + prudence |
| `smb` | practical SMB operations support | business context/objective/backlog/metrics | assumptions + operational constraints |

## 6) Mini benchmark highlight

```text
Public benchmark pack:
  benchmarks/public-comparison/

Scenarios:
  1) Code review (developer)
  2) Structured tutoring (teacher)
  3) SMB support workflow (smb)

Reported metrics (runtime-derived only):
  - execution time
  - confidence
  - verification rate
  - observable trace size
  - output stability
```

> In restricted environments, report generation keeps `N/A` values instead of inventing results.

## 7) User path (clone → quickstart → benchmark → overview)

```text
git clone ...
   │
   ▼
./quickstart/flagship-demo/run.sh
   │
   ▼
python3 benchmarks/public-comparison/run_benchmark.py
   │
   ▼
Read PRODUCT_OVERVIEW.md + README + docs index
```

## 8) Coherence with PRODUCT_OVERVIEW

This visual sheet is aligned with `PRODUCT_OVERVIEW.md`:
- same scope,
- same architectural claims,
- same maturity level,
- no additional feature claims.
