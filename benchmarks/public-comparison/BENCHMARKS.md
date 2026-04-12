# Public Comparison Benchmarks

This folder contains reproducible public benchmark assets for Pata.

## Run the benchmark

From repository root:

```bash
python3 benchmarks/public-comparison/run_benchmark.py
```

It runs 3 representative scenarios:
- code review (`developer`)
- structured tutoring (`teacher`)
- SMB support workflow (`smb`)

## Report output

The command generates:

- `benchmarks/public-comparison/REPORT.md`

The report includes runtime-derived metrics only:
- execution time,
- confidence,
- verification rate,
- observable trace size,
- output stability across 2 runs.

## Notes on restricted environments

If live runtime execution fails (e.g. crates.io blocked), the report still generates and marks missing values as `N/A` without inventing numbers.
