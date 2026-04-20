## Context

The current OpenSpec proposal establishes the conceptual center of `dont`, but operational behaviour remains unsplit. The next useful cut line groups together project bootstrap concerns (`init`, seed vocabulary, and modes) and the lifecycle-adjacent verbs that operate around the four-verb epistemic core.

## Goals
- Capture initialization and operating-mode behaviour without mixing it back into the core verb spec
- Capture `lock`, `reopen`, `ignore`, and `verify-evidence` as a separate lifecycle-oriented capability
- Maintain traceability to the v0.3.2 monolith

## Non-Goals
- Specify envelopes, derived commands, or import adapters in this change
- Convert the entire remaining monolith in one pass

## Decisions
- Use two capabilities in this change:
  - `dont-init-modes`: initialization, seed vocabulary, per-project mode, and mode change events
  - `dont-lifecycle-verbs`: `lock`, `reopen`, `ignore`, and `verify-evidence`
- Keep seed vocabulary with init/modes because the seed is installed during initialization and constrains later lifecycle behavior.
- Keep evidence liveness with lifecycle verbs because the draft explicitly separates it from `dismiss`.

## Source Mapping
- `dont-init-modes` derives primarily from sections 4.4, 7, and 8.
- `dont-lifecycle-verbs` derives primarily from section 9A.

## Risks / Trade-offs
- Grouping seed vocabulary and modes together is broader than a tiny capability.
  - Mitigation: both are anchored to `dont init`, making the grouping operationally coherent.
- `verify-evidence` touches network behavior that could later deserve a separate capability.
  - Mitigation: keep this pass focused on the command contract and defer broader operational/network policy if needed.

## Migration Plan
1. Add operational capability specs for init/modes and lifecycle verbs.
2. Validate the new change strictly.
3. Follow with separate changes in this order:
   - envelopes, JSON contracts, and error taxonomy
   - data model, evidence, and import behavior
   - derived commands, spawn/orchestration, and operational diagnostics
