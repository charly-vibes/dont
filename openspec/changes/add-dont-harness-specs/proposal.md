# Change: add dont harness, help, and derived-query specs

## Why

The monolithic spec still holds the operational surface that makes `dont` usable by LLM harnesses: read-only derived commands, the agent-facing help and teaching surface, and the spawn-request protocol. These concerns are downstream of the envelope/payload work but are not yet captured as focused OpenSpec capabilities. Extracting them now makes the harness contract testable without dragging in imports or project-layout details.

## What Changes
- Add `dont-derived-queries` for read-only derived commands: `list`, `vocab`, `show`, `why`, `prime`, `doctor`, `schema`, and `examples`.
- Add `dont-agent-help` for the managed docs block, orientation prompt, tutorial/how-to/help surface, and `sync-docs` behaviour.
- Add `dont-spawn-protocol` for `guess` / `assume` / `overlook`, harness-mode detection, spawn timeouts, and `dont spawns` listing semantics.

## Deferred
- Rule-authoring and `dont rules` command semantics — separate rule-system concern (§13)
- Rule explanation payloads beyond help routing (`dont explain`) — depends on rule docs and rule inventory
- Project layout and `config.toml` schema — infrastructure concern (§14)
- Import adapters and importer-specific behaviour — separate integration concern (§15)

## Traceability
- `dont-derived-queries` is sourced from `dont-spec-v0_3_2.md` §10 command summaries plus §10.7.6 references to help/tutorial entry points.
- `dont-agent-help` is sourced from `dont-spec-v0_3_2.md` §§10.7.6, 11, and the managed block template.
- `dont-spawn-protocol` is sourced from `dont-spec-v0_3_2.md` §§10 command summaries and 12.

## Impact
- Affected specs: `dont-derived-queries`, `dont-agent-help`, `dont-spawn-protocol` (all new)
- Cross-references: `dont-payload-types` (PrimeView, WhyView, DoctorReport, ExamplesList, SchemaDoc, SpawnRequest), `dont-cli-surface` (flags/stdin/help/exit conventions), `dont-envelope` (envelope kinds), `dont-errors` (spawn-related and usage/internal errors)
- Affected workflow: future rule-system, import, and project-layout specs can reference these harness-facing capabilities instead of restating behaviour
