#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

FIXTURE_PATH="quickstart/flagship-demo/fixture_developer_output.json"
OUTPUT_PATH="quickstart/recruiter-demo/DEMO_OUTPUT.md"
FORCE_OFFLINE=false

if [[ "${1:-}" == "--offline" ]]; then
  FORCE_OFFLINE=true
  shift
fi

SCENARIO="${1:-Un candidat en 3 minutes doit comprendre la valeur: un moteur agentique vérifiable plutôt qu un chatbot opaque.}"
GOAL="${2:-Fix rust compile error in ownership handling}"
TMP_JSON="$(mktemp -t pata-recruiter-demo-XXXXXX.json)"

cleanup() {
  rm -f "$TMP_JSON"
}
trap cleanup EXIT

echo "🎬 Pata Recruiter Demo (3 minutes)"
echo

if [[ "$FORCE_OFFLINE" == true ]]; then
  echo "⚙️  OFFLINE MODE requested (--offline)"
  cp "$FIXTURE_PATH" "$TMP_JSON"
  DEMO_MODE="OFFLINE"
else
  echo "⚙️  LIVE MODE"
  echo "Command: cargo run -q -p cli -- --persona developer --goal \"$GOAL\" --config config/app.toml"
  echo

  if cargo run -q -p cli -- --persona developer --goal "$GOAL" --config config/app.toml > "$TMP_JSON"; then
    DEMO_MODE="LIVE"
  else
    echo "⚠️  LIVE MODE failed. Switching to FALLBACK OFFLINE MODE."
    cp "$FIXTURE_PATH" "$TMP_JSON"
    DEMO_MODE="FALLBACK_OFFLINE"
  fi
fi

python3 quickstart/recruiter-demo/render_demo.py \
  "$TMP_JSON" \
  "$SCENARIO" \
  "$GOAL" \
  "$DEMO_MODE" \
  "$OUTPUT_PATH"
