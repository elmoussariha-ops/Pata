# Public Comparison Benchmarks

This folder contains reproducible public benchmark assets for Pata.

## Run the benchmark

From repository root:

```bash
python3 benchmarks/public-comparison/run_benchmark.py --runs 5
```

Optional gate tuning:

```bash
python3 benchmarks/public-comparison/run_benchmark.py --runs 5 --p95-budget-ms 1200 --min-verification-rate 0.8
```

It runs 3 representative scenarios:
- code review (`developer`)
- structured tutoring (`teacher`)
- SMB support workflow (`smb`)

## Report output

The command generates:

- `benchmarks/public-comparison/REPORT.md` (human-readable report)
- `benchmarks/public-comparison/latest.json` (machine-readable summary)

The report includes runtime-derived metrics only:
- latency distribution (`p50`, `p95`, `stddev`),
- confidence,
- verification rate,
- observable trace size,
- output stability across multiple runs,
- explicit quality-gate status (`PASS` / `FAIL`).

## Notes on restricted environments

If live runtime execution fails (e.g. crates.io blocked), the report still generates and marks missing values as `N/A` without inventing numbers.
