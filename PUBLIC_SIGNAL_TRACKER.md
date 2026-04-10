# PUBLIC_SIGNAL_TRACKER — Public Signal Capture Sprint

This document defines how to capture, interpret, and operationalize the first external signals after public launch.

It is intentionally analytical and sober:
- no fabricated traction numbers,
- no inflated interpretation of weak signals,
- no new technical features,
- no additional architecture complexity.

## Scope (first signal window)

Recommended cadence:
- daily review for the first 14 days,
- then weekly review.

## Signals to track

1. **Stars**
2. **Forks**
3. **External issues opened**
4. **GitHub discussions activity**
5. **Hacker News / Reddit reactions**
6. **X / LinkedIn mentions**
7. **Benchmark reactions** (method quality, credibility, scenario requests)

## Lightweight tracking table

Use this markdown table in weekly updates:

| Date | Stars | Forks | External issues | Discussions | HN/Reddit reactions | X/LinkedIn mentions | Benchmark reactions | Notes |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| YYYY-MM-DD |  |  |  |  |  |  |  |  |

## Signal matrix: importance → interpretation → next move

| Signal | Importance | Interpretation | Next product move |
|---|---|---|---|
| Stars increasing, low issues | Medium | Top-of-funnel curiosity is growing, but deep adoption remains unclear. | Improve first-run prompts and explicitly ask for concrete feedback after initial setup. |
| Forks increasing, low discussions | Medium | Technical interest exists, but shared learning is limited. | Publish a short architecture Q&A and invite implementation-focused discussion threads. |
| External issues increasing (bugs/docs friction) | High | Real hands-on usage has started; onboarding/runtime friction is visible. | Prioritize fast fixes in quickstart, install path, and runtime troubleshooting docs. |
| GitHub discussions increasing (use-cases/questions) | High | Interest is expanding from code to product fit and practical usage. | Convert recurring discussion topics into roadmap candidates and FAQ updates. |
| HN/Reddit reactions increasing | Medium | Public visibility is improving; quality depends on criticism depth and reproducibility concerns. | Summarize recurring critiques and respond with concrete clarifications, examples, and doc updates. |
| X/LinkedIn mentions increasing, low conversion | Low to Medium | Awareness is rising, but activation into trial/feedback is weak. | Shorten README → quickstart → feedback path and add clearer calls to action. |
| Benchmark skepticism increasing | High | Trust gap in evaluation methodology or scenario relevance. | Improve benchmark transparency, assumptions, and scenario rationale in benchmark docs. |
| Benchmark positive reactions increasing | Medium | Technical narrative credibility is improving externally. | Publish benchmark deltas with clear changelogs and known limitations. |

## What to improve after first feedback

1. **Triage within 48 hours**
   - classify incoming feedback: bug / feature request / product feedback / benchmark feedback.
2. **Prioritize top 3 friction points**
   - focus first on onboarding and reproducibility blockers.
3. **Publish a short response note**
   - what was heard,
   - what will change now,
   - what is intentionally deferred.
4. **Update public artifacts**
   - README,
   - quickstart instructions,
   - benchmark notes,
   - `PUBLIC_FEEDBACK.md`,
   - `LAUNCH_VISIBILITY.md`.
5. **Close the loop publicly**
   - reference related issue/discussion links in updates.

## Operating rules

- Never claim traction that is not directly observable.
- Prefer concrete user evidence over assumptions.
- Keep decisions reversible and documented.
- Optimize for learning velocity, not vanity metrics.

## Related docs

- `PUBLIC_FEEDBACK.md`
- `LAUNCH_VISIBILITY.md`
- `USE_CASE_PROOFS.md`
- `benchmarks/public-comparison/BENCHMARKS.md`
