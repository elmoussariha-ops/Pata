#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

FIXTURE_PATH="quickstart/flagship-demo/fixture_developer_output.json"
FORCE_OFFLINE=false

if [[ "${1:-}" == "--offline" ]]; then
  FORCE_OFFLINE=true
  shift
fi

GOAL="${1:-Fix rust compile error in ownership handling}"
TMP_JSON="$(mktemp -t pata-flagship-demo-XXXXXX.json)"

cleanup() {
  rm -f "$TMP_JSON"
}
trap cleanup EXIT

echo "🚀 Running Pata Flagship Demo (persona=developer)"

if [[ "$FORCE_OFFLINE" == true ]]; then
  echo ""
  echo "⚙️  OFFLINE MODE requested (--offline)"
  cp "$FIXTURE_PATH" "$TMP_JSON"
  DEMO_MODE="OFFLINE"
else
  echo ""
  echo "⚙️  LIVE MODE"
  echo "Command: cargo run -p cli -- --persona developer --goal \"$GOAL\" --config config/app.toml"
  echo ""

  if cargo run -q -p cli -- --persona developer --goal "$GOAL" --config config/app.toml > "$TMP_JSON"; then
    DEMO_MODE="LIVE"
  else
    echo "⚠️  LIVE MODE failed. Switching to FALLBACK OFFLINE MODE."
    cp "$FIXTURE_PATH" "$TMP_JSON"
    DEMO_MODE="FALLBACK_OFFLINE"
  fi
fi

python3 - "$TMP_JSON" "$GOAL" "$DEMO_MODE" <<'PY'
import json
import sys
from pathlib import Path

json_path = Path(sys.argv[1])
input_goal = sys.argv[2]
demo_mode = sys.argv[3]
payload = json.loads(json_path.read_text())

answer = payload.get("answer", "")
structured = payload.get("structured_output") or {}
trace = structured.get("execution_trace", {})
events = trace.get("events", [])

sections = {
    "ANALYSIS": "",
    "HYPOTHESIS": "",
    "ACTION_PLAN": "",
    "VALIDATION": "",
    "DURABLE_RULES_CHECK": "",
    "FINAL_ANSWER": "",
}

for line in answer.splitlines():
    for key in sections:
        prefix = f"{key}:"
        if line.strip().startswith(prefix):
            sections[key] = line.strip()[len(prefix):].strip()

print("=" * 78)
print("PATA V3 FLAGSHIP QUICKSTART DEMO")
print("=" * 78)
print(f"MODE: {demo_mode}")
print(f"Persona: {payload.get('persona', 'unknown')}")
print()
print("1) INPUT")
print(f"- Goal: {input_goal}")
print()
print("2) STRUCTURED REASONING (developer contract)")
for key in [
    "ANALYSIS",
    "HYPOTHESIS",
    "ACTION_PLAN",
    "VALIDATION",
    "DURABLE_RULES_CHECK",
    "FINAL_ANSWER",
]:
    value = sections[key] or "(section present in output but not line-split friendly)"
    print(f"- {key}: {value}")
print()
print("3) VERIFICATION")
print(f"- verification_status: {structured.get('verification_status', 'n/a')}")
print(f"- confidence_level: {structured.get('confidence_level', 'n/a')}")
failures = structured.get("global_failures", [])
print(f"- global_failures: {len(failures)}")
if failures:
    for f in failures:
        print(f"  - {f}")
print()
print("4) CONFIDENCE SCORE")
print(f"- confidence: {payload.get('confidence', 'n/a')}")
print()
print("5) OBSERVABLE TRACE")
run_id = trace.get("run_id", "n/a")
print(f"- run_id: {run_id}")
print(f"- events_count: {len(events)}")
for e in events[:6]:
    print(f"  - [{e.get('index','?')}] {e.get('event_type','?')}: {e.get('detail','')}")
if len(events) > 6:
    print(f"  - ... ({len(events)-6} more events)")
print()
print("6) FINAL OUTPUT")
print(f"- {sections['FINAL_ANSWER'] or payload.get('answer', '').splitlines()[-1]}")
print("=" * 78)
PY
