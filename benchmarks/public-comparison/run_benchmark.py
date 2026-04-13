#!/usr/bin/env python3
import argparse
import hashlib
import json
import statistics
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
REPORT_PATH = ROOT / "benchmarks/public-comparison/REPORT.md"
LATEST_JSON_PATH = ROOT / "benchmarks/public-comparison/latest.json"


@dataclass
class Scenario:
    key: str
    title: str
    persona: str
    goal: str


SCENARIOS = [
    Scenario(
        key="code_review",
        title="Code review",
        persona="developer",
        goal="Review a Rust patch for ownership and safety regressions",
    ),
    Scenario(
        key="structured_tutoring",
        title="Structured tutoring",
        persona="teacher",
        goal="Explain Rust ownership to a beginner with one guided exercise",
    ),
    Scenario(
        key="smb_support_workflow",
        title="SMB support workflow",
        persona="smb",
        goal="Plan a weekly SMB support workflow to improve customer retention",
    ),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run public comparison benchmark for Pata personas")
    parser.add_argument("--runs", type=int, default=5, help="Number of runs per scenario (default: 5)")
    parser.add_argument(
        "--p95-budget-ms",
        type=float,
        default=1200.0,
        help="Maximum acceptable p95 latency in ms for quality gate (default: 1200)",
    )
    parser.add_argument(
        "--min-verification-rate",
        type=float,
        default=0.8,
        help="Minimum acceptable verification success rate [0,1] for quality gate (default: 0.8)",
    )
    return parser.parse_args()


def run_once(scenario: Scenario) -> dict[str, Any]:
    cmd = [
        "cargo",
        "run",
        "-q",
        "-p",
        "cli",
        "--",
        "--persona",
        scenario.persona,
        "--goal",
        scenario.goal,
        "--config",
        "config/app.toml",
    ]

    start = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - start) * 1000

    result: dict[str, Any] = {
        "ok": proc.returncode == 0,
        "elapsed_ms": round(elapsed_ms, 2),
        "stderr": proc.stderr.strip(),
        "stdout": proc.stdout.strip(),
    }

    if proc.returncode == 0:
        try:
            payload = json.loads(proc.stdout)
            result["payload"] = payload
            result["answer_hash"] = hashlib.sha256(payload.get("answer", "").encode()).hexdigest()
        except json.JSONDecodeError as err:
            result["ok"] = False
            result["stderr"] = f"invalid JSON output: {err}"

    return result


def percentile(values: list[float], q: float) -> float | None:
    if not values:
        return None
    if len(values) == 1:
        return values[0]
    ordered = sorted(values)
    rank = (len(ordered) - 1) * q
    low = int(rank)
    high = min(low + 1, len(ordered) - 1)
    weight = rank - low
    return ordered[low] * (1 - weight) + ordered[high] * weight


def fmt_float(v: float | None) -> str:
    if v is None:
        return "N/A"
    return f"{v:.2f}"


def compute_summary_row(sc: Scenario, runs: list[dict[str, Any]], args: argparse.Namespace) -> dict[str, Any]:
    ok_runs = [r for r in runs if r["ok"]]

    if not ok_runs:
        execution_avg = sum(r["elapsed_ms"] for r in runs) / len(runs)
        err = runs[0].get("stderr") or "unknown error"
        return {
            "key": sc.key,
            "title": sc.title,
            "persona": sc.persona,
            "goal": sc.goal,
            "status": f"FAILED (0/{len(runs)})",
            "ok_runs": 0,
            "total_runs": len(runs),
            "execution_ms_avg": fmt_float(execution_avg),
            "execution_ms_p50": "N/A",
            "execution_ms_p95": "N/A",
            "execution_ms_stddev": "N/A",
            "confidence_avg": "N/A",
            "verification_rate": "N/A",
            "trace_events_avg": "N/A",
            "stability": "N/A",
            "quality_gate": "FAIL",
            "notes": f"Live runtime failed in this environment: `{err.splitlines()[-1]}`",
        }

    elapsed = [float(r["elapsed_ms"]) for r in ok_runs]
    confidence_vals = [
        float(r["payload"].get("confidence"))
        for r in ok_runs
        if r.get("payload") and isinstance(r["payload"].get("confidence"), (int, float))
    ]

    verification_hits = 0
    trace_counts: list[int] = []
    hashes: list[str] = []

    for run in ok_runs:
        payload = run["payload"]
        structured = payload.get("structured_output") or {}
        if structured.get("verification_status") == "Accept":
            verification_hits += 1
        trace_counts.append(len((structured.get("execution_trace") or {}).get("events") or []))
        hashes.append(run.get("answer_hash", ""))

    verification_rate_value = verification_hits / len(ok_runs)
    p95 = percentile(elapsed, 0.95)
    gate_pass = verification_rate_value >= args.min_verification_rate and (p95 is not None and p95 <= args.p95_budget_ms)

    unique_hashes = len({h for h in hashes if h})
    stability = "stable" if unique_hashes == 1 and len(hashes) > 1 else "variable"

    return {
        "key": sc.key,
        "title": sc.title,
        "persona": sc.persona,
        "goal": sc.goal,
        "status": f"OK ({len(ok_runs)}/{len(runs)})",
        "ok_runs": len(ok_runs),
        "total_runs": len(runs),
        "execution_ms_avg": fmt_float(sum(elapsed) / len(elapsed)),
        "execution_ms_p50": fmt_float(percentile(elapsed, 0.50)),
        "execution_ms_p95": fmt_float(p95),
        "execution_ms_stddev": fmt_float(statistics.pstdev(elapsed) if len(elapsed) > 1 else 0.0),
        "confidence_avg": fmt_float(sum(confidence_vals) / len(confidence_vals) if confidence_vals else None),
        "verification_rate": f"{verification_rate_value * 100:.0f}%",
        "trace_events_avg": fmt_float(sum(trace_counts) / len(trace_counts) if trace_counts else None),
        "stability": stability,
        "quality_gate": "PASS" if gate_pass else "FAIL",
        "notes": "",
    }


