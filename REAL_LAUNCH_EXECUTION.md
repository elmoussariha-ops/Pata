# REAL_LAUNCH_EXECUTION — 48h Operational Checklist

Purpose: execute the first 48 hours after public diffusion with discipline, fast observation, and fast response.

This checklist is intentionally strict:
- no invented signals,
- no new technical feature work,
- no architecture changes,
- execution focus only.

Related tracker: `PUBLIC_SIGNAL_TRACKER.md`.

---

## T0 — Publish

### What to do

1. Publish launch artifacts (README updates, launch post(s), benchmark links, quickstart link).
2. Confirm all public links resolve correctly.
3. Start a single launch log entry (timestamp + channel + action).

### What to monitor

- publication success (post visible, links valid),
- first traffic/engagement traces,
- first inbound questions.

### How to respond

- acknowledge first questions quickly,
- redirect to canonical docs (README, QUICKSTART, benchmark notes),
- keep responses factual and concise.

### What to ignore

- vanity excitement with no concrete user action,
- speculative debates not tied to usage,
- pressure to add features in the first hours.

### Immediate product action triggers

- broken public link,
- quickstart blocker,
- invalid benchmark reference,
- major misunderstanding caused by unclear docs.

---

## T+6h — First signal check

### What to do

1. Run first structured signal scan using `PUBLIC_SIGNAL_TRACKER.md`.
2. Capture only observable facts (counts, links, issue/discussion URLs, quote snippets).
3. Label each signal as: noise / useful / action-required.

### What to monitor

- stars, forks,
- external issues,
- GitHub discussions,
- Hacker News / Reddit reactions,
- X / LinkedIn mentions,
- benchmark reactions.

### How to respond

- answer high-signal clarifying questions,
- open/triage issues for reproducible friction,
- document repeated confusion patterns.

### What to ignore

- non-actionable praise/criticism without reproducible detail,
- one-off comments with no pattern,
- platform drama unrelated to product use.

### Immediate product action triggers

- repeated onboarding failure pattern,
- reproducible runtime error,
- benchmark credibility concern with concrete evidence,
- public confusion repeated across multiple channels.

---

## T+24h — Response pass

### What to do

1. Consolidate the first 24h signals into top 3 friction points.
2. Publish a short public response note:
   - what we heard,
   - what we are changing now,
   - what is intentionally deferred.
3. Apply doc-first corrections (README, quickstart, benchmark clarifications).

### What to monitor

- whether clarified docs reduce repeated questions,
- whether issue quality improves after clarifications,
- whether discussion tone shifts from confusion to usage.

### How to respond

- prefer concrete fixes in public docs,
- link every response to a canonical source,
- keep scope tight to observed friction.

### What to ignore

- requests that require new architecture,
- broad roadmap expansion under launch pressure,
- unverified claims with no reproduction path.

### Immediate product action triggers

- unresolved blocker still preventing first successful run,
- conflicting docs creating contradictory guidance,
- high-credibility external report of a serious flaw.

---

## T+48h — Product learning pass

### What to do

1. Convert 48h observations into a compact learning memo:
   - strongest adoption signal,
   - strongest friction signal,
   - highest-confidence next product move.
2. Update `PUBLIC_SIGNAL_TRACKER.md` with confirmed patterns.
3. Align follow-up backlog with evidence strength (high / medium / low confidence).

### What to monitor

- retention of discussion quality,
- recurrence of the same friction points,
- conversion from visibility to real trial/feedback.

### How to respond

- prioritize reversible, low-complexity improvements,
- close feedback loops publicly with links,
- schedule next weekly signal review.

### What to ignore

- one-day anomalies presented as trends,
- metric spikes with no user-level corroboration,
- internal opinions not supported by external evidence.

### Immediate product action triggers

- a persistent friction point appears across channels,
- benchmark trust issues remain unresolved after clarification,
- evidence suggests onboarding path is still leaking heavily.

---

## Minimal operating rules

- Evidence first, interpretation second.
- Fast response, small scope, public traceability.
- No feature expansion during the 48h execution window.
- Every action should map to an observed external signal.

