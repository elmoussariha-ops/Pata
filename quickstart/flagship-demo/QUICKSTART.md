# Quickstart Flagship Demo (V3)

This is the fastest way to understand Pata’s value in under 3 minutes.

## One-command launch

From the repository root:

```bash
./quickstart/flagship-demo/run.sh
```

Optional custom goal:

```bash
./quickstart/flagship-demo/run.sh "Fix rust compile error in ownership handling"
```

Offline-safe mode (no runtime/network dependency):

```bash
./quickstart/flagship-demo/run.sh --offline
```

## What this demo shows (clearly)

The script runs the **developer persona** and prints:

1. **Input goal**
2. **Structured reasoning** (`ANALYSIS`, `HYPOTHESIS`, `ACTION_PLAN`, `VALIDATION`, `DURABLE_RULES_CHECK`, `FINAL_ANSWER`)
3. **Verification status**
4. **Confidence score**
5. **Observable trace events**
6. **Final answer**

The rendered output is identical across:
- **LIVE MODE** (real CLI execution),
- **FALLBACK OFFLINE MODE** (automatic fallback if live fails),
- **OFFLINE MODE** (forced via `--offline`).

## Why this is flagship

It demonstrates the platform’s core differentiators in one shot:

- Rust-first architecture
- specialized persona behavior
- structured reasoning
- explicit verification
- observability via execution trace
- deterministic confidence/evaluation payload

## Requirements

- Rust toolchain (`cargo`)
- access to crates.io for dependency resolution on first run

## Troubleshooting

If dependencies cannot be downloaded in your environment, the runner automatically switches from **LIVE MODE** to **FALLBACK OFFLINE MODE** using the versioned fixture JSON.

You can also force deterministic offline execution any time with:

```bash
./quickstart/flagship-demo/run.sh --offline
```