def render_report(results: list[dict[str, Any]], args: argparse.Namespace) -> str:
    lines: list[str] = []
    lines.append("# Public Comparison Benchmark Report")
    lines.append("")
    lines.append("_Generated by `benchmarks/public-comparison/run_benchmark.py`._")
    lines.append("")
    lines.append("## Method")
    lines.append("")
    lines.append("- 3 representative scenarios: code review, structured tutoring, SMB support workflow.")
    lines.append(f"- Each scenario is executed **{args.runs} times** to measure variance and reliability.")
    lines.append("- Metrics are computed from raw runtime outputs only (no synthetic scoring).")
    lines.append("")
    lines.append("## Quality gates")
    lines.append("")
    lines.append(f"- Verification rate gate: **>= {args.min_verification_rate * 100:.0f}%**")
    lines.append(f"- Latency gate (p95): **<= {args.p95_budget_ms:.0f} ms**")
    lines.append("")
    lines.append("## Comparative table")
    lines.append("")
    lines.append("| Scenario | Persona | Run status | p50 (ms) | p95 (ms) | Stddev (ms) | Confidence (avg) | Verification rate | Quality gate |")
    lines.append("|---|---|---|---:|---:|---:|---:|---:|---|")

    for row in results:
        lines.append(
            "| {title} | `{persona}` | {status} | {p50} | {p95} | {stddev} | {c} | {v} | **{gate}** |".format(
                title=row["title"],
                persona=row["persona"],
                status=row["status"],
                p50=row["execution_ms_p50"],
                p95=row["execution_ms_p95"],
                stddev=row["execution_ms_stddev"],
                c=row["confidence_avg"],
                v=row["verification_rate"],
                gate=row["quality_gate"],
            )
        )

    lines.append("")
    lines.append("## Scenario details")
    lines.append("")

    for row in results:
        lines.append(f"### {row['title']} (`{row['persona']}`)")
        lines.append("")
        lines.append(f"- Goal: `{row['goal']}`")
        lines.append(f"- Status: **{row['status']}**")
        lines.append(f"- Latency p50/p95 (ms): `{row['execution_ms_p50']}` / `{row['execution_ms_p95']}`")
        lines.append(f"- Latency stddev (ms): `{row['execution_ms_stddev']}`")
        lines.append(f"- Confidence avg: `{row['confidence_avg']}`")
        lines.append(f"- Verification rate: `{row['verification_rate']}`")
        lines.append(f"- Trace events avg: `{row['trace_events_avg']}`")
        lines.append(f"- Output stability: `{row['stability']}`")
        lines.append(f"- Quality gate: **{row['quality_gate']}**")
        if row["notes"]:
            lines.append(f"- Notes: {row['notes']}")
        lines.append("")

    return "\n".join(lines)


def write_latest_json(results: list[dict[str, Any]], args: argparse.Namespace) -> None:
    payload = {
        "generated_from": "benchmarks/public-comparison/run_benchmark.py",
        "runs_per_scenario": args.runs,
        "quality_gates": {
            "min_verification_rate": args.min_verification_rate,
            "max_p95_latency_ms": args.p95_budget_ms,
        },
        "scenarios": results,
    }
    LATEST_JSON_PATH.write_text(json.dumps(payload, indent=2) + "\n")


def main() -> int:
    args = parse_args()
    if args.runs < 2:
        raise SystemExit("--runs must be >= 2 for variance and stability analysis")

    summary_rows: list[dict[str, Any]] = []

    for sc in SCENARIOS:
        runs = [run_once(sc) for _ in range(args.runs)]
        summary_rows.append(compute_summary_row(sc, runs, args))

    report = render_report(summary_rows, args)
    REPORT_PATH.write_text(report)
    write_latest_json(summary_rows, args)

    print(f"Wrote benchmark report: {REPORT_PATH.relative_to(ROOT)}")
    print(f"Wrote machine-readable summary: {LATEST_JSON_PATH.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
