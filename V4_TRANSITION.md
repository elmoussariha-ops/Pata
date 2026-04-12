# V4_TRANSITION — From Internal Build to Real-World Learning

V3 is complete. V4 starts only when public reality produces enough signal to justify roadmap movement.

This transition document is founder-level and evidence-driven:
- no speculative feature work,
- no intuition-only roadmap changes,
- no architecture complexity increase,
- post-launch learning only.

Linked operating documents:
- `PUBLIC_SIGNAL_TRACKER.md`
- `REAL_LAUNCH_EXECUTION.md`

---

## V4 entry criteria (minimum real-world evidence)

V4 is considered "opened" only when all five first signals are observed and documented:

1. **First stars**
   - evidence: observable stars on repository and timestamped capture.
2. **First forks**
   - evidence: at least one external fork, with link and date.
3. **First external issue**
   - evidence: issue opened by an external contributor/user.
4. **First technical feedback**
   - evidence: concrete external technical comment (architecture, runtime, benchmark, docs).
5. **First friction point**
   - evidence: reproducible onboarding or usage blocker seen in real external usage.

If one criterion is missing, V4 remains closed and execution stays in launch-learning mode.

---

## Roadmap decision framework (post-launch only)

### Step 1 — Capture (facts only)

- record signal source, date/time, link, and raw statement,
- separate observed fact from interpretation,
- reject unverifiable or second-hand claims.

### Step 2 — Classify

Assign each signal to one class:
- adoption signal,
- trust/credibility signal,
- friction signal,
- clarity/documentation signal,
- ecosystem/community signal.

### Step 3 — Score decision weight

For each signal, assess:
- recurrence (single / repeated / cross-channel),
- reproducibility (low / medium / high),
- user impact (local / broad),
- time sensitivity (can wait / should act now).

### Step 4 — Decide roadmap action

Allowed decisions only:
- **Document now** (docs clarification, no feature changes),
- **Fix now** (small, evidence-backed friction removal),
- **Observe** (continue collecting signal),
- **Defer explicitly** (not enough external evidence yet).

### Step 5 — Close loop publicly

- publish what was heard,
- publish what changed,
- publish what is deferred and why,
- link every decision to source signals.

---

## Decision matrix: signal → urgency → impact → roadmap decision

| Signal | Urgency | Product impact | Roadmap decision |
|---|---|---|---|
| First stars appear quickly after launch | Low | Indicates top-of-funnel interest, not usage depth | **Observe** and continue monitoring conversion to real usage |
| First forks appear | Medium | Suggests technical exploration and implementation intent | **Document now** with clearer architecture/extension guidance |
| First external issue reports reproducible blocker | High | Direct onboarding/usage friction with immediate adoption risk | **Fix now** with smallest possible corrective change |
| First technical feedback questions benchmark or methodology | High | Impacts credibility and trust in public narrative | **Document now** (clarify assumptions/methods) or **Fix now** if factual error exists |
| First friction point repeats across channels | High | Strong evidence of systemic user-path leakage | **Fix now** and validate via next signal review |
| Isolated non-reproducible criticism | Low | Weak decision confidence, uncertain user impact | **Observe** until recurrence/reproducibility improves |
| Feature request without usage evidence | Low | Potentially useful but speculative for current phase | **Defer explicitly** until external usage evidence accumulates |

---

## V4 operating guardrails

- No feature expansion without external signal evidence.
- No roadmap priority shift based on internal preference alone.
- No technical complexity added during early launch-learning cycles.
- Every accepted roadmap move must cite at least one real external signal.

---

## Practical execution rhythm

- Use `REAL_LAUNCH_EXECUTION.md` for the first 48h operational cadence.
- Use `PUBLIC_SIGNAL_TRACKER.md` for structured signal capture and interpretation.
- Reassess V4 entry criteria at each weekly review until all five are satisfied.

V4 is not a planned release by intuition.
V4 is a consequence of external reality.
