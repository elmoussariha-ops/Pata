# Public Launch Pack — Pata V2

Ce document fournit un kit de communication réutilisable pour présenter le projet publiquement sans survendre.

## 1) GitHub short description (repo "About")

**Option recommandée**

> Rust-first platform for specialized, verifiable AI agents with structured reasoning, observability, and multi-persona runtime.

## 2) Tagline finale (concise)

**Tagline recommandée**

> Rust-first platform for specialized, verifiable AI agents.

## 3) Note de release publique (réutilisable)

### Title

**Pata V2 Launch: Rust-first specialized AI agents with structured reasoning and verification**

### Body

Today we are opening Pata V2 as a Rust-first foundation for building specialized AI agents with explicit contracts.

What is available now:
- modular crates for traits, memory, reasoning, orchestration, personas, and runtime,
- deterministic orchestrated pipeline (memory → reasoning → verification → evaluation),
- 4 specialized personas (`developer`, `teacher`, `personal`, `smb`) through a persona registry,
- lightweight runtime surfaces (CLI and HTTP server),
- comparative docs and showcases to inspect behavior differences persona by persona.

What this release is (and is not):
- this is a credible, extensible technical foundation,
- not yet a full enterprise platform (no multi-tenant control plane, no advanced persistence stack).

If you want to explore quickly:
1. read `README.md`,
2. run `cargo run -p cli -- --list-personas`,
3. open `examples/multi_persona_demo.md`.

Feedback, critiques and contributions are welcome.

## 4) Message court (communautés / réseaux techniques)

### Version FR

On lance **Pata V2**: une plateforme **Rust-first** d’agents IA spécialisés.

- personas: `developer`, `teacher`, `personal`, `smb`
- pipeline structuré: mémoire → raisonnement → vérification → évaluation
- runtime déterministe: CLI + API HTTP

Objectif: une base open source crédible et extensible pour des agents spécialisés (pas un wrapper générique).

Repo + démo comparative: `README.md` et `examples/multi_persona_demo.md`.

### Version EN

We’re launching **Pata V2**: a **Rust-first** platform for specialized AI agents.

- personas: `developer`, `teacher`, `personal`, `smb`
- structured pipeline: memory → reasoning → verification → evaluation
- deterministic runtime: CLI + HTTP API

Goal: an open-source, extensible foundation for specialized agents (not a generic wrapper).

Start with `README.md` and `examples/multi_persona_demo.md`.

## 5) Post détaillé (GitHub / Reddit / Discord)

### Suggested post

We just released **Pata V2**, a Rust-first project focused on specialized, verifiable AI agents.

The core idea is simple: keep one orchestrated engine, then differentiate behavior through strong personas with explicit output contracts.

Current personas:
- `developer` (technical diagnosis + action + verification),
- `teacher` (pedagogical adaptation + understanding checks),
- `personal` (goal clarification + structured personal action plans),
- `smb` (small-business operational structuring + simple decision support).

What we implemented so far:
- deterministic memory/reasoning/orchestration pipeline,
- local + global verification steps,
- execution trace and deterministic evaluator,
- persona registry for runtime discovery and selection,
- minimal runtime interfaces (CLI + HTTP server),
- comparative docs to inspect persona differences.

What we did **not** claim yet:
- no enterprise multi-tenant layer,
- no advanced persistence stack,
- no marketplace complexity.

If you’re interested in Rust agent architectures with explicit contracts and observable behavior, feedback is very welcome.

Recommended first look:
1. `README.md`
2. `docs/launch_v2.md`
3. `examples/multi_persona_demo.md`
4. `examples/persona_comparison.md`

## 6) Fidelity checklist (before publishing)

- [x] Rust-first positioning is explicit
- [x] multi-persona specialization is explicit
- [x] structured reasoning + verification are explicit
- [x] observability and deterministic evaluation are explicit
- [x] scope limits are explicitly stated (no overclaim)
