## Context

The current OpenSpec proposal establishes the conceptual center of `dont`, but operational behaviour remains unsplit. The next useful cut line groups together project bootstrap concerns (`init`, seed vocabulary, and modes) and the lifecycle-adjacent verbs that operate around the four-verb epistemic core.

## Goals
- Capture initialization (`dont init`), operating-mode behavior (`permissive` vs `strict`), `DONT_HARNESS=1` integration context, and the explicitly authored seed vocabulary.
- Capture `lock` (claims only), `reopen` (from terminal states only), `ignore` (requires reason), and `verify-evidence` (trace warnings only) as a separate lifecycle-oriented capability.
- Maintain traceability to the v0.3.2 monolith while aligning with the strict data-model and rule-engine invariants established in prior specs.

## Non-Goals
- Specify envelopes, derived commands, or import adapters in this change
- Convert the entire remaining monolith in one pass

## Decisions
- Use two capabilities in this change:
  - `dont-init-modes`: initialization semantics (establishes `project:` entity), seed vocabulary (explicit `defined` events), mode changes (explicit `mode-changed` events), and the `DONT_HARNESS=1` environment variable.
  - `dont-lifecycle-verbs`: `lock`, `reopen`, `ignore`, and `verify-evidence`.
- **Seed vocabulary is fully authored**: Seed terms are explicit `defined` events in the first transaction, authored by `tool:dont-init`, preserving the append-only history invariant.
- **Mode overrides**: The `strict` mode acts as a global runtime override, elevating all `warn` severities to hard refusals, providing an instant kill-switch.
- **Terminal states are unconstrained**: `lock` and `ignore` transition entities without requiring atom completion.
- **Lock is claim-only**: Terms cannot be locked, preserving their ability to append redefinitions.
- **Ignore requires reason**: `ignore` strictly requires a non-empty, substantive reason (like `trust`).
- **Verify-evidence is a trace analysis**: Dead URIs found by `verify-evidence` emit a computed trace warning and an `evidence-checked` event, but MUST NOT mutate the persisted status of the claim/term.
- **Reopen is terminal-only**: restricted strictly to reversing the `locked` and `ignored` states back to `unverified`.
- **Filesystem layout deferred**: The semantic outcome of initialization is defined here, but physical paths (e.g. `.dont/config.toml`) are deferred to `dont-project-layout-specs`.

## Source Mapping
- `dont-init-modes` derives primarily from sections 4.4, 7, and 8.
- `dont-lifecycle-verbs` derives primarily from section 9A.

## Risks / Trade-offs
- Grouping seed vocabulary and modes together is broader than a tiny capability.
  - Mitigation: both are anchored to `dont init` and establish the initial graph state/ruleset, making the grouping operationally coherent.
- `verify-evidence` touches network behavior that could later deserve a separate capability.
  - Mitigation: keep this pass focused on the command contract (emitting trace warnings and events) and defer broader operational/network policy if needed.
- `DONT_HARNESS=1` spans integration concerns.
  - Mitigation: it's a global operational flag that forces `--json` everywhere, making its inclusion here appropriate alongside mode handling.

## Migration Plan
1. Add operational capability specs for init/modes and lifecycle verbs.
2. Validate the new change strictly.
3. Follow with separate changes in this order:
   - envelopes, JSON contracts, and error taxonomy
   - data model, evidence, and import behavior
   - derived commands, spawn/orchestration, and operational diagnostics
