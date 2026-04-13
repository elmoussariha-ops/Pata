#!/usr/bin/env python3
import json
import sys
from pathlib import Path

json_path = Path(sys.argv[1])
scenario = sys.argv[2]
goal = sys.argv[3]
demo_mode = sys.argv[4]
output_path = Path(sys.argv[5])
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
    stripped = line.strip()
    for key in sections:
        prefix = f"{key}:"
        if stripped.startswith(prefix):
            sections[key] = stripped[len(prefix):].strip()

verification_status = structured.get("verification_status", "n/a")
confidence_level = structured.get("confidence_level", "n/a")
confidence = payload.get("confidence", "n/a")
reasoning_steps = structured.get("reasoning_steps_executed", "n/a")
local_verifications = structured.get("local_verifications", "n/a")

raw_json_preview = json.dumps(payload, ensure_ascii=False, indent=2)
if len(raw_json_preview) > 500:
    raw_json_preview = raw_json_preview[:500].rstrip() + "\n..."

events_lines = []
for e in events[:5]:
    idx = e.get("index", "?")
    ev_type = e.get("event_type", "?")
    detail = e.get("detail", "")
    events_lines.append(f"- [{idx}] {ev_type}: {detail}")

if len(events) > 5:
    events_lines.append(f"- ... ({len(events) - 5} événements supplémentaires)")

lines = [
    "# Pata — Démo recruteur startup (≤ 3 min)",
    "",
    f"_Mode: **{demo_mode}**_",
    "",
    "## 1) Scénario de pitch (30 sec)",
    "",
    f"**Contexte:** {scenario}",
    "",
    "**Promesse démontrée:** Pata ne renvoie pas juste une réponse; il expose un **raisonnement structuré**, une **vérification explicite** et une **trace observable**.",
    "",
    "## 2) Avant / Après lisible (90 sec)",
    "",
    "### Avant (JSON brut difficile à pitcher en entretien)",
    "",
    "```json",
    raw_json_preview,
    "```",
    "",
    "### Après (lecture orientée valeur produit)",
    "",
    "| Signal | Ce qu'un recruteur comprend vite |",
    "|---|---|",
    f"| Goal | {goal} |",
    f"| Persona | {payload.get('persona', 'unknown')} (spécialisée, non générique) |",
    f"| Reasoning steps | {reasoning_steps} |",
    f"| Vérifications locales | {local_verifications} |",
    f"| Statut global | {verification_status} |",
    f"| Niveau de confiance | {confidence_level} |",
    f"| Score de confiance | {confidence} |",
    "",
    "**Contrat developer (résumé):**",
    f"- ANALYSIS: {sections['ANALYSIS'] or 'n/a'}",
    f"- HYPOTHESIS: {sections['HYPOTHESIS'] or 'n/a'}",
    f"- ACTION_PLAN: {sections['ACTION_PLAN'] or 'n/a'}",
    f"- VALIDATION: {sections['VALIDATION'] or 'n/a'}",
    f"- DURABLE_RULES_CHECK: {sections['DURABLE_RULES_CHECK'] or 'n/a'}",
    f"- FINAL_ANSWER: {sections['FINAL_ANSWER'] or 'n/a'}",
    "",
    "## 3) Ce qui est fidèle à l'implémentation réelle (45 sec)",
    "",
    "- Aucun benchmark inventé ni métrique simulée ajoutée.",
    "- Toutes les valeurs affichées proviennent du JSON runtime (live) ou de la fixture versionnée offline.",
    "- La démo reste alignée avec le pipeline existant: mémoire → reasoning → vérification → résultat.",
    "",
    "## 4) Trace observable à montrer à l'oral (30 sec)",
    "",
]

if events_lines:
    lines.extend(events_lines)
else:
    lines.append("- Aucun événement de trace.")

lines.extend([
    "",
    "---",
    "",
    "_Généré automatiquement par quickstart/recruiter-demo/run.sh_",
    "",
])

md = "\n".join(lines)
output_path.write_text(md)

print("=" * 78)
print("PATA RECRUITER DEMO — READY TO SHARE")
print("=" * 78)
print(f"Mode: {demo_mode}")
print(f"Scenario: {scenario}")
print(f"Goal: {goal}")
print()
print("Core signals:")
print(f"- persona: {payload.get('persona', 'unknown')}")
print(f"- verification_status: {verification_status}")
print(f"- confidence_level: {confidence_level}")
print(f"- confidence: {confidence}")
print(f"- reasoning_steps_executed: {reasoning_steps}")
print(f"- local_verifications: {local_verifications}")
print()
print("Readable final answer:")
print(f"- {sections['FINAL_ANSWER'] or '(n/a)'}")
print()
print(f"Shareable file generated: {output_path}")
print("=" * 78)
